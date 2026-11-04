use crate::runtime::{now_ms, Runtime, ShutdownOutcome};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const PROTOCOL: u8 = 1;
const REQUEST_TTL_MS: i64 = 2 * 60 * 1000;
const TOKEN_TTL_MS: i64 = 10 * 60 * 1000;
const REQUEST_FILE: &str = "install-handoff-request.json";
const TOKEN_FILE: &str = "install-handoff-token.json";
const RESULT_FILE: &str = "install-handoff-result.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandoffRequest {
    pub protocol: u8,
    pub nonce: String,
    pub target_version: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandoffToken {
    pub protocol: u8,
    pub nonce: String,
    pub target_version: String,
    pub session_id: String,
    pub checkpoint_hash: String,
    pub revision: u64,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandoffResult {
    pub protocol: u8,
    pub nonce: String,
    pub status: String,
    pub reason: Option<String>,
}

fn request_path(data_dir: &Path) -> PathBuf {
    data_dir.join(REQUEST_FILE)
}
fn token_path(data_dir: &Path) -> PathBuf {
    data_dir.join(TOKEN_FILE)
}
fn result_path(data_dir: &Path) -> PathBuf {
    data_dir.join(RESULT_FILE)
}

pub fn launch_nonce() -> Option<String> {
    parse_launch_nonce(std::env::args())
}

fn parse_launch_nonce(args: impl IntoIterator<Item = String>) -> Option<String> {
    args.into_iter().find_map(|arg| {
        arg.strip_prefix("--install-handoff=")
            .filter(|nonce| !nonce.is_empty())
            .map(str::to_owned)
    })
}

pub fn load_request(data_dir: &Path) -> Option<HandoffRequest> {
    let path = request_path(data_dir);
    let raw = fs::read_to_string(&path).ok()?;
    let Ok(request) = serde_json::from_str::<HandoffRequest>(&raw) else {
        let _ = fs::remove_file(path);
        return None;
    };
    let now = now_ms();
    if request.protocol != PROTOCOL
        || request.nonce.is_empty()
        || request.target_version.is_empty()
        || request.created_at_ms > now + 5_000
        || now.saturating_sub(request.created_at_ms) > REQUEST_TTL_MS
    {
        let _ = fs::remove_file(path);
        return None;
    }
    Some(request)
}

pub fn finish_exit(
    data_dir: &Path,
    request: Option<HandoffRequest>,
    outcome: &ShutdownOutcome,
) -> Result<(), String> {
    let Some(request) = request else {
        cleanup_stale(data_dir);
        return Ok(());
    };

    let resumable = outcome.was_running
        && outcome.phase == timer_core::Phase::Paused
        && outcome.pause_reason.as_deref() == Some("quit")
        && outcome.session_id.is_some()
        && outcome.checkpoint_hash.is_some();
    if resumable {
        let token = HandoffToken {
            protocol: PROTOCOL,
            nonce: request.nonce.clone(),
            target_version: request.target_version.clone(),
            session_id: outcome.session_id.clone().expect("checked session"),
            checkpoint_hash: outcome.checkpoint_hash.clone().expect("checked hash"),
            revision: outcome.revision,
            created_at_ms: now_ms(),
        };
        write_json(&token_path(data_dir), &token)?;
        let _ = fs::remove_file(request_path(data_dir));
        let _ = fs::remove_file(result_path(data_dir));
        return Ok(());
    }

    let reason = if !outcome.was_running {
        "not_running"
    } else if outcome.phase != timer_core::Phase::Paused {
        "not_paused"
    } else if outcome.pause_reason.as_deref() != Some("quit") {
        "not_quit_pause"
    } else {
        "missing_checkpoint"
    };
    let result = HandoffResult {
        protocol: PROTOCOL,
        nonce: request.nonce,
        status: "not_resumable".into(),
        reason: Some(reason.into()),
    };
    write_json(&result_path(data_dir), &result)?;
    let _ = fs::remove_file(request_path(data_dir));
    let _ = fs::remove_file(token_path(data_dir));
    Ok(())
}

pub fn consume_startup(
    data_dir: &Path,
    runtime: &mut Runtime,
    nonce: &str,
) -> Result<HandoffResult, String> {
    let path = token_path(data_dir);
    let raw = fs::read_to_string(&path);
    let result = match raw
        .ok()
        .and_then(|raw| serde_json::from_str::<HandoffToken>(&raw).ok())
    {
        None => result(nonce, "not_resumed", "missing_or_invalid_token"),
        Some(token) => {
            let now = now_ms();
            let reason = if token.protocol != PROTOCOL {
                Some("protocol_mismatch")
            } else if token.nonce != nonce {
                Some("nonce_mismatch")
            } else if token.target_version != env!("CARGO_PKG_VERSION") {
                Some("version_mismatch")
            } else if token.created_at_ms > now + 5_000
                || now.saturating_sub(token.created_at_ms) > TOKEN_TTL_MS
            {
                Some("token_expired")
            } else if runtime.engine.state.phase != timer_core::Phase::Paused {
                Some("not_paused")
            } else if runtime.engine.state.pause_reason.as_deref() != Some("quit") {
                Some("not_quit_pause")
            } else if runtime
                .engine
                .state
                .current
                .as_ref()
                .map(|session| session.id.as_str())
                != Some(token.session_id.as_str())
            {
                Some("session_mismatch")
            } else if runtime.startup_checkpoint_hash() != Some(token.checkpoint_hash.as_str()) {
                Some("checkpoint_mismatch")
            } else if runtime.engine.state.revision != token.revision {
                Some("revision_mismatch")
            } else {
                None
            };
            if let Some(reason) = reason {
                result(nonce, "not_resumed", reason)
            } else {
                match runtime.resume_after_replace() {
                    Ok(Some(_)) => result(nonce, "resumed", ""),
                    Ok(None) => result(nonce, "not_resumed", "not_resumable"),
                    Err(error) => result(nonce, "not_resumed", &error),
                }
            }
        }
    };
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(request_path(data_dir));
    write_json(&result_path(data_dir), &result)?;
    Ok(result)
}

fn result(nonce: &str, status: &str, reason: &str) -> HandoffResult {
    HandoffResult {
        protocol: PROTOCOL,
        nonce: nonce.into(),
        status: status.into(),
        reason: (!reason.is_empty()).then(|| reason.into()),
    }
}

pub fn discard_all(data_dir: &Path) {
    let _ = fs::remove_file(request_path(data_dir));
    let _ = fs::remove_file(token_path(data_dir));
    let _ = fs::remove_file(result_path(data_dir));
}

fn cleanup_stale(data_dir: &Path) {
    cleanup_older_than(&request_path(data_dir), REQUEST_TTL_MS);
    cleanup_older_than(&token_path(data_dir), TOKEN_TTL_MS);
    cleanup_older_than(&result_path(data_dir), TOKEN_TTL_MS);
}

fn cleanup_older_than(path: &Path, ttl_ms: i64) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return;
    };
    if age.as_millis() as i64 > ttl_ms {
        let _ = fs::remove_file(path);
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "handoff_path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("handoff"),
        std::process::id()
    ));
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;
    use timer_core::Phase;

    fn request(nonce: &str) -> HandoffRequest {
        HandoffRequest {
            protocol: PROTOCOL,
            nonce: nonce.into(),
            target_version: env!("CARGO_PKG_VERSION").into(),
            created_at_ms: now_ms(),
        }
    }

    fn quit_runtime(dir: &Path) -> (Runtime, String) {
        let db = dir.join("handoff.sqlite3");
        let mut runtime = Runtime::open(&db).unwrap();
        runtime.action("start", "Read").unwrap();
        let id = runtime.engine.state.current.as_ref().unwrap().id.clone();
        (runtime, id)
    }

    #[test]
    fn launch_nonce_requires_a_non_empty_value() {
        assert_eq!(
            parse_launch_nonce(["app".into(), "--install-handoff=abc".into()]),
            Some("abc".into())
        );
        assert_eq!(
            parse_launch_nonce(["app".into(), "--install-handoff=".into()]),
            None
        );
        assert_eq!(parse_launch_nonce(["app".into()]), None);
    }

    #[test]
    fn matching_token_resumes_once_and_is_consumed() {
        let dir = tempfile::tempdir().unwrap();
        let (mut runtime, id) = quit_runtime(dir.path());
        let outcome = runtime.shutdown_for_handoff().unwrap();
        let paused_duration = runtime.engine.state.current.as_ref().unwrap().duration_ms;
        finish_exit(dir.path(), Some(request("nonce")), &outcome).unwrap();
        assert!(token_path(dir.path()).exists());
        assert!(!request_path(dir.path()).exists());
        // A second ExitRequested must not delete a token written by the first one.
        finish_exit(dir.path(), None, &outcome).unwrap();
        assert!(token_path(dir.path()).exists());

        let db = dir.path().join("handoff.sqlite3");
        let mut reopened = Runtime::open(&db).unwrap();
        let result = consume_startup(dir.path(), &mut reopened, "nonce").unwrap();
        assert_eq!(result.status, "resumed");
        assert_eq!(reopened.engine.state.phase, Phase::Running);
        assert_eq!(reopened.engine.state.current.as_ref().unwrap().id, id);
        assert_eq!(
            reopened.engine.state.current.as_ref().unwrap().duration_ms,
            paused_duration
        );
        assert!(!token_path(dir.path()).exists());

        let replay = consume_startup(dir.path(), &mut reopened, "nonce").unwrap();
        assert_eq!(replay.status, "not_resumed");
        assert_eq!(replay.reason.as_deref(), Some("missing_or_invalid_token"));
    }

    #[test]
    fn old_quit_checkpoint_without_token_is_not_resumed() {
        let dir = tempfile::tempdir().unwrap();
        let (mut runtime, _) = quit_runtime(dir.path());
        runtime.shutdown_for_handoff().unwrap();

        let db = dir.path().join("handoff.sqlite3");
        let mut reopened = Runtime::open(&db).unwrap();
        let result = consume_startup(dir.path(), &mut reopened, "nonce").unwrap();
        assert_eq!(result.status, "not_resumed");
        assert_eq!(result.reason.as_deref(), Some("missing_or_invalid_token"));
        assert_eq!(reopened.engine.state.phase, Phase::Paused);
        assert_eq!(reopened.engine.state.pause_reason.as_deref(), Some("quit"));
    }

    #[test]
    fn paused_session_writes_non_resumable_result_not_token() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("handoff.sqlite3");
        let mut runtime = Runtime::open(&db).unwrap();
        runtime.action("start", "Read").unwrap();
        runtime.action("pause", "").unwrap();
        let outcome = runtime.shutdown_for_handoff().unwrap();
        assert!(!outcome.was_running);
        finish_exit(dir.path(), Some(request("nonce")), &outcome).unwrap();
        assert!(!token_path(dir.path()).exists());
        let raw = fs::read_to_string(result_path(dir.path())).unwrap();
        let result: HandoffResult = serde_json::from_str(&raw).unwrap();
        assert_eq!(result.status, "not_resumable");
        assert_eq!(result.reason.as_deref(), Some("not_running"));
    }

    #[test]
    fn mismatched_token_does_not_resume_and_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let (mut runtime, _) = quit_runtime(dir.path());
        let outcome = runtime.shutdown_for_handoff().unwrap();
        finish_exit(dir.path(), Some(request("nonce")), &outcome).unwrap();
        let mut token: HandoffToken =
            serde_json::from_str(&fs::read_to_string(token_path(dir.path())).unwrap()).unwrap();
        token.nonce = "other".into();
        write_json(&token_path(dir.path()), &token).unwrap();

        let db = dir.path().join("handoff.sqlite3");
        let mut reopened = Runtime::open(&db).unwrap();
        let result = consume_startup(dir.path(), &mut reopened, "nonce").unwrap();
        assert_eq!(result.status, "not_resumed");
        assert_eq!(result.reason.as_deref(), Some("nonce_mismatch"));
        assert_eq!(reopened.engine.state.phase, Phase::Paused);
        assert!(!token_path(dir.path()).exists());
    }

    #[test]
    fn expired_request_is_removed_and_normal_exit_keeps_fresh_handoff_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut expired = request("nonce");
        expired.created_at_ms = now_ms() - REQUEST_TTL_MS - 1;
        write_json(&request_path(dir.path()), &expired).unwrap();
        assert!(load_request(dir.path()).is_none());
        assert!(!request_path(dir.path()).exists());

        write_json(&token_path(dir.path()), &"fresh").unwrap();
        write_json(&result_path(dir.path()), &"fresh").unwrap();
        let db = dir.path().join("handoff.sqlite3");
        let mut runtime = Runtime::open(&db).unwrap();
        let outcome = runtime.shutdown_for_handoff().unwrap();
        finish_exit(dir.path(), None, &outcome).unwrap();
        assert!(!request_path(dir.path()).exists());
        assert!(token_path(dir.path()).exists());
        assert!(result_path(dir.path()).exists());

        discard_all(dir.path());
        assert!(!token_path(dir.path()).exists());
        assert!(!result_path(dir.path()).exists());
    }
}
