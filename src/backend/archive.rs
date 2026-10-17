//! Default local storage and validation for the two import destinations.
//!
//! By default, imports go directly into Steam's active directories:
//!   .lua files      -> <Steam>\config\stplug-in
//!   .manifest files -> <Steam>\depotcache
//!
//! Those two directories are the ones Steam's own client, and BetterSteamTools,
//! read at start-up. Writing to them is the whole point of the tool.
//!
//! If no Steam installation is discovered, the tool falls back to a private
//! local library under the user's application data folder so the UI still
//! works on machines without Steam.

use std::fs;
use std::path::{Component, Path, PathBuf};

use super::steam;

/// The two independent import destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageSettings {
    /// Folder used for `.manifest` and `.manifests` files.
    pub manifest_dir: PathBuf,
    /// Folder used for `.lua` files.
    pub lua_dir: PathBuf,
}

/// Returns the default destinations. Prefers Steam's active directories
/// when a Steam installation is present; otherwise uses a private local
/// library so the application remains usable.
pub(crate) fn default_settings() -> StorageSettings {
    if let Some(root) = steam::discover_root() {
        return StorageSettings {
            manifest_dir: steam::depotcache_dir(&root),
            lua_dir: steam::stplug_in_dir(&root),
        };
    }

    let root = data_directory().join("Locomotive").join("library");
    StorageSettings {
        manifest_dir: root.join("manifests"),
        lua_dir: root.join("lua"),
    }
}

/// Loads saved locations, falling back to the current defaults on invalid data.
pub(crate) fn load_settings() -> StorageSettings {
    let defaults = default_settings();
    let Ok(contents) = fs::read_to_string(settings_file()) else {
        return defaults;
    };

    let mut manifest_dir = None;
    let mut lua_dir = None;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("manifest=") {
            manifest_dir = Some(PathBuf::from(value));
        } else if let Some(value) = line.strip_prefix("lua=") {
            lua_dir = Some(PathBuf::from(value));
        }
    }

    let Some(manifest_dir) = manifest_dir else {
        return defaults;
    };
    let Some(lua_dir) = lua_dir else {
        return defaults;
    };
    match normalize_and_validate(manifest_dir, lua_dir) {
        Ok(settings) => settings,
        Err(_) => defaults,
    }
}

/// Validates and persists the two import destinations.
pub(crate) fn save_settings(
    manifest_dir: &Path,
    lua_dir: &Path,
) -> Result<StorageSettings, String> {
    let settings = normalize_and_validate(manifest_dir.to_path_buf(), lua_dir.to_path_buf())?;
    fs::create_dir_all(&settings.manifest_dir).map_err(|error| {
        format!("Could not create manifest folder '{}': {error}", settings.manifest_dir.display())
    })?;
    fs::create_dir_all(&settings.lua_dir).map_err(|error| {
        format!("Could not create Lua folder '{}': {error}", settings.lua_dir.display())
    })?;

    let file = settings_file();
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create settings folder '{}': {error}", parent.display()))?;
    }
    let contents = format!(
        "manifest={}\nlua={}\n",
        settings.manifest_dir.display(),
        settings.lua_dir.display()
    );
    fs::write(&file, contents)
        .map_err(|error| format!("Could not save storage settings '{}': {error}", file.display()))?;

    Ok(settings)
}

/// Returns a settings-independent pair of normalized paths, rejecting
/// unsafe overlaps and paths inside a Steam installation that are not
/// one of the two active directories the tool is designed to write to.
pub(crate) fn normalize_and_validate(
    manifest_dir: PathBuf,
    lua_dir: PathBuf,
) -> Result<StorageSettings, String> {
    let manifest_dir = resolve_existing_prefix(&manifest_dir)?;
    let lua_dir = resolve_existing_prefix(&lua_dir)?;

    if same_or_nested(&manifest_dir, &lua_dir) || same_or_nested(&lua_dir, &manifest_dir) {
        return Err("Manifest and Lua folders must be separate and must not contain one another.".into());
    }
    for path in [&manifest_dir, &lua_dir] {
        if is_rejected_steam_path(path) {
            return Err(format!(
                "'{}' is inside a Steam installation but is not one of the two folders the tool writes to. Use <Steam>\\depotcache for manifests and <Steam>\\config\\stplug-in for Lua files, or pick a folder outside Steam.",
                path.display()
            ));
        }
    }
    Ok(StorageSettings { manifest_dir, lua_dir })
}

fn absolute_normalized(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("Could not resolve relative path '{}': {error}", path.display()))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

/// Resolves existing symlinked ancestors before validating a destination. The final
/// directory does not need to exist yet; any missing suffix is appended after the
/// nearest existing ancestor has been canonicalized.
fn resolve_existing_prefix(path: &Path) -> Result<PathBuf, String> {
    let normalized = absolute_normalized(path)?;
    let mut existing = normalized.as_path();
    let mut missing = Vec::new();

    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            format!("Could not resolve storage directory '{}'.", normalized.display())
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            format!("Could not resolve storage directory '{}'.", normalized.display())
        })?;
    }

    let mut resolved = fs::canonicalize(existing).map_err(|error| {
        format!("Could not resolve existing path '{}': {error}", existing.display())
    })?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    absolute_normalized(&resolved)
}

fn same_or_nested(path: &Path, parent: &Path) -> bool {
    if cfg!(windows) {
        let path = path.to_string_lossy().trim_end_matches(|ch| ch == '\\' || ch == '/').to_lowercase();
        let parent = parent.to_string_lossy().trim_end_matches(|ch| ch == '\\' || ch == '/').to_lowercase();
        path == parent || path.starts_with(&format!("{parent}\\")) || path.starts_with(&format!("{parent}/"))
    } else {
        path == parent || path.starts_with(parent)
    }
}

/// True if `path` is exactly the stplug-in directory or a subdirectory of it.
fn is_stplug_in(path: &Path) -> bool {
    let components: Vec<String> = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    components.windows(2).any(|pair| {
        pair[0].eq_ignore_ascii_case("config") && pair[1].eq_ignore_ascii_case("stplug-in")
    })
}

/// True if `path` is exactly a directory named `depotcache` or a subdirectory of one.
fn is_depotcache(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str().to_string_lossy().eq_ignore_ascii_case("depotcache")
    })
}

/// True when the path is inside a Steam installation but is not one of the
/// two active directories the tool is designed to write to.
fn is_rejected_steam_path(path: &Path) -> bool {
    if is_stplug_in(path) || is_depotcache(path) {
        return false;
    }
    // Block writing directly inside steamapps (protects game files)
    // but allow custom folders elsewhere in the Steam directory (e.g. config/lua).
    path.ancestors().any(|ancestor| {
        ancestor.join("steamapps").is_dir() && ancestor.file_name().and_then(|n| n.to_str()) == Some("steamapps")
    })
}

fn settings_file() -> PathBuf {
    config_directory().join("Locomotive").join("paths.conf")
}

fn data_directory() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(path);
    }
    if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(path);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local").join("share");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}


/// Returns the private cache directory used for game names and cover art.
pub(crate) fn cache_directory() -> PathBuf {
    data_directory().join("Locomotive").join("cache")
}

fn config_directory() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("APPDATA") {
        return PathBuf::from(path);
    }
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::normalize_and_validate;

    #[test]
    fn storage_directories_must_not_overlap() {
        let root = std::env::temp_dir().join("locomotive-storage-validation");
        let result = normalize_and_validate(root.join("files"), root.join("files/lua"));
        assert!(result.is_err());
    }

    #[test]
    fn independent_directories_are_accepted_outside_steam() {
        let root = std::env::temp_dir().join("locomotive-storage-validation");
        let result = normalize_and_validate(root.join("manifests"), root.join("lua"));
        assert!(result.is_ok());
    }

    #[test]
    fn normalizes_relative_path_components() {
        let root = std::env::temp_dir();
        let result = normalize_and_validate(
            root.join("locomotive-normalize/../manifests"),
            root.join("locomotive-normalize/lua"),
        );
        assert!(result.is_ok());
        let settings = result.expect("the normalized paths should be independent");
        assert_eq!(settings.manifest_dir, root.join("manifests"));
        assert_eq!(settings.lua_dir, root.join("locomotive-normalize/lua"));
    }

    #[test]
    fn the_two_steam_active_directories_are_accepted() {
        let root = std::env::temp_dir().join(format!("locomotive-steam-path-test-{}", std::process::id()));
        let steamapps = root.join("steamapps");
        let config = root.join("config");
        std::fs::create_dir_all(&steamapps).expect("test steamapps directory should be created");
        std::fs::create_dir_all(&config).expect("test config directory should be created");
        let result = normalize_and_validate(
            root.join("depotcache"),
            config.join("stplug-in"),
        );
        let _ = std::fs::remove_dir_all(&root);
        assert!(result.is_ok(), "the tool's two active directories must be permitted");
    }

    #[test]
    fn other_paths_inside_a_steam_installation_are_rejected() {
        let root = std::env::temp_dir().join(format!("locomotive-steam-reject-test-{}", std::process::id()));
        let steamapps = root.join("steamapps");
        let config = root.join("config");
        std::fs::create_dir_all(&steamapps).expect("test steamapps directory should be created");
        std::fs::create_dir_all(&config).expect("test config directory should be created");
        let result = normalize_and_validate(
            steamapps.join("common").join("SomeGame"),
            config.join("backup"),
        );
        let _ = std::fs::remove_dir_all(&root);
        assert!(result.is_err(), "arbitrary paths inside Steam must still be rejected");
    }

    #[test]
    fn accepts_manifest_extension_aliases() {
        use crate::state::FileKind;
        assert_eq!(FileKind::from_path(&PathBuf::from("42.manifests")), FileKind::Manifest);
        assert_eq!(FileKind::from_path(&PathBuf::from("42.MANIFEST")), FileKind::Manifest);
    }
}