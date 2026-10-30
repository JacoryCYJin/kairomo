//! SQLite adapter. All durable state belongs in the OS app-data directory.
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use timer_core::{Kind, Repository, Session, Snapshot};

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("data: {0}")]
    Data(#[from] serde_json::Error),
    #[error("unsupported_database_version")]
    UnsupportedVersion,
    #[error("record_not_found")]
    NotFound,
}

pub struct SqliteRepository {
    connection: Connection,
}
impl SqliteRepository {
    pub fn backup_to(&self, destination: &Path) -> Result<(), StorageError> {
        self.connection.backup("main", destination, None)?;
        Ok(())
    }
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        Self::from_connection(Connection::open(path)?)
    }
    fn from_connection(connection: Connection) -> Result<Self, StorageError> {
        connection.busy_timeout(std::time::Duration::from_secs(3))?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )?;
        let version: u32 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match version {
            0 => connection.execute_batch(concat!(
                "BEGIN IMMEDIATE;",
                include_str!("../migrations/001_initial.sql"),
                "COMMIT;"
            ))?,
            1 => (),
            _ => return Err(StorageError::UnsupportedVersion),
        }
        Ok(Self { connection })
    }
}
impl Repository for SqliteRepository {
    type Error = StorageError;
    fn load_checkpoint(&self) -> Result<Option<Snapshot>, Self::Error> {
        let value: Option<String> = self
            .connection
            .query_row("SELECT state FROM checkpoint WHERE id=1", [], |r| r.get(0))
            .optional()?;
        value
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(Into::into)
    }
    fn persist(&mut self, state: &Snapshot, completed: &[Session]) -> Result<(), Self::Error> {
        let tx = self.connection.transaction()?;
        for s in completed.iter().filter(|s| s.is_recordable_focus()) {
            tx.execute(
                "INSERT INTO sessions(id, started_at_ms, data) VALUES(?1, ?2, ?3)",
                params![s.id, s.started_at_ms, serde_json::to_string(s)?],
            )?;
        }
        tx.execute("INSERT INTO checkpoint(id,state) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET state=excluded.state", [serde_json::to_string(state)?])?;
        tx.commit()?;
        Ok(())
    }
    fn sessions(&self) -> Result<Vec<Session>, Self::Error> {
        let mut query = self
            .connection
            .prepare("SELECT data FROM sessions ORDER BY started_at_ms DESC, id DESC")?;
        let rows = query.query_map([], |r| r.get::<_, String>(0))?;
        let records: Result<Vec<Session>, Self::Error> =
            rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect();
        records.map(|records| {
            records
                .into_iter()
                .filter(|s| s.kind == Kind::Focus && s.ended_at_ms.is_some())
                .collect()
        })
    }
    fn delete_session(&mut self, id: &str) -> Result<(), Self::Error> {
        if self
            .connection
            .execute("DELETE FROM sessions WHERE id=?1", [id])?
            == 0
        {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }
    fn clear_sessions(&mut self) -> Result<(), Self::Error> {
        self.connection.execute("DELETE FROM sessions", [])?;
        Ok(())
    }
    fn update_session(&mut self, s: &Session) -> Result<(), Self::Error> {
        if self.connection.execute(
            "UPDATE sessions SET data=?1 WHERE id=?2",
            params![serde_json::to_string(s)?, s.id],
        )? == 0
        {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use timer_core::Engine;
    fn db() -> SqliteRepository {
        SqliteRepository::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    #[test]
    fn clear_history_preserves_checkpoint_and_is_durable_and_repeatable() {
        let path = std::env::temp_dir().join(format!(
            "kairomo-clear-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut repo = SqliteRepository::open(&path).unwrap();
        let mut engine = Engine::default();
        engine.start("saved".into(), "Read".into(), 1000).unwrap();
        engine.advance(60_000, 61_000);
        let record = engine.finish(61_000).unwrap();
        engine
            .start("current".into(), "Write".into(), 62_000)
            .unwrap();
        engine.state.settings.focus_minutes = 35;
        repo.persist(&engine.state, &[record]).unwrap();
        let checkpoint = serde_json::to_value(&engine.state).unwrap();
        repo.clear_sessions().unwrap();
        repo.clear_sessions().unwrap();
        assert!(repo.sessions().unwrap().is_empty());
        drop(repo);
        let mut reopened = SqliteRepository::open(&path).unwrap();
        assert!(reopened.sessions().unwrap().is_empty());
        assert_eq!(
            serde_json::to_value(reopened.load_checkpoint().unwrap().unwrap()).unwrap(),
            checkpoint
        );
        // A later checkpoint save must not resurrect deleted history.
        reopened.persist(&engine.state, &[]).unwrap();
        assert!(reopened.sessions().unwrap().is_empty());
        drop(reopened);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn failed_clear_is_atomic_and_preserves_checkpoint() {
        let mut repo = db();
        let mut engine = Engine::default();
        let mut records = Vec::new();
        for id in ["one", "two"] {
            engine.start(id.into(), "Read".into(), 1000).unwrap();
            engine.advance(60_000, 61_000);
            records.push(engine.finish(61_000).unwrap());
        }
        repo.persist(&engine.state, &records).unwrap();
        let checkpoint = serde_json::to_value(repo.load_checkpoint().unwrap()).unwrap();
        repo.connection.execute_batch(
            "CREATE TRIGGER reject_clear BEFORE DELETE ON sessions WHEN OLD.id='two' BEGIN SELECT RAISE(ABORT, 'simulated failure'); END;"
        ).unwrap();
        assert!(repo.clear_sessions().is_err());
        assert_eq!(repo.sessions().unwrap().len(), 2);
        assert_eq!(
            serde_json::to_value(repo.load_checkpoint().unwrap()).unwrap(),
            checkpoint
        );
    }
    #[test]
    fn rest_and_unfinished_sessions_never_enter_history_but_rest_checkpoint_recovers() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("focus".into(), "Read".into(), 1000).unwrap();
        engine.advance(60_000, 61_000);
        let focus = engine.switch("rest".into(), 61_000).unwrap();
        engine.resume(61_000).unwrap();
        engine.advance(10_000, 71_000);
        engine.pause(71_000, "manual").unwrap();
        let checkpoint = engine.state.clone();
        let ongoing_rest = checkpoint.current.as_ref().unwrap().clone();
        let rest = engine.finish(71_000).unwrap();
        let mut unfinished_focus = focus.clone();
        unfinished_focus.id = "unfinished".into();
        unfinished_focus.ended_at_ms = None;
        repo.persist(&checkpoint, &[focus, rest, unfinished_focus, ongoing_rest])
            .unwrap();
        let raw_count: u32 = repo
            .connection
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(raw_count, 1);
        let loaded = repo.load_checkpoint().unwrap().unwrap();
        assert_eq!(loaded.phase, timer_core::Phase::Paused);
        let current = loaded.current.unwrap();
        assert_eq!(current.kind, Kind::Break);
        assert_eq!(current.duration_ms, 10_000);
        assert_eq!(repo.sessions().unwrap()[0].duration_ms, 60_000);
    }
    #[test]
    fn legacy_rest_rows_are_excluded_without_deleting_existing_data() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("focus".into(), "Read".into(), 1000).unwrap();
        engine.advance(60_000, 61_000);
        let focus = engine.switch("rest".into(), 61_000).unwrap();
        engine.resume(61_500).unwrap();
        let rest = engine.finish(62_000).unwrap();
        repo.connection
            .execute(
                "INSERT INTO sessions(id,started_at_ms,data) VALUES(?1,?2,?3)",
                params![
                    rest.id,
                    rest.started_at_ms,
                    serde_json::to_string(&rest).unwrap()
                ],
            )
            .unwrap();
        repo.persist(&engine.state, &[focus]).unwrap();
        assert_eq!(repo.sessions().unwrap().len(), 1);
        let raw_count: u32 = repo
            .connection
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(raw_count, 2);
    }
    #[test]
    fn never_started_focus_is_checkpoint_only_even_if_submitted_as_completed() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("first".into(), "Read".into(), 1000).unwrap();
        engine.advance(60_000, 61_000);
        let first = engine.switch("rest".into(), 61_000).unwrap();
        engine.resume(61_500).unwrap();
        engine.switch("prepared".into(), 62_000).unwrap();
        let waiting = engine.state.clone();
        // Simulate malformed legacy input without bypassing the engine's guard.
        let mut never_started = waiting.current.as_ref().unwrap().clone();
        never_started.ended_at_ms = Some(100_000);
        repo.persist(&waiting, &[first, never_started]).unwrap();
        assert_eq!(repo.sessions().unwrap().len(), 1);
        let checkpoint = repo.load_checkpoint().unwrap().unwrap();
        assert_eq!(checkpoint.pause_reason.as_deref(), Some("ready"));
        assert!(checkpoint.current.unwrap().intervals.is_empty());
    }
    #[test]
    fn records_and_checkpoint_are_atomic_and_roundtrip() {
        let mut repo = db();
        let mut e = Engine::default();
        e.start("one".into(), "阅读,\"Rust\"\nnotes".into(), 1000)
            .unwrap();
        e.advance(60_000, 61_000);
        let record = e.finish(61_000).unwrap();
        repo.persist(&e.state, std::slice::from_ref(&record))
            .unwrap();
        assert_eq!(repo.sessions().unwrap()[0].title, record.title);
        let mut new = Engine::default();
        new.start("next".into(), "next".into(), 62_000).unwrap();
        // A duplicate completed ID rolls back the checkpoint in the same transaction.
        assert!(repo.persist(&new.state, &[record]).is_err());
        assert!(repo.load_checkpoint().unwrap().unwrap().current.is_none());
    }
    #[test]
    fn legacy_checkpoint_loads_with_system_appearance_and_keeps_records() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("legacy".into(), "阅读".into(), 1000).unwrap();
        engine.advance(30_000, 31_000);
        let record = engine.finish(31_000).unwrap();
        repo.persist(&engine.state, &[]).unwrap();
        repo.connection
            .execute(
                "INSERT INTO sessions(id,started_at_ms,data) VALUES(?1,?2,?3)",
                params![
                    record.id,
                    record.started_at_ms,
                    serde_json::to_string(&record).unwrap()
                ],
            )
            .unwrap();
        let mut legacy = serde_json::to_value(&engine.state).unwrap();
        legacy.as_object_mut().unwrap().remove("bean_random_seed");
        legacy.as_object_mut().unwrap().remove("phrase_selection");
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("phrases");
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("visual_series");
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("appearance");
        repo.connection
            .execute(
                "UPDATE checkpoint SET state=?1 WHERE id=1",
                [legacy.to_string()],
            )
            .unwrap();
        let state = repo.load_checkpoint().unwrap().unwrap();
        assert_eq!(state.settings.appearance, timer_core::Appearance::System);
        assert_eq!(state.settings.visual_series, "coffee");
        assert_eq!(state.settings.phrases, timer_core::Phrases::default());
        assert!(state.phrase_selection.is_none());
        assert!(state.bean_random_seed.is_none());
        assert_eq!(repo.sessions().unwrap()[0].id, record.id);
    }
    #[test]
    fn language_and_all_appearance_modes_survive_reopen() {
        let mut repo = db();
        for locale in ["zh-CN", "zh-TW", "en-US", "ja-JP"] {
            for appearance in [
                timer_core::Appearance::System,
                timer_core::Appearance::Light,
                timer_core::Appearance::Dark,
            ] {
                let mut state = Snapshot::default();
                state.settings.locale = locale.into();
                state.settings.appearance = appearance.clone();
                repo.persist(&state, &[]).unwrap();
                repo = SqliteRepository::from_connection(repo.connection).unwrap();
                let loaded = repo.load_checkpoint().unwrap().unwrap();
                assert_eq!(loaded.settings.locale, locale);
                assert_eq!(loaded.settings.appearance, appearance);
            }
        }
    }
    #[test]
    fn new_records_require_one_minute_and_pauses_do_not_count() {
        let mut repo = db();
        for duration in [59_999, 60_000, 60_001] {
            let mut engine = Engine::default();
            engine
                .start(duration.to_string(), "Boundary".into(), 1000)
                .unwrap();
            engine.advance(duration, 1000 + duration as i64);
            let done = engine.finish(1000 + duration as i64).unwrap();
            repo.persist(&engine.state, &[done]).unwrap();
        }
        let records = repo.sessions().unwrap();
        assert_eq!(records.len(), 2);
        assert!(records.iter().any(|s| s.duration_ms == 60_000));
        assert!(records.iter().any(|s| s.duration_ms == 60_001));
        let mut engine = Engine::default();
        engine
            .start("paused".into(), "Paused".into(), 1000)
            .unwrap();
        engine.advance(30_000, 31_000);
        engine.pause(31_000, "manual").unwrap();
        engine.advance(600_000, 631_000);
        repo.persist(&engine.state, &[]).unwrap();
        assert_eq!(
            repo.load_checkpoint()
                .unwrap()
                .unwrap()
                .current
                .unwrap()
                .duration_ms,
            30_000
        );
        engine.resume(631_000).unwrap();
        engine.advance(30_000, 661_000);
        let done = engine.finish(661_000).unwrap();
        repo.persist(&engine.state, &[done]).unwrap();
        let mut saved = repo
            .sessions()
            .unwrap()
            .into_iter()
            .find(|s| s.id == "paused")
            .unwrap();
        assert_eq!(saved.duration_ms, 60_000);
        assert_eq!(saved.intervals.len(), 2);
        saved.duration_ms = 30_000;
        saved.corrected = true;
        repo.update_session(&saved).unwrap();
        assert_eq!(
            repo.sessions()
                .unwrap()
                .into_iter()
                .find(|s| s.id == "paused")
                .unwrap()
                .duration_ms,
            30_000
        );
    }
    #[test]
    fn migration_reopens_without_losing_data() {
        let conn = Connection::open_in_memory().unwrap();
        let repo = SqliteRepository::from_connection(conn).unwrap();
        let mut repo = SqliteRepository::from_connection(repo.connection).unwrap();
        repo.persist(&Snapshot::default(), &[]).unwrap();
        assert!(repo.load_checkpoint().unwrap().is_some());
        assert!(repo
            .update_session(&Session {
                id: "missing".into(),
                kind: timer_core::Kind::Focus,
                title: String::new(),
                started_at_ms: 0,
                ended_at_ms: Some(1),
                target_ms: 1,
                focus_mode: timer_core::FocusMode::Manual,
                duration_ms: 1,
                intervals: vec![],
                corrected: false
            })
            .is_err());
        assert!(repo.delete_session("missing").is_err());
    }
}
