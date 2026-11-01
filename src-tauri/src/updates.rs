use crate::runtime::AppState;
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::Duration;
use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

const BETA_MANIFEST: &str =
    "https://github.com/JacoryCYJin/kairomo/releases/download/beta-channel/latest.json";
const ASSET_PREFIX: &str = "https://github.com/JacoryCYJin/kairomo/releases/download/";

// Reqwest discovery runs before the updater plugin's own TLS initialization.
// Keep this idempotent: another caller may already have installed a provider.
pub(crate) fn initialize_tls() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn release_client() -> Result<reqwest::Client, String> {
    initialize_tls();
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Kairomo-Beta-Updater")
        .build()
        .map_err(|_| "update_request_failed".to_owned())
}

fn request_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "update_check_timeout"
    } else {
        "update_request_failed"
    }
    .into()
}

fn channel_status(status: reqwest::StatusCode) -> Result<bool, String> {
    match status.as_u16() {
        200 => Ok(true),
        204 | 404 => Ok(false),
        403 | 429 => Err("update_service_busy".into()),
        _ => Err("update_request_failed".into()),
    }
}

fn validate_beta_manifest(manifest: &Value) -> Result<Version, String> {
    let invalid = || "update_invalid_release".to_owned();
    let version =
        Version::parse(manifest["version"].as_str().ok_or_else(invalid)?).map_err(|_| invalid())?;
    let pre: Vec<_> = version.pre.as_str().split('.').collect();
    if pre.len() != 2
        || pre[0] != "beta"
        || pre[1].parse::<u64>().is_err()
        || !version.build.is_empty()
    {
        return Err(invalid());
    }
    let platforms = manifest["platforms"]
        .as_object()
        .filter(|p| !p.is_empty())
        .ok_or_else(invalid)?;
    let prefix = format!("{ASSET_PREFIX}v{version}/");
    for platform in platforms.values() {
        let url = platform["url"].as_str().ok_or_else(invalid)?;
        let name = url.strip_prefix(&prefix).ok_or_else(invalid)?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            || platform["signature"]
                .as_str()
                .is_none_or(|s| s.trim().is_empty())
        {
            return Err(invalid());
        }
    }
    Ok(version)
}

async fn fetch_beta_manifest(endpoint: &str) -> Result<Option<Value>, String> {
    let response = release_client()?
        .get(endpoint)
        .send()
        .await
        .map_err(request_error)?;
    if !channel_status(response.status())? {
        return Ok(None);
    }
    let manifest: Value = response.json().await.map_err(|error| {
        if error.is_timeout() {
            "update_check_timeout"
        } else if error.is_decode() {
            "update_invalid_release"
        } else {
            "update_request_failed"
        }
    })?;
    validate_beta_manifest(&manifest)?;
    Ok(Some(manifest))
}

fn updater_error(error: tauri_plugin_updater::Error, fallback: &str) -> String {
    use tauri_plugin_updater::Error;
    match error {
        Error::Minisign(_)
        | Error::Base64(_)
        | Error::SignatureUtf8(_)
        | Error::SignedVersionMismatch { .. }
        | Error::MissingSignedVersion => "update_signature_failed".into(),
        Error::TargetNotFound(_) | Error::TargetsNotFound(_) => {
            "update_platform_unavailable".into()
        }
        Error::Serialization(_) | Error::Semver(_) => "update_invalid_release".into(),
        _ => fallback.into(),
    }
}

#[derive(Default)]
pub struct Updates {
    busy: AtomicBool,
    pending: Mutex<Option<(String, Update)>>,
}
struct Operation<'a>(&'a AtomicBool);
impl Drop for Operation<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
impl Updates {
    fn begin(&self) -> Result<Operation<'_>, String> {
        self.busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "update_busy".to_owned())?;
        Ok(Operation(&self.busy))
    }
}
#[derive(Serialize)]
pub struct ReleaseInfo {
    name: String,
    version: String,
    channel: &'static str,
}
#[tauri::command]
pub fn release_info(app: tauri::AppHandle) -> ReleaseInfo {
    ReleaseInfo {
        name: app
            .config()
            .product_name
            .clone()
            .unwrap_or_else(|| "Kairomo".into()),
        version: app.package_info().version.to_string(),
        channel: if app.config().identifier.ends_with(".dev") {
            "dev"
        } else {
            "beta"
        },
    }
}
#[derive(Serialize)]
pub struct CheckResult {
    status: &'static str,
    version: Option<String>,
    notes: Option<String>,
    token: Option<String>,
}
fn status(value: &'static str) -> CheckResult {
    CheckResult {
        status: value,
        version: None,
        notes: None,
        token: None,
    }
}
#[tauri::command]
pub async fn check_update(
    app: tauri::AppHandle,
    state: State<'_, Updates>,
) -> Result<CheckResult, String> {
    if app.config().identifier.ends_with(".dev") {
        return Ok(status("development"));
    }
    let _operation = state.begin()?;
    *state.pending.lock().map_err(|_| "update_busy")? = None;
    let Some(manifest) = fetch_beta_manifest(BETA_MANIFEST).await? else {
        return Ok(status("unpublished"));
    };
    let version = validate_beta_manifest(&manifest)?;
    if version <= app.package_info().version {
        return Ok(status("current"));
    }
    let update = app
        .updater_builder()
        .endpoints(vec![BETA_MANIFEST
            .parse()
            .map_err(|_| "update_invalid_release")?])
        .map_err(|_| "update_invalid_release")?
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| "update_invalid_release")?
        .restore_update(manifest)
        .map_err(|error| updater_error(error, "update_invalid_release"))?;
    let Some(update) = update else {
        return Ok(status("current"));
    };
    if update.version != version.to_string() || update.download_url.scheme() != "https" {
        return Err("update_invalid_release".into());
    }
    let token = uuid::Uuid::new_v4().to_string();
    let result = CheckResult {
        status: "available",
        version: Some(update.version.clone()),
        notes: update.body.clone(),
        token: Some(token.clone()),
    };
    *state.pending.lock().map_err(|_| "update_busy")? = Some((token, update));
    Ok(result)
}
#[derive(Clone, Serialize)]
struct Progress {
    stage: &'static str,
    downloaded: u64,
    total: Option<u64>,
}
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    state: State<'_, Updates>,
    token: String,
) -> Result<(), String> {
    if app.config().identifier.ends_with(".dev") {
        return Err("update_development".into());
    }
    let _operation = state.begin()?;
    let update = {
        let mut pending = state.pending.lock().map_err(|_| "update_busy")?;
        if !pending
            .as_ref()
            .is_some_and(|(expected, _)| expected == &token)
        {
            return Err("update_stale_confirmation".into());
        }
        pending.take().ok_or("update_stale_confirmation")?.1
    };
    let mut downloaded = 0;
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                let _ = app.emit(
                    "update:progress",
                    Progress {
                        stage: "downloading",
                        downloaded,
                        total,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|error| updater_error(error, "update_download_failed"))?;
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|_| "update_backup_failed")?
        .join("backups");
    std::fs::create_dir_all(&directory).map_err(|_| "update_backup_failed")?;
    let runtime = app.state::<AppState>();
    {
        // Hold the runtime lock through replacement so no new timer operation can
        // race the final checkpoint. Download and signature verification happen first.
        let mut rt = runtime.0.lock().map_err(|_| "update_persist_failed")?;
        rt.shutdown().map_err(|_| "update_persist_failed")?;
        let _ = app.emit("timer:changed", &rt.engine.state);
        rt.repository
            .backup_to(&directory.join(format!("before-update-{}.sqlite3", uuid::Uuid::new_v4())))
            .map_err(|_| "update_backup_failed")?;
        let _ = app.emit(
            "update:progress",
            Progress {
                stage: "installing",
                downloaded,
                total: Some(downloaded),
            },
        );
        update
            .install(&bytes)
            .map_err(|error| updater_error(error, "update_install_failed"))?;
    }
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_client_initializes_tls_in_a_fresh_process() {
        const CHILD: &str = "KAIROMO_UPDATE_TLS_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            assert!(rustls::crypto::CryptoProvider::get_default().is_none());
            tauri::async_runtime::block_on(async {
                release_client().expect("first HTTPS client must not panic");
                // Repeated initialization must remain safe.
                release_client().expect("subsequent HTTPS client must also build");
            });
            return;
        }
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "updates::tests::release_client_initializes_tls_in_a_fresh_process",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "fresh-process TLS check failed: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[test]
    fn channel_status_distinguishes_unpublished_busy_and_failed() {
        assert!(channel_status(reqwest::StatusCode::OK).unwrap());
        for code in [204, 404] {
            assert!(!channel_status(reqwest::StatusCode::from_u16(code).unwrap()).unwrap());
        }
        for code in [403, 429] {
            assert_eq!(
                channel_status(reqwest::StatusCode::from_u16(code).unwrap()).unwrap_err(),
                "update_service_busy"
            );
        }
        assert_eq!(
            channel_status(reqwest::StatusCode::INTERNAL_SERVER_ERROR).unwrap_err(),
            "update_request_failed"
        );
    }
    #[test]
    fn fixed_channel_requires_beta_and_version_bound_release_assets() {
        let valid = serde_json::json!({"version":"0.1.0-beta.7", "platforms":{"darwin-aarch64":{
            "url":format!("{ASSET_PREFIX}v0.1.0-beta.7/Kairomo.app.tar.gz"), "signature":"signed"}}});
        assert_eq!(
            validate_beta_manifest(&valid).unwrap().to_string(),
            "0.1.0-beta.7"
        );
        for version in [
            "0.1.0",
            "0.1.0-alpha.1",
            "0.1.0-beta",
            "0.1.0-beta.x",
            "0.1.0-beta.7.extra",
        ] {
            let mut fixture = valid.clone();
            fixture["version"] = version.into();
            assert!(validate_beta_manifest(&fixture).is_err());
        }
        for url in [
            "http://example.com/app",
            "https://example.com/app",
            "https://github.com/JacoryCYJin/kairomo/releases/download/v0.1.0-beta.6/app",
            "https://github.com/JacoryCYJin/kairomo/releases/download/v0.1.0-beta.7/../app",
        ] {
            let mut fixture = valid.clone();
            fixture["platforms"]["darwin-aarch64"]["url"] = url.into();
            assert!(validate_beta_manifest(&fixture).is_err());
        }
        let mut fixture = valid.clone();
        fixture["platforms"]["darwin-aarch64"]["signature"] = "".into();
        assert!(validate_beta_manifest(&fixture).is_err());
        assert!(validate_beta_manifest(
            &serde_json::json!({"version":"0.1.0-beta.7", "platforms":{}})
        )
        .is_err());
    }
    #[test]
    fn updater_signature_failures_keep_their_distinct_error() {
        assert_eq!(
            updater_error(
                tauri_plugin_updater::Error::MissingSignedVersion,
                "update_download_failed"
            ),
            "update_signature_failed"
        );
        assert_eq!(
            updater_error(
                tauri_plugin_updater::Error::TargetNotFound("windows-x86_64".into()),
                "update_invalid_release"
            ),
            "update_platform_unavailable"
        );
    }
    #[test]
    fn update_operations_are_serialized_and_guard_releases_after_error() {
        let state = Updates::default();
        let operation = state.begin().unwrap();
        assert!(state.begin().is_err());
        drop(operation);
        assert!(state.begin().is_ok());
    }
}

#[cfg(all(test, target_os = "macos"))]
#[path = "updates_tests.rs"]
mod integration_tests;
