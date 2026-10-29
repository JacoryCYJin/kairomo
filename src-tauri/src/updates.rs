use crate::runtime::AppState;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::Duration;
use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

const RELEASES: &str = "https://api.github.com/repos/JacoryCYJin/kairomo/releases?per_page=100";
const ASSET_PREFIX: &str = "https://github.com/JacoryCYJin/kairomo/releases/download/";

// Reqwest discovery runs before the updater plugin's own TLS initialization.
// Keep this idempotent: another caller may already have installed a provider.
pub(crate) fn initialize_tls() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn release_client() -> Result<reqwest::Client, String> {
    initialize_tls();
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Kairomo-Beta-Updater")
        .build()
        .map_err(|_| "update_request_failed".to_owned())
}

async fn discover_beta_manifest() -> Result<Option<(Version, String)>, String> {
    let response = release_client()?
        .get(RELEASES)
        .send()
        .await
        .map_err(|_| "update_request_failed")?
        .error_for_status()
        .map_err(|_| "update_request_failed")?;
    let releases: Vec<Release> = response
        .json()
        .await
        .map_err(|_| "update_invalid_release")?;
    Ok(beta_manifest(&releases))
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
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
fn beta_manifest(releases: &[Release]) -> Option<(Version, String)> {
    releases
        .iter()
        .filter(|r| !r.draft && r.prerelease)
        .filter_map(|r| {
            let version = Version::parse(r.tag_name.trim_start_matches('v')).ok()?;
            if version.pre.as_str().split('.').next() != Some("beta") {
                return None;
            }
            let asset = r.assets.iter().find(|a| {
                a.name == "latest.json" && a.browser_download_url.starts_with(ASSET_PREFIX)
            })?;
            Some((version, asset.browser_download_url.clone()))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
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
    let Some((version, endpoint)) = discover_beta_manifest().await? else {
        return Ok(status("unpublished"));
    };
    if version <= app.package_info().version {
        return Ok(status("current"));
    }
    let update = app
        .updater_builder()
        .endpoints(vec![endpoint
            .parse()
            .map_err(|_| "update_invalid_release")?])
        .map_err(|_| "update_invalid_release")?
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| "update_invalid_release")?
        .check()
        .await
        .map_err(|_| "update_request_failed")?;
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
        .map_err(|_| "update_download_failed")?;
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
            .map_err(|_| "update_install_failed")?;
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
    fn beta_discovery_ignores_drafts_stable_alpha_and_unusable_assets_and_sorts_semver() {
        let releases = serde_json::from_value::<Vec<Release>>(serde_json::json!([
            {"tag_name":"v1.0.0", "draft":false,"prerelease":false,"assets":[]},
            {"tag_name":"v0.1.0-beta.2", "draft":false,"prerelease":true,"assets":[{"name":"latest.json","browser_download_url":format!("{ASSET_PREFIX}v0.1.0-beta.2/latest.json")}]},
            {"tag_name":"v0.1.0-beta.10", "draft":false,"prerelease":true,"assets":[{"name":"latest.json","browser_download_url":format!("{ASSET_PREFIX}v0.1.0-beta.10/latest.json")}]},
            {"tag_name":"v9.0.0-beta.1", "draft":true,"prerelease":true,"assets":[]},
            {"tag_name":"v9.0.0-alpha.1", "draft":false,"prerelease":true,"assets":[]},
            {"tag_name":"v8.0.0-beta.1", "draft":false,"prerelease":true,"assets":[{"name":"latest.json","browser_download_url":"https://example.com/latest.json"}]}
        ])).unwrap();
        assert_eq!(
            beta_manifest(&releases).unwrap().0.to_string(),
            "0.1.0-beta.10"
        );
        assert!(beta_manifest(&[]).is_none());
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
