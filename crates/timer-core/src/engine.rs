use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Engine {
    pub state: Snapshot,
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
        self.begin(id, Kind::Focus, title.trim().into(), now);
        Ok(())
    }
    fn begin(&mut self, id: String, kind: Kind, title: String, now: i64) {
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
            duration_ms: 0,
            intervals: vec![Interval {
                started_at_ms: now,
                ended_at_ms: None,
                duration_ms: 0,
            }],
            corrected: false,
        });
        self.state.phase = Phase::Running;
        self.state.needs_recovery = false;
        self.state.pause_reason = None;
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
        self.begin(id, next, title, now);
        Ok(done)
    }
    pub fn configure(&mut self, settings: Settings) -> Result<(), TimerError> {
        settings.validate()?;
        self.state.settings = settings;
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
}
