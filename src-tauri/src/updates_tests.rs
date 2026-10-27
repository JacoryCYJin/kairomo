//! Entirely local fixture: throwaway public key, a signed fake app, and no production data.
use base64::Engine as _;
use std::io::{Read, Write};
use tauri_plugin_updater::UpdaterExt;
use timer_core::{Phase, Repository};

#[test]
fn signed_update_checks_downloads_installs_and_retains_data_in_an_isolated_app() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("update_fixture.json")).unwrap();
    let archive = base64::engine::general_purpose::STANDARD
        .decode(fixture["archive"].as_str().unwrap())
        .unwrap();
    let folder = tempfile::tempdir().unwrap();
    let executable = folder.path().join("Fixture.app/Contents/MacOS/fixture");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    std::fs::write(&executable, b"old-beta").unwrap();
    let database = folder.path().join("beta-data.sqlite3");
    let mut runtime = crate::runtime::Runtime::open(&database).unwrap();
    runtime.action("start", "Retained focus").unwrap();
    runtime.action("switch", "").unwrap();
    runtime.shutdown().unwrap();
    let current_id = runtime.engine.state.current.as_ref().unwrap().id.clone();
    runtime
        .repository
        .backup_to(&folder.path().join("backup.sqlite3"))
        .unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let manifest = serde_json::json!({
        "version":"9.0.0-beta.1", "notes":"Local signed update fixture",
        "platforms":{"fixture-target":{"url":format!("http://{address}/package"),"signature":fixture["signature"]}}
    }).to_string();
    let served_archive = archive.clone();
    let server = std::thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = [0u8; 4096];
            let count = stream.read(&mut request).unwrap();
            let is_manifest =
                String::from_utf8_lossy(&request[..count]).starts_with("GET /manifest ");
            let body = if is_manifest {
                manifest.as_bytes()
            } else {
                &served_archive
            };
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            stream.write_all(body).unwrap();
        }
    });
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert(
        "updater".into(),
        serde_json::json!({
            "pubkey":fixture["public_key"], "dangerousInsecureTransportProtocol":true, "requireSignedVersion":true
        }),
    );
    let app = tauri::test::mock_builder()
        .plugin(
            tauri_plugin_updater::Builder::new()
                .pubkey(fixture["public_key"].as_str().unwrap())
                .build(),
        )
        .build(context)
        .unwrap();
    let updater = app
        .updater_builder()
        .target("fixture-target")
        .executable_path(&executable)
        .endpoints(vec![format!("http://{address}/manifest").parse().unwrap()])
        .unwrap()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let update = tauri::async_runtime::block_on(updater.check())
        .unwrap()
        .unwrap();
    let downloaded = tauri::async_runtime::block_on(update.download(|_, _| {}, || {})).unwrap();
    server.join().unwrap();
    assert_eq!(downloaded, archive);
    let mut tampered = downloaded.clone();
    tampered[0] ^= 1;
    assert!(update.install(&tampered).is_err());
    assert_eq!(std::fs::read(&executable).unwrap(), b"old-beta");
    update.install(&downloaded).unwrap();
    assert_eq!(std::fs::read(&executable).unwrap(), b"new-beta");
    let restored = crate::runtime::Runtime::open(&database).unwrap();
    assert_eq!(restored.engine.state.phase, Phase::Paused);
    assert_eq!(restored.engine.state.current.unwrap().id, current_id);
    assert_eq!(restored.repository.sessions().unwrap().len(), 1);
    let backup = crate::runtime::Runtime::open(&folder.path().join("backup.sqlite3")).unwrap();
    assert_eq!(
        backup.repository.sessions().unwrap()[0].title,
        "Retained focus"
    );
    let dev = crate::runtime::Runtime::open(&folder.path().join("dev-data.sqlite3")).unwrap();
    assert!(dev.repository.sessions().unwrap().is_empty());
}

#[test]
fn dev_and_beta_configuration_keep_separate_identities_and_the_legacy_beta_data_identity() {
    let dev: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let beta: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.beta.conf.json")).unwrap();
    assert_eq!(dev["identifier"], "com.jacorycyjin.kairomo.dev");
    assert_eq!(beta["identifier"], "com.jacorycyjin.kairomo");
    assert_ne!(dev["productName"], beta["productName"]);
    assert_eq!(beta["plugins"]["updater"]["requireSignedVersion"], true);
    assert!(beta["plugins"]["updater"]
        .get("dangerousInsecureTransportProtocol")
        .is_none());
}
