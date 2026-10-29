fn main() {
    let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../theme-kits");
    println!("cargo:rerun-if-changed={}", root.display());
    let catalog: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("catalog.json")).unwrap()).unwrap();
    let default = catalog["default"].as_str().unwrap();
    assert!(catalog["themes"].get(default).is_some());
    let mut generated = String::from("pub fn rest_icon(id: &str) -> &'static [u8] { match id {\n");
    for (id, theme) in catalog["themes"].as_object().unwrap() {
        let relative = std::path::Path::new(theme["menu"]["restIcon"].as_str().unwrap());
        assert!(relative
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_))));
        let icon = root
            .join(relative)
            .canonicalize()
            .expect("missing theme menu icon");
        generated.push_str(&format!(
            "{id:?} => include_bytes!({:?}),\n",
            icon.to_str().unwrap()
        ));
    }
    generated.push_str(&format!("_ => rest_icon({default:?}), }} }}\n"));
    std::fs::write(
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("theme_icons.rs"),
        generated,
    )
    .unwrap();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/platform/macos/sleep.m");
        println!("cargo:rerun-if-changed=src/platform/macos/popup.m");
        println!("cargo:rerun-if-changed=src/platform/macos/appearance.m");
        println!("cargo:rerun-if-changed=src/platform/macos/tray.m");
        println!("cargo:rerun-if-changed=src/platform/macos/titlebar.m");
        cc::Build::new()
            .file("src/platform/macos/sleep.m")
            .file("src/platform/macos/popup.m")
            .file("src/platform/macos/appearance.m")
            .file("src/platform/macos/tray.m")
            .file("src/platform/macos/titlebar.m")
            .flag("-fobjc-arc")
            .flag("-fblocks")
            .compile("kairomo_sleep");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreText");
    }
    tauri_build::build();
}
