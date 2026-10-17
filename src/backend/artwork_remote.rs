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
const REQUEST_TIMEOUT: &str = "6";
const IMAGE_TIMEOUT: &str = "3";

/// Minimal metadata needed to show a title and an image without an API key.
#[derive(Debug, Clone)]
pub(crate) struct AppMetadata {
    /// Human-readable name returned by the Steam Store.
    pub name: Option<String>,
    /// Official landscape header-art URL supplied by the Store API.
    pub header_image: Option<String>,
    /// Store capsule image URLs used if library artwork is unavailable.
    pub capsule_image: Option<String>,
    /// Older Store capsule image URL exposed alongside `capsule_image`.
    pub capsule_image_v5: Option<String>,
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
    let capsule_image = store_body
        .as_deref()
        .and_then(|body| json_string_for_key(body, "capsule_image"))
        .filter(|url| is_steam_image_url(url));
    let capsule_image_v5 = store_body
        .as_deref()
        .and_then(|body| json_string_for_key(body, "capsule_imagev5"))
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

    (name.is_some() || header_image.is_some() || capsule_image.is_some() || capsule_image_v5.is_some())
        .then_some(AppMetadata {
            name,
            header_image,
            capsule_image,
            capsule_image_v5,
        })
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

/// Downloads portrait library artwork first, then uses other Store artwork as a fallback.
pub(crate) fn download_cover(
    cache_dir: &Path,
    app_id: &str,
    metadata: Option<&AppMetadata>,
) -> Option<PathBuf> {
    // A previous build may have cached a wide header banner. Do not let it stop
    // us from trying the actual portrait library capsule.
    for extension in ["jpg", "png", "jpeg"] {
        let cached = cache_dir.join(format!("{app_id}.cover.{extension}"));
        if is_image_file(&cached, extension) {
            return Some(cached);
        }
    }

    let header_image = metadata.and_then(|metadata| metadata.header_image.as_deref());

    // Newer Steam images can live in app-ID/hash/asset.jpg paths. The Store API
    // gives us the exact header URL, so try portrait files beside that image.
    if let Some(url) = header_image.filter(|url| is_steam_image_url(url)) {
        for filename in ["library_600x900_2x.jpg", "library_600x900.jpg"] {
            if let Some(candidate) = sibling_asset_url(url, filename) {
                if let Some(path) = download_image(cache_dir, app_id, &candidate, "jpg", false) {
                    return Some(path);
                }
            }
        }
    }

    // Try a small set of current and legacy AppID-only paths. Keeping this list
    // short avoids a long silent wait when an App ID has no portrait artwork.
    let candidates = [
        format!("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900_2x.jpg"),
        format!("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{app_id}/library_600x900.jpg"),
        format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/library_600x900.jpg"),
        format!("https://steamcdn-a.akamaihd.net/steam/apps/{app_id}/library_600x900.jpg"),
    ];

    for url in candidates {
        if let Some(path) = download_image(cache_dir, app_id, &url, "jpg", false) {
            return Some(path);
        }
    }

    // A portrait is not available for every App ID. Try exact Store image URLs
    // next so even games without library capsules can still show official art.
    for url in [
        header_image,
        metadata.and_then(|metadata| metadata.capsule_image.as_deref()),
        metadata.and_then(|metadata| metadata.capsule_image_v5.as_deref()),
    ]
    .into_iter()
    .flatten()
    .filter(|url| is_steam_image_url(url))
    {
        let extension = image_extension(url).unwrap_or("jpg");
        if let Some(path) = download_image(cache_dir, app_id, url, extension, true) {
            return Some(path);
        }
    }

    // Last resort for App IDs whose Store response lacks artwork metadata.
    for url in [
        format!("https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/{app_id}/header.jpg"),
        format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/header.jpg"),
    ] {
        if let Some(path) = download_image(cache_dir, app_id, &url, "jpg", true) {
            return Some(path);
        }
    }

    // If all remote requests fail but an old header is cached, keep displaying
    // it rather than replacing artwork with an empty tile.
    for extension in ["jpg", "png", "jpeg"] {
        let cached_header = cache_dir.join(format!("{app_id}.cover-header.{extension}"));
        if is_image_file(&cached_header, extension) {
            return Some(cached_header);
        }
    }
    None
}

fn sibling_asset_url(image_url: &str, filename: &str) -> Option<String> {
    let clean_url = image_url.split(|character| character == '?' || character == '#').next()?;
    let (directory, _) = clean_url.rsplit_once('/')?;
    Some(format!("{directory}/{filename}"))
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
        IMAGE_TIMEOUT,
        "--header",
        "User-Agent: Locomotive/0.0.1",
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


#[cfg(test)]
mod tests {
    use super::{image_extension, is_steam_image_url, sibling_asset_url};

    #[test]
    fn portrait_lookup_reuses_hashed_store_asset_directory() {
        let url = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/12345/abc987/header.jpg?t=123";
        assert_eq!(
            sibling_asset_url(url, "library_600x900.jpg").as_deref(),
            Some("https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/12345/abc987/library_600x900.jpg"),
        );
    }

    #[test]
    fn steam_image_urls_can_include_cache_busters() {
        let url = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/12345/header.jpg?t=123";
        assert!(is_steam_image_url(url));
        assert_eq!(image_extension(url), Some("jpg"));
    }
}
