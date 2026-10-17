//! Cached Steam metadata and artwork enrichment for app-ID-based library entries.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::state::library_state::LibrarySnapshot;

use super::{archive, artwork_remote, steam};

const MAX_PARALLEL_LOOKUPS: usize = 8;

#[derive(Clone)]
struct Lookup {
    entry_index: usize,
    app_id: String,
    needs_name: bool,
    needs_cover: bool,
    cover_path: Option<PathBuf>,
}

/// Attaches local Steam artwork and cached names without network access.
pub(crate) fn enrich_local(snapshot: &mut LibrarySnapshot) {
    let _ = prepare_lookups(snapshot);
    sort_by_name(snapshot);
}

/// Fills remaining names and covers through cached, background-friendly Steam lookups.
pub(crate) fn enrich_remote(snapshot: &mut LibrarySnapshot) {
    let Some((cache_dir, steam_root, lookups)) = prepare_lookups(snapshot) else {
        return;
    };

    for batch in lookups.chunks(MAX_PARALLEL_LOOKUPS) {
        let resolved = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .cloned()
                .map(|lookup| {
                    let cache_dir = &cache_dir;
                    let steam_root = steam_root.as_deref();
                    scope.spawn(move || resolve_lookup(lookup, cache_dir, steam_root))
                })
                .collect();
            handles
                .into_iter()
                .filter_map(|handle| handle.join().ok())
                .collect::<Vec<_>>()
        });

        for (entry_index, name, cover_path) in resolved {
            let Some(entry) = snapshot.entries.get_mut(entry_index) else {
                continue;
            };
            if let Some(name) = name {
                entry.name = name;
                entry.name_is_fallback = false;
            }
            if cover_path.is_some() {
                entry.cover_path = cover_path;
            }
        }
    }

    sort_by_name(snapshot);
}

fn prepare_lookups(
    snapshot: &mut LibrarySnapshot,
) -> Option<(PathBuf, Option<PathBuf>, Vec<Lookup>)> {
    if snapshot.entries.is_empty() {
        return None;
    }

    let cache_dir = archive::cache_directory();
    fs::create_dir_all(&cache_dir).ok()?;
    let steam_root = steam::discover_root();
    let mut lookups = Vec::new();

    for (entry_index, entry) in snapshot.entries.iter_mut().enumerate() {
        let Some(app_id) = entry.key.strip_prefix("app:") else {
            continue;
        };
        if app_id.is_empty() || !app_id.chars().all(|character| character.is_ascii_digit()) {
            continue;
        }

        if entry.name_is_fallback || entry.name == format!("App {app_id}") {
            if let Some(name) = read_cached_name(&cache_dir, app_id) {
                entry.name = name;
                entry.name_is_fallback = false;
            }
        }

        let local_cover = find_cover(steam_root.as_deref(), app_id);
        let cached_portrait = cached_portrait(&cache_dir, app_id);
        let cached_fallback = cached_header(&cache_dir, app_id);
        let cover_path = cached_portrait
            .or_else(|| local_cover.clone().filter(|path| !is_header_art(path)))
            .or(local_cover)
            .or(cached_fallback);
        let needs_name = entry.name_is_fallback || entry.name == format!("App {app_id}");
        // A cached wide banner is only a fallback, not a finished cover lookup:
        // keep trying for the portrait capsule on subsequent launches.
        let needs_cover = cover_path.as_deref().is_none_or(is_header_art);
        entry.cover_path = cover_path.clone();
        if needs_name || needs_cover {
            lookups.push(Lookup {
                entry_index,
                app_id: app_id.to_owned(),
                needs_name,
                needs_cover,
                cover_path,
            });
        }
    }

    Some((cache_dir, steam_root, lookups))
}

fn resolve_lookup(
    lookup: Lookup,
    cache_dir: &Path,
    steam_root: Option<&Path>,
) -> (usize, Option<String>, Option<PathBuf>) {
    let fetch_name = lookup.needs_name
        && !recent_failure(cache_dir, &lookup.app_id, "name", Duration::from_secs(6 * 60 * 60));
    let fetch_cover = lookup.needs_cover
        && !recent_failure(cache_dir, &lookup.app_id, "cover", Duration::from_secs(24 * 60 * 60));
    let metadata = (fetch_name || fetch_cover)
        .then(|| artwork_remote::fetch_app_metadata(&lookup.app_id))
        .flatten();

    let name = if lookup.needs_name {
        let resolved = metadata.as_ref().and_then(|metadata| metadata.name.clone());
        let resolved = resolved.or_else(|| read_cached_name(cache_dir, &lookup.app_id));
        let marker = failure_marker(cache_dir, &lookup.app_id, "name");
        if let Some(name) = &resolved {
            let _ = fs::write(cache_dir.join(format!("{}.name", lookup.app_id)), name);
            let _ = fs::remove_file(marker);
        } else if fetch_name {
            let _ = fs::write(marker, b"");
        }
        resolved
    } else {
        None
    };

    let downloaded_cover = if fetch_cover {
        artwork_remote::download_cover(cache_dir, &lookup.app_id, metadata.as_ref())
    } else {
        None
    };

    if fetch_cover {
        let marker = failure_marker(cache_dir, &lookup.app_id, "cover");
        if downloaded_cover.is_some() {
            let _ = fs::remove_file(marker);
        } else {
            let _ = fs::write(marker, b"");
        }
    }

    // Prefer a newly downloaded portrait over an older cached header banner.
    let cover_path = downloaded_cover
        .or(lookup.cover_path)
        .or_else(|| cached_cover(cache_dir, &lookup.app_id))
        .or_else(|| find_cover(steam_root, &lookup.app_id));
    (lookup.entry_index, name, cover_path)
}

fn recent_failure(cache_dir: &Path, app_id: &str, kind: &str, retry_after: Duration) -> bool {
    let Ok(metadata) = fs::metadata(failure_marker(cache_dir, app_id, kind)) else {
        return false;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    modified.elapsed().is_ok_and(|elapsed| elapsed < retry_after)
}

fn failure_marker(cache_dir: &Path, app_id: &str, kind: &str) -> PathBuf {
    // Versioned markers bypass misses from earlier CDN strategies. v4 retries
    // App IDs whose older lookups could not find portrait or hashed CDN assets.
    cache_dir.join(format!("{app_id}.{kind}-v4-miss"))
}

fn read_cached_name(cache_dir: &Path, app_id: &str) -> Option<String> {
    let name = fs::read_to_string(cache_dir.join(format!("{app_id}.name"))).ok()?;
    let name = name.trim();
    (!name.is_empty() && name.len() < 512).then(|| name.to_owned())
}

fn cached_portrait(cache_dir: &Path, app_id: &str) -> Option<PathBuf> {
    [
        cache_dir.join(format!("{app_id}.cover.jpg")),
        cache_dir.join(format!("{app_id}.cover.png")),
        cache_dir.join(format!("{app_id}.cover.jpeg")),
        cache_dir.join(format!("{app_id}.jpg")),
    ]
    .into_iter()
    .find(|path| artwork_remote::is_valid_cached_image(path))
}

fn cached_header(cache_dir: &Path, app_id: &str) -> Option<PathBuf> {
    [
        cache_dir.join(format!("{app_id}.cover-header.jpg")),
        cache_dir.join(format!("{app_id}.cover-header.png")),
        cache_dir.join(format!("{app_id}.cover-header.jpeg")),
    ]
    .into_iter()
    .find(|path| artwork_remote::is_valid_cached_image(path))
}

fn cached_cover(cache_dir: &Path, app_id: &str) -> Option<PathBuf> {
    cached_portrait(cache_dir, app_id).or_else(|| cached_header(cache_dir, app_id))
}

fn is_header_art(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().contains("header"))
}

fn find_cover(steam_root: Option<&Path>, app_id: &str) -> Option<PathBuf> {
    let root = steam_root?;
    let directories = [
        root.join("appcache").join("librarycache"),
        root.join("steamapps").join("librarycache"),
    ];
    let suffixes = [
        "_library_600x900_2x.jpg", "_library_600x900.jpg", "_library_600x900.png",
        "_library_600x900.jpeg", "_header.jpg", "_header.png",
    ];

    for directory in &directories {
        for suffix in suffixes {
            let path = directory.join(format!("{app_id}{suffix}"));
            if artwork_remote::is_valid_cached_image(&path) {
                return Some(path);
            }
        }
        let app_cache = directory.join(app_id);
        if let Some(path) = search_artwork_directory(&app_cache, 4) {
            return Some(path);
        }
    }
    None
}

fn search_artwork_directory(directory: &Path, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let entries = fs::read_dir(directory).ok()?.flatten().map(|entry| entry.path()).collect::<Vec<_>>();
    let preferred = [
        "library_600x900_2x.jpg", "library_600x900.jpg", "library_600x900.png",
        "library_600x900.jpeg", "library_capsule.jpg", "library_capsule.png",
        "library_header.jpg", "header.jpg", "header.png",
    ];
    for filename in preferred {
        if let Some(path) = entries.iter().find(|path| {
            path.file_name().and_then(|name| name.to_str()) == Some(filename)
                && artwork_remote::is_valid_cached_image(path)
        }) {
            return Some(path.clone());
        }
    }
    for path in entries {
        if path.is_dir() {
            if let Some(found) = search_artwork_directory(&path, depth - 1) {
                return Some(found);
            }
        }
    }
    None
}

fn sort_by_name(snapshot: &mut LibrarySnapshot) {
    snapshot.entries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.key.cmp(&right.key))
    });
}

