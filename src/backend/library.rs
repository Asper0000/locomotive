//! Reconstructs the local archive into grouped entries on each scan.
//!
//! Grouping works in two passes:
//!
//!   1. Every `.lua` file is read and parsed for its app id and depot list.
//!      A `.lua` file named `2651280.lua` is an entry for app 2651280, and
//!      the depots it declares form a depot-to-app map.
//!   2. Every `.manifest` file is matched to an app through that map. A
//!      manifest that no Lua references still becomes an entry of its own,
//!      keyed by depot id.
//!
//! Steam metadata, when present, only improves display names. Grouping does
//! not depend on it, so uninstalled games still group correctly.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::state::{FileKind, library_state::{LibraryEntry, LibraryFile, LibrarySnapshot}};

use super::artwork;
use super::lua_meta;
use super::steam::{self, AppInfo, SteamMetadata};

#[derive(Debug)]
struct EntryBuilder {
    key: String,
    name: String,
    name_is_fallback: bool,
    files: Vec<LibraryFile>,
}

/// Scans configured folders and uses read-only Steam metadata when available to improve names.
pub(crate) fn scan(manifest_dir: &Path, lua_dir: &Path) -> LibrarySnapshot {
    let metadata = steam::discover_root().and_then(|root| steam::read_metadata(&root).ok());
    scan_with_metadata(manifest_dir, lua_dir, metadata.as_ref())
}

/// Rebuilds library entries and enriches unknown app IDs with cached Steam names and covers.
pub(crate) fn refresh(manifest_dir: &Path, lua_dir: &Path) -> LibrarySnapshot {
    let mut snapshot = scan(manifest_dir, lua_dir);
    artwork::enrich_local(&mut snapshot);
    snapshot
}

fn scan_with_metadata(
    manifest_dir: &Path,
    lua_dir: &Path,
    metadata: Option<&SteamMetadata>,
) -> LibrarySnapshot {
    let empty_apps = HashMap::new();
    let empty_depot_owners = HashMap::new();
    let apps: &HashMap<String, AppInfo> = metadata.map(|layout| &layout.apps).unwrap_or(&empty_apps);
    let depot_owners: &HashMap<String, String> = metadata
        .map(|layout| &layout.depot_owners)
        .unwrap_or(&empty_depot_owners);

    let mut groups: HashMap<String, EntryBuilder> = HashMap::new();
    let mut depot_to_app: HashMap<String, String> = HashMap::new();

    // Pass 1: read every Lua file, determine its app id, and register
    // the depots it declares. The app id comes from the Lua contents
    // first, from the filename stem second, and from Steam metadata only
    // as a display-name source.
    for (path, _) in collect_files(lua_dir, FileKind::Lua) {
        let meta = lua_meta::read(&path).unwrap_or_default();
        let stem = file_stem(&path);
        let app_id = numeric_stem(&stem);
        let key = match &app_id {
            Some(id) => format!("app:{id}"),
            None => format!("name:{}", normalized_key(&stem)),
        };
        let metadata_name = app_id
            .as_ref()
            .and_then(|id| apps.get(id))
            .map(|app| app.name.clone());
        let name_is_fallback = metadata_name.is_none();
        let name = metadata_name.unwrap_or_else(|| display_stem(&stem));
        let entry = groups.entry(key.clone()).or_insert_with(|| EntryBuilder {
            key: key.clone(),
            name,
            name_is_fallback,
            files: Vec::new(),
        });
        entry.files.push(LibraryFile { path, kind: FileKind::Lua });

        if let Some(id) = &app_id {
            for depot in &meta.depots {
                depot_to_app.insert(depot.clone(), id.clone());
            }
        }
    }

    // Pass 2: read every manifest, resolve its depot to an app, and add it
    // to the matching group. Falls back to Steam's depot ownership map, then
    // to a standalone entry keyed by depot id.
    for (path, kind) in collect_files(manifest_dir, FileKind::Manifest) {
        let stem = file_stem(&path);
        let depot_id = manifest_depot_id(&stem);
        let app_id = depot_id
            .as_ref()
            .and_then(|id| depot_to_app.get(id).cloned())
            .or_else(|| depot_id.as_ref().and_then(|id| depot_owners.get(id).cloned()));
        let key = match &app_id {
            Some(id) => format!("app:{id}"),
            None => match &depot_id {
                Some(id) => format!("depot:{id}"),
                None => format!("name:{}", normalized_key(&group_stem(&stem))),
            },
        };
        let metadata_name = app_id
            .as_ref()
            .and_then(|id| apps.get(id))
            .map(|app| app.name.clone());
        let has_existing_group = groups.contains_key(&key);
        let fallback_name = metadata_name
            .clone()
            .or_else(|| groups.get(&key).map(|entry| entry.name.clone()))
            .unwrap_or_else(|| match &depot_id {
                Some(id) => format!("Depot {id}"),
                None => display_stem(&group_stem(&stem)),
            });
        let entry = groups.entry(key.clone()).or_insert_with(|| EntryBuilder {
            key,
            name: fallback_name,
            name_is_fallback: metadata_name.is_none() && !has_existing_group,
            files: Vec::new(),
        });
        entry.files.push(LibraryFile { path, kind });
    }

    let mut entries: Vec<LibraryEntry> = groups
        .into_values()
        .map(|mut entry| {
            entry.files.sort_by(|left, right| {
                let left_name = left.path.file_name().and_then(|name| name.to_str()).unwrap_or("");
                let right_name = right.path.file_name().and_then(|name| name.to_str()).unwrap_or("");
                left_name.to_lowercase().cmp(&right_name.to_lowercase())
            });
            LibraryEntry {
                key: entry.key,
                name: entry.name,
                name_is_fallback: entry.name_is_fallback,
                cover_path: None,
                files: entry.files,
            }
        })
        .collect();
    entries.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.key.cmp(&right.key))
    });

    LibrarySnapshot { entries, error: None }
}

fn collect_files(directory: &Path, expected: FileKind) -> Vec<(PathBuf, FileKind)> {
    let mut collected = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return collected;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && FileKind::from_path(&path) == expected {
            collected.push((path, expected));
        }
    }
    collected
}

fn numeric_stem(stem: &str) -> Option<String> {
    (!stem.is_empty() && stem.chars().all(|character| character.is_ascii_digit()))
        .then(|| stem.to_owned())
}

fn manifest_depot_id(stem: &str) -> Option<String> {
    let first = stem.split('_').next()?;
    (!first.is_empty() && first.chars().all(|character| character.is_ascii_digit()))
        .then(|| first.to_owned())
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Unknown file")
        .to_owned()
}

fn group_stem(stem: &str) -> String {
    if let Some((prefix, suffix)) = stem.rsplit_once('_') {
        if !prefix.is_empty() && !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit()) {
            return prefix.to_owned();
        }
    }
    stem.to_owned()
}

fn normalized_key(value: &str) -> String {
    value.trim().to_lowercase()
}

fn display_stem(stem: &str) -> String {
    if !stem.is_empty() && stem.chars().all(|character| character.is_ascii_digit()) {
        return format!("App {stem}");
    }
    let name = stem.replace('_', " ").replace('-', " ").trim().to_owned();
    if name.is_empty() {
        "Unknown game".into()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::{display_stem, group_stem, manifest_depot_id, numeric_stem};

    #[test]
    fn matching_file_stems_group_by_name() {
        assert_eq!(group_stem("MyGame_884422"), "MyGame");
        assert_eq!(group_stem("My_Game"), "My_Game");
        assert_eq!(display_stem("My_Game"), "My Game");
    }

    #[test]
    fn numeric_stems_are_accepted_as_app_ids() {
        assert_eq!(numeric_stem("2651280").as_deref(), Some("2651280"));
        assert_eq!(numeric_stem("MyGame"), None);
        assert_eq!(numeric_stem(""), None);
    }

    #[test]
    fn depot_id_is_extracted_from_manifest_name() {
        assert_eq!(
            manifest_depot_id("2651281_2482069238759486759").as_deref(),
            Some("2651281"),
        );
        assert_eq!(manifest_depot_id("readme"), None);
    }
}