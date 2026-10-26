CREATE TABLE checkpoint (id INTEGER PRIMARY KEY CHECK (id = 1), state TEXT NOT NULL);
CREATE TABLE sessions (id TEXT PRIMARY KEY, started_at_ms INTEGER NOT NULL, data TEXT NOT NULL);
CREATE INDEX sessions_started ON sessions(started_at_ms DESC);
PRAGMA user_version = 1;
