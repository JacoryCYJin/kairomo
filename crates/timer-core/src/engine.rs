use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Engine {
    pub state: Snapshot,
}
pub struct PhaseAdvance {
    pub reached: Option<TargetReached>,
    pub completed: Option<Session>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetReached {
    pub kind: Kind,
    pub completed: bool,
}
impl Engine {
    pub fn restore(mut state: Snapshot) -> Self {
        if state.phase == Phase::Running {
            // Unknown time after the durable checkpoint is never silently counted.
            if let Some(session) = &mut state.current {
                if let Some(interval) = session.intervals.last_mut() {
                    interval.ended_at_ms = Some(state.checkpoint_at_ms);
                }
            }
            state.phase = Phase::Paused;
            state.needs_recovery = true;
            state.pause_reason = Some("recovery".into());
        }
        Self { state }
    }
    pub fn start(&mut self, id: String, title: String, now: i64) -> Result<(), TimerError> {
        if self.state.phase != Phase::Idle {
            return Err(TimerError::InvalidTransition);
        }
        if title.chars().count() > 160 {
            return Err(TimerError::TitleTooLong);
        }
        self.begin(id, Kind::Focus, title.trim().into(), now, true);
        Ok(())
    }
    fn begin(&mut self, id: String, kind: Kind, title: String, now: i64, running: bool) {
        let minutes = match kind {
            Kind::Focus => self.state.settings.focus_minutes,
            Kind::Break => self.state.settings.break_minutes,
        };
        self.state.current = Some(Session {
            id,
            kind,
            title,
            started_at_ms: now,
            ended_at_ms: None,
            target_ms: minutes as u64 * 60_000,
            focus_mode: self.state.settings.focus_mode.clone(),
            duration_ms: 0,
            intervals: if running {
                vec![Interval {
                    started_at_ms: now,
                    ended_at_ms: None,
                    duration_ms: 0,
                }]
            } else {
                Vec::new()
            },
            corrected: false,
        });
        self.state.phase = if running {
            Phase::Running
        } else {
            Phase::Paused
        };
        self.state.needs_recovery = false;
        self.state.pause_reason = if running { None } else { Some("ready".into()) };
        self.state.target_notified = false;
        self.state.checkpoint_at_ms = now;
    }
    /// Returns true once at the target. Reaching a target never changes the phase.
    pub fn advance(&mut self, delta_ms: u64, now: i64) -> bool {
        self.state.checkpoint_at_ms = now;
        if self.state.phase != Phase::Running {
            return false;
        }
        if let Some(s) = &mut self.state.current {
            s.duration_ms = s.duration_ms.saturating_add(delta_ms);
            if let Some(i) = s.intervals.last_mut() {
                i.duration_ms = i.duration_ms.saturating_add(delta_ms);
            }
            if !self.state.target_notified && s.duration_ms >= s.target_ms {
                self.state.target_notified = true;
                return true;
            }
        }
        false
    }
    /// Resolve the deadline using effective time, then prepare (never start) the next phase.
    pub fn advance_phase(&mut self, delta_ms: u64, now: i64, next_id: String) -> PhaseAdvance {
        let current = self.state.current.as_ref();
        let automatic = self.state.phase == Phase::Running
            && current.is_some_and(|s| s.kind == Kind::Break || s.focus_mode == FocusMode::Timed);
        let kind = current.map(|s| s.kind.clone());
        let remaining = current.map_or(0, |s| s.target_ms.saturating_sub(s.duration_ms));
        let ends = automatic && delta_ms >= remaining;
        let counted = if ends {
            delta_ms.min(remaining)
        } else {
            delta_ms
        };
        let cutoff = now.saturating_sub(delta_ms.saturating_sub(counted) as i64);
        let reached = self.advance(counted, if ends { cutoff } else { now });
        let completed = if ends {
            // Includes legacy overdue breaks resumed after upgrading.
            Some(
                self.switch(next_id, cutoff)
                    .expect("running phase has a session"),
            )
        } else {
            None
        };
        self.state.checkpoint_at_ms = now;
        PhaseAdvance {
            reached: if reached || ends {
                kind.map(|kind| TargetReached {
                    kind,
                    completed: ends,
                })
            } else {
                None
            },
            completed,
        }
    }
    pub fn pause(&mut self, now: i64, reason: &str) -> Result<(), TimerError> {
        if self.state.phase != Phase::Running {
            return Err(TimerError::InvalidTransition);
        }
        if let Some(s) = &mut self.state.current {
            if let Some(i) = s.intervals.last_mut() {
                i.ended_at_ms = Some(now);
            }
        }
        self.state.phase = Phase::Paused;
        self.state.pause_reason = Some(reason.into());
        self.state.checkpoint_at_ms = now;
        Ok(())
    }
    pub fn resume(&mut self, now: i64) -> Result<(), TimerError> {
        if self.state.phase != Phase::Paused {
            return Err(TimerError::InvalidTransition);
        }
        if let Some(s) = &mut self.state.current {
            // A prepared phase has no active interval or actual start time yet.
            if s.intervals.is_empty() {
                s.started_at_ms = now;
            }
            s.intervals.push(Interval {
                started_at_ms: now,
                ended_at_ms: None,
                duration_ms: 0,
            });
        }
        self.state.phase = Phase::Running;
        self.state.needs_recovery = false;
        self.state.pause_reason = None;
        self.state.checkpoint_at_ms = now;
        Ok(())
    }
    pub fn finish(&mut self, now: i64) -> Result<Session, TimerError> {
        if !self
            .state
            .current
            .as_ref()
            .is_some_and(|s| !s.intervals.is_empty())
        {
            return Err(TimerError::InvalidTransition);
        }
        let mut session = self
            .state
            .current
            .take()
            .ok_or(TimerError::InvalidTransition)?;
        if let Some(i) = session.intervals.last_mut() {
            if i.ended_at_ms.is_none() {
                i.ended_at_ms = Some(now);
            }
        }
        session.ended_at_ms = Some(now);
        self.state.phase = Phase::Idle;
        self.state.needs_recovery = false;
        self.state.pause_reason = None;
        self.state.target_notified = false;
        self.state.checkpoint_at_ms = now;
        Ok(session)
    }
    /// Discard only the session the confirmation referred to. No record is returned to save.
    pub fn cancel(&mut self, expected_id: &str, now: i64) -> Result<(), TimerError> {
        if self.state.current.as_ref().map(|s| s.id.as_str()) != Some(expected_id) {
            return Err(TimerError::InvalidTransition);
        }
        self.finish(now)?;
        Ok(())
    }
    pub fn switch(&mut self, id: String, now: i64) -> Result<Session, TimerError> {
        let current = self
            .state
            .current
            .as_ref()
            .ok_or(TimerError::InvalidTransition)?;
        let next = if current.kind == Kind::Focus {
            Kind::Break
        } else {
            Kind::Focus
        };
        let title = current.title.clone();
        let done = self.finish(now)?;
        self.begin(id, next, title, now, false);
        Ok(done)
    }
    pub fn configure(&mut self, settings: Settings) -> Result<(), TimerError> {
        settings.validate()?;
        self.state.settings = settings;
        if let Some(session) = &mut self.state.current {
            if session.intervals.is_empty() {
                let minutes = match session.kind {
                    Kind::Focus => self.state.settings.focus_minutes,
                    Kind::Break => self.state.settings.break_minutes,
                };
                session.target_ms = minutes as u64 * 60_000;
                session.focus_mode = self.state.settings.focus_mode.clone();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn active() -> Engine {
        let mut e = Engine::default();
        e.start("a".into(), "Read".into(), 1_000).unwrap();
        e
    }
    #[test]
    fn overtime_is_one_continuous_session() {
        let mut e = active();
        assert!(e.advance(50 * 60_000, 3_001_000));
        assert_eq!(e.state.phase, Phase::Running);
        assert!(!e.advance(12 * 60_000, 3_721_000));
        let done = e.switch("b".into(), 3_721_000).unwrap();
        assert_eq!(done.duration_ms, 62 * 60_000);
        assert_eq!(e.state.current.unwrap().kind, Kind::Break);
    }
    #[test]
    fn pauses_are_excluded_and_resume_opens_an_interval() {
        let mut e = active();
        e.advance(30_000, 31_000);
        e.pause(31_000, "manual").unwrap();
        e.advance(180_000, 211_000);
        e.resume(211_000).unwrap();
        e.advance(15_000, 226_000);
        let s = e.finish(226_000).unwrap();
        assert_eq!(s.duration_ms, 45_000);
        assert_eq!(s.intervals.len(), 2);
    }
    #[test]
    fn timed_focus_stops_at_its_exact_effective_target_then_waits_for_break_start() {
        let mut e = Engine::default();
        e.configure(Settings {
            focus_minutes: 1,
            focus_mode: FocusMode::Timed,
            ..Settings::default()
        })
        .unwrap();
        e.start("focus".into(), "Read".into(), 1000).unwrap();
        e.advance_phase(30_000, 31_000, "unused".into());
        e.pause(31_000, "manual").unwrap();
        e.resume(91_000).unwrap();
        let event = e.advance_phase(30_750, 121_750, "rest".into());
        assert_eq!(
            event.reached,
            Some(TargetReached {
                kind: Kind::Focus,
                completed: true
            })
        );
        let done = event.completed.unwrap();
        assert_eq!(done.duration_ms, 60_000);
        assert_eq!(done.ended_at_ms, Some(121_000));
        assert_eq!(done.intervals.last().unwrap().ended_at_ms, Some(121_000));
        assert_eq!(
            done.intervals.iter().map(|i| i.duration_ms).sum::<u64>(),
            60_000
        );
        assert_eq!(e.state.phase, Phase::Paused);
        assert_eq!(e.state.pause_reason.as_deref(), Some("ready"));
        assert_eq!(e.state.current.as_ref().unwrap().kind, Kind::Break);
        assert_eq!(e.state.checkpoint_at_ms, 121_750);
        assert!(e
            .advance_phase(90_000, 211_750, "unused".into())
            .completed
            .is_none());
        assert_eq!(e.state.current.as_ref().unwrap().duration_ms, 0);
    }
    #[test]
    fn rest_always_ends_at_target_for_both_modes_and_waits_for_focus_start() {
        for mode in [FocusMode::Manual, FocusMode::Timed] {
            let mut e = active();
            e.configure(Settings {
                focus_mode: mode.clone(),
                break_minutes: 1,
                ..Settings::default()
            })
            .unwrap();
            e.switch("rest".into(), 2000).unwrap();
            e.resume(10_000).unwrap();
            let event = e.advance_phase(60_999, 70_999, "next".into());
            assert_eq!(
                event.reached,
                Some(TargetReached {
                    kind: Kind::Break,
                    completed: true
                })
            );
            assert_eq!(event.completed.unwrap().duration_ms, 60_000);
            assert_eq!(e.state.phase, Phase::Paused);
            let next = e.state.current.as_ref().unwrap();
            assert_eq!(next.kind, Kind::Focus);
            assert_eq!(next.focus_mode, mode);
            assert!(next.intervals.is_empty());
            e.advance_phase(20_000, 90_999, "unused".into());
            assert_eq!(e.state.current.as_ref().unwrap().duration_ms, 0);
        }
    }
    #[test]
    fn mode_changes_are_latched_to_started_focus_and_legacy_data_defaults_to_manual() {
        let mut e = active();
        e.configure(Settings {
            focus_minutes: 1,
            focus_mode: FocusMode::Timed,
            ..Settings::default()
        })
        .unwrap();
        assert_eq!(
            e.state.current.as_ref().unwrap().focus_mode,
            FocusMode::Manual
        );
        assert!(e
            .advance_phase(51 * 60_000, 3_061_000, "unused".into())
            .completed
            .is_none());
        e.switch("rest".into(), 3_061_000).unwrap();
        e.resume(3_061_500).unwrap();
        e.switch("next".into(), 3_062_000).unwrap();
        assert_eq!(
            e.state.current.as_ref().unwrap().focus_mode,
            FocusMode::Timed
        );
        e.resume(3_063_000).unwrap();
        e.configure(Settings::default()).unwrap();
        assert_eq!(
            e.state.current.as_ref().unwrap().focus_mode,
            FocusMode::Timed
        );
        assert!(e
            .advance_phase(60_000, 3_123_000, "rest2".into())
            .completed
            .is_some());
        let mut legacy = serde_json::to_value(active().state).unwrap();
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("focus_mode");
        legacy["current"]
            .as_object_mut()
            .unwrap()
            .remove("focus_mode");
        let restored: Snapshot = serde_json::from_value(legacy).unwrap();
        assert_eq!(restored.settings.focus_mode, FocusMode::Manual);
        assert_eq!(restored.current.unwrap().focus_mode, FocusMode::Manual);
    }
    #[test]
    fn both_switches_wait_for_manual_start_and_exclude_the_waiting_gap() {
        let mut e = active();
        e.advance(25_000, 26_000);
        let focus = e.switch("rest".into(), 26_000).unwrap();
        assert_eq!(focus.duration_ms, 25_000);
        for (kind, start_at) in [(Kind::Break, 90_000), (Kind::Focus, 180_000)] {
            assert_eq!(e.state.phase, Phase::Paused);
            assert_eq!(e.state.pause_reason.as_deref(), Some("ready"));
            assert_eq!(e.state.current.as_ref().unwrap().kind, kind);
            assert!(e.state.current.as_ref().unwrap().intervals.is_empty());
            assert!(!e.advance(60_000, start_at));
            assert_eq!(e.state.current.as_ref().unwrap().duration_ms, 0);
            // Reopening preserves the prepared phase rather than starting it.
            e = Engine::restore(e.state.clone());
            assert!(!e.state.needs_recovery);
            e.resume(start_at).unwrap();
            assert_eq!(e.state.current.as_ref().unwrap().started_at_ms, start_at);
            e.advance(5_000, start_at + 5_000);
            e.pause(start_at + 5_000, "manual").unwrap();
            e.advance(10_000, start_at + 15_000);
            e.resume(start_at + 15_000).unwrap();
            e.advance(3_000, start_at + 18_000);
            let done = e
                .switch(format!("next-{start_at}"), start_at + 18_000)
                .unwrap();
            assert_eq!(done.duration_ms, 8_000);
            assert_eq!(done.intervals.len(), 2);
            assert_eq!(done.title, "Read");
        }
    }
    #[test]
    fn preparing_a_phase_uses_new_settings_but_does_not_change_started_sessions() {
        let mut e = active();
        e.switch("rest".into(), 2000).unwrap();
        e.configure(Settings {
            break_minutes: 7,
            ..Settings::default()
        })
        .unwrap();
        assert_eq!(e.state.current.as_ref().unwrap().target_ms, 7 * 60_000);
        e.resume(3000).unwrap();
        e.configure(Settings {
            break_minutes: 3,
            ..Settings::default()
        })
        .unwrap();
        assert_eq!(e.state.current.as_ref().unwrap().target_ms, 7 * 60_000);
    }
    #[test]
    fn recovery_never_counts_unconfirmed_time() {
        let mut e = active();
        e.advance(42_000, 43_000);
        let state: Snapshot =
            serde_json::from_str(&serde_json::to_string(&e.state).unwrap()).unwrap();
        let mut restored = Engine::restore(state);
        assert!(restored.state.needs_recovery);
        assert_eq!(restored.state.phase, Phase::Paused);
        restored.advance(10_000_000, 10_043_000);
        assert_eq!(restored.state.current.unwrap().duration_ms, 42_000);
    }
    #[test]
    fn partial_sessions_are_saved_and_double_actions_rejected() {
        let mut e = active();
        assert!(e.start("b".into(), "x".into(), 2000).is_err());
        e.advance(5_000, 6000);
        assert_eq!(e.finish(6000).unwrap().duration_ms, 5000);
        assert!(e.finish(6000).is_err());
        assert!(e.resume(6000).is_err());
    }
    #[test]
    fn sleep_pauses_without_automatic_resume() {
        let mut e = active();
        e.advance(1000, 2000);
        e.pause(2000, "sleep").unwrap();
        e.advance(3_600_000, 3_602_000);
        assert_eq!(e.state.phase, Phase::Paused);
        assert_eq!(e.state.current.unwrap().duration_ms, 1000);
    }
    #[test]
    fn settings_apply_to_next_session_not_current() {
        let mut e = active();
        e.configure(Settings {
            focus_minutes: 25,
            ..Settings::default()
        })
        .unwrap();
        assert_eq!(e.state.current.as_ref().unwrap().target_ms, 50 * 60_000);
        assert!(e
            .configure(Settings {
                focus_minutes: 0,
                ..Settings::default()
            })
            .is_err());
    }
    #[test]
    fn cancel_discards_running_paused_and_break_sessions_but_rejects_stale_ids() {
        for paused in [false, true] {
            let mut e = active();
            e.advance(30_000, 31_000);
            if paused {
                e.pause(31_000, "manual").unwrap();
            }
            assert!(e.cancel("old", 32_000).is_err());
            assert!(e.state.current.is_some());
            e.cancel("a", 32_000).unwrap();
            assert_eq!(e.state.phase, Phase::Idle);
            assert!(e.state.current.is_none());
            assert!(!e.state.target_notified);
            assert!(e.cancel("a", 33_000).is_err());
            e.start("new".into(), "Read".into(), 34_000).unwrap();
            assert_eq!(e.state.current.as_ref().unwrap().duration_ms, 0);
            e.switch("rest".into(), 35_000).unwrap();
            e.resume(35_500).unwrap();
            e.cancel("rest", 36_000).unwrap();
            assert!(e.state.current.is_none());
        }
    }

    #[test]
    fn prepared_focus_and_rest_reject_completion_cancel_and_skip_without_mutation() {
        for kind in [Kind::Focus, Kind::Break] {
            let mut e = Engine::default();
            e.begin("ready".into(), kind, "Read".into(), 1000, false);
            let before = serde_json::to_value(&e.state).unwrap();
            assert!(e.finish(2000).is_err());
            assert!(e.cancel("ready", 2000).is_err());
            assert!(e.switch("other".into(), 2000).is_err());
            assert_eq!(serde_json::to_value(&e.state).unwrap(), before);
            // Even a zero-duration start followed by pause enables the actions.
            e.resume(3000).unwrap();
            e.pause(3000, "manual").unwrap();
            assert_eq!(e.state.current.as_ref().unwrap().duration_ms, 0);
            assert!(e.clone().finish(3000).is_ok());
            assert!(e.clone().cancel("ready", 3000).is_ok());
            assert!(e.switch("other".into(), 3000).is_ok());
        }
    }
}
