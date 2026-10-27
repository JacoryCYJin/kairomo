fn main() {
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
