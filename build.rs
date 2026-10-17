//! Build script. On Windows, embeds the application icon and version
//! metadata into the .exe so Explorer, the taskbar, and Alt-Tab show the
//! real icon and the correct application name.
//!
//! Also emits the `has_icon_png` cfg flag when `assets/icon.png` exists,
//! so `app.rs` and `views::components` can conditionally load it without
//! breaking the build on a fresh checkout.

fn main() {
    // build.rs runs on the host, so `target_os` would be the host OS, not
    // the target. CARGO_CFG_TARGET_OS is the correct variable to check.
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target == "windows" {
        embed_windows_icon();
    }

    // Declare the custom cfg so rustc 1.80+ does not warn about it.
    println!("cargo:rustc-check-cfg=cfg(has_icon_png)");

    let png_path = std::path::Path::new("assets/icon.png");
    if png_path.exists() {
        println!("cargo:rustc-cfg=has_icon_png");
        if let Ok(meta) = std::fs::metadata(png_path) {
            println!(
                "cargo:warning=has_icon_png enabled (assets/icon.png, {} bytes)",
                meta.len()
            );
        }
    } else {
        println!(
            "cargo:warning=assets/icon.png not found. The titlebar brand mark \
             and the window icon will be blank. Create a 256x256 PNG at that \
             path."
        );
    }

    // Force a rerun when either input changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon.png");
}

fn embed_windows_icon() {
    let icon = "assets/icon.ico";
    if !std::path::Path::new(icon).exists() {
        println!(
            "cargo:warning=assets/icon.ico not found. The .exe will use the \
             default Rust icon. Convert a PNG with: \
             magick assets/icon.png -define icon:auto-resize=256,128,64,48,32,16 assets/icon.ico"
        );
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon);

    // Set every version-info field explicitly. Leaving any of them unset
    // lets winresource fall back to CARGO_PKG_DESCRIPTION, which is how the
    // old description kept reappearing in Task Manager.
    res.set("ProductName", "Locomotive");
    res.set("FileDescription", "Locomotive");
    res.set("InternalName", "Locomotive");
    res.set("OriginalFilename", "locomotive.exe");
    res.set("CompanyName", "Locomotive");
    res.set("LegalCopyright", "Locomotive contributors");
    res.set("Comments", "");

    match res.compile() {
        Ok(()) => {
            println!("cargo:warning=embedded Windows resources (icon + version info)");
        }
        Err(e) => {
            println!("cargo:warning=failed to embed Windows resources: {e}");
        }
    }
}