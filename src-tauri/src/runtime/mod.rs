mod export;
use std::{
    path::Path,
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use timer_core::{Engine, Interval, Phase, Repository, Session, Settings, Snapshot};
use timer_storage::SqliteRepository;
use uuid::Uuid;

pub struct Runtime {
    pub engine: Engine,
    pub repository: SqliteRepository,
    last_tick: Instant,
    last_wall: i64,
}
pub struct AppState(pub Mutex<Runtime>);
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
impl Runtime {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut repository = SqliteRepository::open(path).map_err(|e| e.to_string())?;
        let checkpoint = repository.load_checkpoint().map_err(|e| e.to_string())?;
        let engine = checkpoint.map(Engine::restore).unwrap_or_default();
        engine
            .state
            .settings
            .validate()
            .map_err(|e| e.to_string())?;
        repository
            .persist(&engine.state, &[])
            .map_err(|e| e.to_string())?;
        Ok(Self {
            engine,
            repository,
            last_tick: Instant::now(),
            last_wall: now_ms(),
        })
    }
    /// Prepare state privately, commit it, then expose it to all entry points.
    fn commit(
        &mut self,
        mut engine: Engine,
        done: &[Session],
        measured: Instant,
    ) -> Result<Snapshot, String> {
        engine.state.revision = self.engine.state.revision.saturating_add(1);
        self.repository
            .persist(&engine.state, done)
            .map_err(|e| e.to_string())?;
        self.engine = engine;
        self.last_tick = measured;
        self.last_wall = now_ms();
        Ok(self.engine.state.clone())
    }
    fn advanced(&self) -> (Engine, bool, Instant) {
        let measured = Instant::now();
        let mut next = self.engine.clone();
        let delta = measured.duration_since(self.last_tick).as_millis() as u64;
        let measured = self.last_tick + std::time::Duration::from_millis(delta);
        let now = now_ms();
        // If native suspend delivery is missed, a scheduling gap is unconfirmed time.
        if next.state.phase == Phase::Running
            && (delta > 8000 || now.saturating_sub(self.last_wall) > 8000)
        {
            let _ = next.pause(self.last_wall, "interrupted");
            return (next, false, measured);
        }
        let reached = next.advance(delta, now);
        (next, reached, measured)
    }
    pub fn tick(&mut self) -> Result<(Snapshot, bool), String> {
        let (next, reached, measured) = self.advanced();
        if next.state.phase != Phase::Running && self.engine.state.phase != Phase::Running {
            return Ok((self.engine.state.clone(), false));
        }
        Ok((self.commit(next, &[], measured)?, reached))
    }
    pub fn cancel_session(&mut self, expected_id: &str) -> Result<Snapshot, String> {
        let (mut next, _, measured) = self.advanced();
        next.cancel(expected_id, now_ms())
            .map_err(|e| e.to_string())?;
        self.commit(next, &[], measured)
    }
    pub fn action(&mut self, action: &str, title: &str) -> Result<Snapshot, String> {
        let (mut next, _, measured) = self.advanced();
        let now = now_ms();
        let mut completed = Vec::new();
        match action {
            "start" => next.start(Uuid::new_v4().to_string(), title.into(), now),
            "pause" => next.pause(now, "manual"),
            "resume" => next.resume(now),
            "switch" => next
                .switch(Uuid::new_v4().to_string(), now)
                .map(|s| completed.push(s)),
            "finish" => next.finish(now).map(|s| completed.push(s)),
            _ => return Err("invalid_action".into()),
        }
        .map_err(|e| e.to_string())?;
        self.commit(next, &completed, measured)
    }
    #[cfg(any(target_os = "macos", test))]
    pub fn sleep(&mut self) -> Result<(), String> {
        if self.engine.state.phase == Phase::Running {
            let (mut next, _, measured) = self.advanced();
            if next.state.phase == Phase::Running {
                next.pause(now_ms(), "sleep").map_err(|e| e.to_string())?;
            }
            self.commit(next, &[], measured)?;
        }
        Ok(())
    }
    pub fn shutdown(&mut self) -> Result<(), String> {
        if self.engine.state.phase == Phase::Running {
            let (mut next, _, measured) = self.advanced();
            if next.state.phase == Phase::Running {
                next.pause(now_ms(), "quit").map_err(|e| e.to_string())?;
            }
            self.commit(next, &[], measured)?;
        }
        Ok(())
    }
    pub fn configure(&mut self, settings: Settings) -> Result<Snapshot, String> {
        let (mut next, _, measured) = self.advanced();
        next.configure(settings).map_err(|e| e.to_string())?;
        self.commit(next, &[], measured)
    }
    pub fn preferences(
        &mut self,
        locale: Option<String>,
        appearance: Option<timer_core::Appearance>,
        visual_series: Option<String>,
    ) -> Result<Snapshot, String> {
        // Merge under the runtime lock so another window's saved values cannot be overwritten.
        let mut settings = self.engine.state.settings.clone();
        if let Some(locale) = locale {
            settings.locale = locale;
        }
        if let Some(appearance) = appearance {
            settings.appearance = appearance;
        }
        if let Some(visual_series) = visual_series {
            settings.visual_series = visual_series;
        }
        self.configure(settings)
    }
    pub fn correct(&mut self, id: &str, title: &str, minutes: f64) -> Result<(), String> {
        if title.chars().count() > 160 {
            return Err("title_too_long".into());
        }
        if !minutes.is_finite() || !(0.01..=1440.0).contains(&minutes) {
            return Err("invalid_duration".into());
        }
        let mut record = self
            .repository
            .sessions()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("record_not_found")?;
        record.title = title.trim().into();
        record.corrected = true;
        let duration = (minutes * 60_000.0).round() as u64;
        // A duration-only correction keeps the original placement and scales its intervals.
        let old = record.duration_ms;
        let count = record.intervals.len();
        let mut remaining = duration;
        if count == 0 {
            record.intervals.push(Interval {
                started_at_ms: record.started_at_ms,
                ended_at_ms: record.ended_at_ms,
                duration_ms: duration,
            });
        } else {
            for (index, interval) in record.intervals.iter_mut().enumerate() {
                let part = if index + 1 == count {
                    remaining
                } else if old == 0 {
                    0
                } else {
                    ((interval.duration_ms as u128 * duration as u128) / old as u128) as u64
                };
                interval.duration_ms = part;
                remaining = remaining.saturating_sub(part);
            }
        }
        record.duration_ms = duration;
        self.repository
            .update_session(&record)
            .map_err(|e| e.to_string())
    }
    pub fn export(&self, path: &str) -> Result<(), String> {
        export::write(
            path,
            &self.repository.sessions().map_err(|e| e.to_string())?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn with_runtime(test: impl FnOnce(&mut Runtime, &std::path::Path)) {
        let folder = std::env::temp_dir().join(format!("kairomo-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let file = folder.join("test.sqlite3");
        let mut runtime = Runtime::open(&file).unwrap();
        test(&mut runtime, &file);
        drop(runtime);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn running_checkpoint_recovers_without_counting_offline_time() {
        with_runtime(|rt, path| {
            rt.action("start", "task").unwrap();
            rt.last_tick = Instant::now() - std::time::Duration::from_secs(3);
            rt.tick().unwrap();
            let saved = rt.engine.state.current.as_ref().unwrap().duration_ms;
            let restored = Runtime::open(path).unwrap();
            assert_eq!(restored.engine.state.phase, Phase::Paused);
            assert!(restored.engine.state.needs_recovery);
            assert_eq!(restored.engine.state.current.unwrap().duration_ms, saved);
        });
    }
    #[test]
    fn missed_suspend_and_explicit_sleep_do_not_count_gap() {
        with_runtime(|rt, _| {
            rt.action("start", "task").unwrap();
            rt.last_tick = Instant::now() - std::time::Duration::from_secs(3600);
            rt.tick().unwrap();
            assert_eq!(rt.engine.state.phase, Phase::Paused);
            assert_eq!(rt.engine.state.current.as_ref().unwrap().duration_ms, 0);
            rt.action("resume", "").unwrap();
            rt.sleep().unwrap();
            assert_eq!(rt.engine.state.pause_reason.as_deref(), Some("sleep"));
        });
    }
    #[test]
    fn task_switch_correction_and_delete_roundtrip() {
        with_runtime(|rt, _| {
            rt.action("start", "Read").unwrap();
            rt.last_tick = Instant::now() - std::time::Duration::from_secs(2);
            rt.action("switch", "").unwrap();
            let first = rt.repository.sessions().unwrap().remove(0);
            assert_eq!(first.kind, timer_core::Kind::Focus);
            assert_eq!(
                rt.engine.state.current.as_ref().unwrap().kind,
                timer_core::Kind::Break
            );
            rt.correct(&first.id, "=book", 62.0).unwrap();
            let corrected = rt.repository.sessions().unwrap().remove(0);
            assert_eq!(corrected.duration_ms, 62 * 60_000);
            assert_eq!(
                corrected
                    .intervals
                    .iter()
                    .map(|i| i.duration_ms)
                    .sum::<u64>(),
                corrected.duration_ms
            );
            assert!(rt.correct(&first.id, "x", f64::NAN).is_err());
            rt.repository.delete_session(&first.id).unwrap();
            assert!(rt.repository.sessions().unwrap().is_empty());
        });
    }
    #[test]
    fn ending_a_short_paused_break_immediately_starts_focus_and_saves_rest() {
        with_runtime(|rt, _| {
            rt.action("start", "Read").unwrap();
            rt.last_tick = Instant::now() - std::time::Duration::from_secs(1);
            rt.action("switch", "").unwrap();
            rt.last_tick = Instant::now() - std::time::Duration::from_secs(2);
            rt.action("pause", "").unwrap();
            let next = rt.action("switch", "").unwrap();
            assert_eq!(next.phase, Phase::Running);
            let current = next.current.unwrap();
            assert_eq!(current.kind, timer_core::Kind::Focus);
            assert_eq!(current.duration_ms, 0);
            let records = rt.repository.sessions().unwrap();
            assert_eq!(records.len(), 2);
            assert_eq!(
                records
                    .iter()
                    .filter(|s| s.kind == timer_core::Kind::Focus)
                    .count(),
                1
            );
            let rest = records
                .iter()
                .find(|s| s.kind == timer_core::Kind::Break)
                .unwrap();
            assert!(rest.duration_ms >= 2000 && rest.duration_ms < rest.target_ms);
            assert!(rest.ended_at_ms.is_some());
        });
    }
    #[test]
    fn preferences_preserve_timer_settings_and_active_session() {
        with_runtime(|rt, path| {
            rt.configure(Settings {
                focus_minutes: 33,
                break_minutes: 7,
                notifications: false,
                ..Settings::default()
            })
            .unwrap();
            rt.action("start", "Read").unwrap();
            let session_id = rt.engine.state.current.as_ref().unwrap().id.clone();
            rt.preferences(Some("en-US".into()), None, None).unwrap();
            rt.preferences(None, Some(timer_core::Appearance::Dark), None)
                .unwrap();
            assert!(rt.preferences(Some("invalid".into()), None, None).is_err());
            rt.preferences(None, None, Some("coffee".into())).unwrap();
            let revision = rt.engine.state.revision;
            assert!(rt.preferences(None, None, Some("money".into())).is_err());
            assert_eq!(rt.engine.state.revision, revision);
            let state = &rt.engine.state;
            assert_eq!(state.settings.visual_series, "coffee");
            assert_eq!(state.settings.locale, "en-US");
            assert_eq!(state.settings.appearance, timer_core::Appearance::Dark);
            assert_eq!(state.settings.focus_minutes, 33);
            assert_eq!(state.settings.break_minutes, 7);
            assert!(!state.settings.notifications);
            assert_eq!(state.phase, Phase::Running);
            let current = state.current.as_ref().unwrap();
            assert_eq!(current.id, session_id);
            assert_eq!(current.target_ms, 33 * 60_000);
            let restored = Runtime::open(path).unwrap();
            assert_eq!(restored.engine.state.settings.locale, "en-US");
            assert_eq!(restored.engine.state.settings.visual_series, "coffee");
            assert_eq!(
                restored.engine.state.settings.appearance,
                timer_core::Appearance::Dark
            );
        });
    }
    #[test]
    fn failed_persistence_does_not_advance_shared_state() {
        with_runtime(|rt, _| {
            rt.action("start", "task").unwrap();
            let revision = rt.engine.state.revision;
            // Duplicate completion IDs force a transaction failure after preparing state.
            let mut next = rt.engine.clone();
            let record = next.finish(now_ms()).unwrap();
            rt.repository
                .persist(&rt.engine.state, std::slice::from_ref(&record))
                .unwrap();
            assert!(rt.commit(next, &[record], Instant::now()).is_err());
            assert_eq!(rt.engine.state.phase, Phase::Running);
            assert_eq!(rt.engine.state.revision, revision);
        });
    }
    #[test]
    fn cancellation_persists_idle_without_saving_or_deleting_history() {
        with_runtime(|rt, path| {
            rt.action("start", "completed").unwrap();
            rt.action("switch", "").unwrap();
            let rest_id = rt.engine.state.current.as_ref().unwrap().id.clone();
            rt.action("pause", "").unwrap();
            let revision = rt.engine.state.revision;
            assert!(rt.cancel_session("stale").is_err());
            assert_eq!(rt.engine.state.revision, revision);
            assert_eq!(rt.repository.sessions().unwrap().len(), 1);
            rt.cancel_session(&rest_id).unwrap();
            assert_eq!(rt.repository.sessions().unwrap().len(), 1);
            let restored = Runtime::open(path).unwrap();
            assert_eq!(restored.engine.state.phase, Phase::Idle);
            assert!(restored.engine.state.current.is_none());
            rt.action("start", "discarded focus").unwrap();
            let id = rt.engine.state.current.as_ref().unwrap().id.clone();
            rt.cancel_session(&id).unwrap();
            assert_eq!(rt.repository.sessions().unwrap().len(), 1);
        });
    }
}
