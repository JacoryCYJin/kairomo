//! SQLite adapter. All durable state belongs in the OS app-data directory.
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use timer_core::{Repository, Session, Snapshot};

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
        for s in completed {
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
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
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
                duration_ms: 1,
                intervals: vec![],
                corrected: false
            })
            .is_err());
        assert!(repo.delete_session("missing").is_err());
    }
}
