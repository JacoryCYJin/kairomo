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
        for s in completed
            .iter()
            .filter(|s| s.kind == Kind::Focus && s.ended_at_ms.is_some() && !s.intervals.is_empty())
        {
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
    fn rest_and_unfinished_sessions_never_enter_history_but_rest_checkpoint_recovers() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("focus".into(), "Read".into(), 1000).unwrap();
        engine.advance(30_000, 31_000);
        let focus = engine.switch("rest".into(), 31_000).unwrap();
        engine.resume(31_000).unwrap();
        engine.advance(10_000, 41_000);
        engine.pause(41_000, "manual").unwrap();
        let checkpoint = engine.state.clone();
        let ongoing_rest = checkpoint.current.as_ref().unwrap().clone();
        let rest = engine.finish(41_000).unwrap();
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
        assert_eq!(repo.sessions().unwrap()[0].duration_ms, 30_000);
    }
    #[test]
    fn legacy_rest_rows_are_excluded_without_deleting_existing_data() {
        let mut repo = db();
        let mut engine = Engine::default();
        engine.start("focus".into(), "Read".into(), 1000).unwrap();
        let focus = engine.switch("rest".into(), 2000).unwrap();
        engine.resume(2500).unwrap();
        let rest = engine.finish(3000).unwrap();
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
        let first = engine.switch("rest".into(), 2000).unwrap();
        engine.resume(2500).unwrap();
        engine.switch("prepared".into(), 3000).unwrap();
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
        e.advance(42_000, 43_000);
        let record = e.finish(43_000).unwrap();
        repo.persist(&e.state, std::slice::from_ref(&record))
            .unwrap();
        assert_eq!(repo.sessions().unwrap()[0].title, record.title);
        let mut new = Engine::default();
        new.start("next".into(), "next".into(), 44_000).unwrap();
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
        repo.persist(&engine.state, std::slice::from_ref(&record))
            .unwrap();
        let mut legacy = serde_json::to_value(&engine.state).unwrap();
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
        assert_eq!(repo.sessions().unwrap()[0].id, record.id);
    }
    #[test]
    fn language_and_all_appearance_modes_survive_reopen() {
        let mut repo = db();
        for appearance in [
            timer_core::Appearance::System,
            timer_core::Appearance::Light,
            timer_core::Appearance::Dark,
        ] {
            let mut state = Snapshot::default();
            state.settings.locale = "en-US".into();
            state.settings.appearance = appearance.clone();
            repo.persist(&state, &[]).unwrap();
            repo = SqliteRepository::from_connection(repo.connection).unwrap();
            let loaded = repo.load_checkpoint().unwrap().unwrap();
            assert_eq!(loaded.settings.locale, "en-US");
            assert_eq!(loaded.settings.appearance, appearance);
        }
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
