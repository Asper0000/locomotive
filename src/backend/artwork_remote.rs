//! Public Steam Store metadata and artwork CDN lookups for numeric App IDs.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::artwork_json::json_string_for_key;
#[cfg(target_os = "windows")]
use super::steam;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CONNECT_TIMEOUT: &str = "2";
const REQUEST_TIMEOUT: &str = "4";

/// Minimal metadata needed to show a title and an image without an API key.
#[derive(Debug, Clone)]
pub(crate) struct AppMetadata {
    /// Human-readable name returned by the Steam Store.
    pub name: Option<String>,
    /// Official landscape header-art URL supplied by the Store API.
    pub header_image: Option<String>,
}

/// Looks up a name and its official Store header image from an App ID.
pub(crate) fn fetch_app_metadata(app_id: &str) -> Option<AppMetadata> {
    let store_url = format!(
        "https://store.steampowered.com/api/appdetails?appids={app_id}&l=english"
    );
    let store_body = fetch_json(&store_url, 1_000_000);
    let mut name = store_body
        .as_deref()
        .and_then(|body| json_string_for_key(body, "name"))
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty() && name.len() < 512);
    let header_image = store_body
        .as_deref()
        .and_then(|body| json_string_for_key(body, "header_image"))
        .filter(|url| is_steam_image_url(url));

    // Some App IDs do not have a useful Store page (for example tools or
    // dedicated-server entries). SteamCMD's public app-info service provides
    // a second way to resolve the display name for those IDs.
    if name.is_none() {
        let fallback_url = format!("https://api.steamcmd.net/v1/info/{app_id}");
        name = fetch_json(&fallback_url, 2_000_000)
            .and_then(|body| json_string_for_key(&body, "name"))
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty() && name.len() < 512);
    }

    (name.is_some() || header_image.is_some()).then_some(AppMetadata { name, header_image })
}

fn fetch_json(url: &str, maximum_size: usize) -> Option<String> {
    let output = run_curl([
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--connect-timeout",
        CONNECT_TIMEOUT,
        "--max-time",
        REQUEST_TIMEOUT,
        "--header",
        "User-Agent: Locomotive/0.0.1",
        url,
    ])?;
    if output.stdout.len() > maximum_size {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// Downloads official Store artwork first, with library-capsule fallbacks for portrait art.
pub(crate) fn download_cover(
    cache_dir: &Path,
    app_id: &str,
    header_image: Option<&str>,
) -> Option<PathBuf> {
    for extension in ["jpg", "png", "jpeg"] {
        let cached = cache_dir.join(format!("{app_id}.cover.{extension}"));
        if is_image_file(&cached, extension) {
            return Some(cached);
        }
    }
    for extension in ["jpg", "png", "jpeg"] {
        let cached_header = cache_dir.join(format!("{app_id}.cover-header.{extension}"));
        if is_image_file(&cached_header, extension) {
            return Some(cached_header);
        }
    }

    // Prefer the exact image URL returned by the Store API. This is the most
    // reliable current URL for an App ID; probing several guessed portrait CDN
    // URLs first made name-and-art updates appear to stall for unavailable IDs.
    if let Some(url) = header_image.filter(|url| is_steam_image_url(url)) {
        let extension = image_extension(url).unwrap_or("jpg");
        if let Some(path) = download_image(cache_dir, app_id, url, extension, true) {
            return Some(path);
        }
    }

    // If the API image is unavailable, try the current shared Steam static
    // library capsule endpoint, then a small set of legacy-compatible variants.
    let candidates = [
        (format!("https://shared.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900_2x.jpg"), "jpg"),
        (format!("https://shared.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900.jpg"), "jpg"),
        (format!("https://cdn.akamai.steamstatic.com/steam/apps/{app_id}/library_600x900.jpg"), "jpg"),
        (format!("https://shared.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900.png"), "png"),
    ];

    for (url, extension) in candidates {
        if let Some(path) = download_image(cache_dir, app_id, &url, extension, false) {
            return Some(path);
        }
    }

    for url in [
        format!("https://shared.steamstatic.com/store_item_assets/steam/apps/{app_id}/header.jpg"),
        format!("https://cdn.akamai.steamstatic.com/steam/apps/{app_id}/header.jpg"),
    ] {
        if let Some(path) = download_image(cache_dir, app_id, &url, "jpg", true) {
            return Some(path);
        }
    }
    None
}

fn download_image(
    cache_dir: &Path,
    app_id: &str,
    url: &str,
    extension: &str,
    is_header: bool,
) -> Option<PathBuf> {
    let filename = if is_header {
        format!("{app_id}.cover-header.{extension}")
    } else {
        format!("{app_id}.cover.{extension}")
    };
    let destination = cache_dir.join(filename);
    if is_image_file(&destination, extension) {
        return Some(destination);
    }
    let _ = fs::remove_file(&destination);

    let temporary = cache_dir.join(format!("{app_id}.cover.part"));
    let _ = fs::remove_file(&temporary);
    let temporary_path = temporary.to_string_lossy().into_owned();
    let result = run_curl([
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--connect-timeout",
        CONNECT_TIMEOUT,
        "--max-time",
        REQUEST_TIMEOUT,
        "--output",
        &temporary_path,
        url,
    ]);

    if result.is_none() || !is_image_file(&temporary, extension) {
        let _ = fs::remove_file(temporary);
        return None;
    }
    if fs::rename(&temporary, &destination).is_ok() {
        Some(destination)
    } else {
        let _ = fs::remove_file(temporary);
        is_image_file(&destination, extension).then_some(destination)
    }
}

pub(crate) fn is_valid_cached_image(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    is_image_file(path, &extension.to_ascii_lowercase())
}

fn is_image_file(path: &Path, extension: &str) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.len() < 512 {
        return false;
    }
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut signature = [0; 8];
    let Ok(length) = file.read(&mut signature) else {
        return false;
    };
    match extension {
        "png" => length == 8 && signature == [137, 80, 78, 71, 13, 10, 26, 10],
        "jpg" | "jpeg" => {
            length >= 3 && signature[0] == 255 && signature[1] == 216 && signature[2] == 255
        }
        _ => false,
    }
}

fn image_extension(url: &str) -> Option<&'static str> {
    let path = url.split(|character| character == '?' || character == '#').next()?;
    let extension = path.rsplit('.').next()?.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("png"),
        "jpeg" => Some("jpeg"),
        "jpg" => Some("jpg"),
        _ => None,
    }
}

fn is_steam_image_url(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if scheme != "https" {
        return false;
    }

    let host = rest.split('/').next().unwrap_or_default();
    host == "steamstatic.com"
        || host.ends_with(".steamstatic.com")
        || host == "steampowered.com"
        || host.ends_with(".steampowered.com")
}

fn run_curl<const N: usize>(arguments: [&str; N]) -> Option<Output> {
    #[cfg(target_os = "windows")]
    let mut command = Command::new("curl.exe");
    #[cfg(not(target_os = "windows"))]
    let mut command = Command::new("curl");
    command.args(arguments);
    #[cfg(target_os = "windows")]
    command.creation_flags(steam::CREATE_NO_WINDOW);
    command.output().ok().filter(|output| output.status.success())
}
