//! Read-only Steam metadata discovery used only to improve local library names.
//!
//! This module deliberately exposes no mutating Steam filesystem operation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
#[cfg(target_os = "windows")]
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use super::keyvalues::{self, Value};

/// Windows flag that prevents a spawned console program from opening a
/// console window. `reg.exe` and other console utilities would otherwise
/// flash a black window every time they run.
#[cfg(target_os = "windows")]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// App metadata read from an existing local Steam installation.
#[derive(Debug, Clone)]
pub(crate) struct AppInfo {
    /// Display name stored in Steam's local metadata.
    pub name: String,
}

/// Read-only maps that relate installed app IDs and depot IDs to game names.
#[derive(Debug, Clone, Default)]
pub(crate) struct SteamMetadata {
    /// Installed apps indexed by their AppID.
    pub apps: HashMap<String, AppInfo>,
    /// Known depot IDs mapped to the installed app that owns them.
    pub depot_owners: HashMap<String, String>,
}

/// Cached result of the first Steam root lookup. The underlying probe
/// spawns `reg.exe` on Windows and walks several conventional paths on
/// Linux. Caching means the probe runs once per process instead of on
/// every library refresh, and it removes the repeated console flashes
/// that used to appear whenever the library was rescanned.
static ROOT_CACHE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Locates a Steam client folder on supported operating systems.
pub(crate) fn discover_root() -> Option<PathBuf> {
    ROOT_CACHE.get_or_init(discover_root_uncached).clone()
}

fn discover_root_uncached() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("STEAM_DIR") {
        candidates.push(PathBuf::from(path));
    }

    #[cfg(target_os = "windows")]
    {
        candidates.extend(registry_paths());
        if let Some(path) = std::env::var_os("ProgramFiles(x86)") {
            candidates.push(PathBuf::from(path).join("Steam"));
        }
        if let Some(path) = std::env::var_os("ProgramFiles") {
            candidates.push(PathBuf::from(path).join("Steam"));
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.extend([
            home.join(".steam/steam"),
            home.join(".steam/root"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ]);
    }

    candidates.into_iter().find(|path| is_steam_root(path))
}

/// The directory where Steam loads Lua depot configurations from.
/// This is the folder BetterSteamTools and OpenSteamTool read at start-up.
pub(crate) fn stplug_in_dir(root: &Path) -> PathBuf {
    root.join("config").join("stplug-in")
}

/// The directory where Steam looks for depot manifests by filename.
/// Files must be named `<depot_id>_<manifest_gid>.manifest`.
pub(crate) fn depotcache_dir(root: &Path) -> PathBuf {
    root.join("depotcache")
}

/// Reads installed app names and depot ownership without modifying Steam files.
pub(crate) fn read_metadata(root: &Path) -> Result<SteamMetadata, String> {
    if !is_steam_root(root) {
        return Err(format!("Not a Steam client directory: {}", root.display()));
    }

    let steamapps = root.join("steamapps");
    let mut steamapps_dirs = vec![steamapps.clone()];
    let library_file = steamapps.join("libraryfolders.vdf");
    if let Ok(source) = fs::read_to_string(library_file) {
        for path in library_paths(&source) {
            let path = path.join("steamapps");
            if path.is_dir() && !contains_path(&steamapps_dirs, &path) {
                steamapps_dirs.push(path);
            }
        }
    }

    let mut metadata = SteamMetadata::default();
    for directory in &steamapps_dirs {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !filename.starts_with("appmanifest_")
                || !filename.ends_with(".acf")
                || !path.is_file()
            {
                continue;
            }
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            let nodes = keyvalues::parse(&source);
            let app_state = keyvalues::object(&nodes, "AppState").unwrap_or(&nodes);
            let app_id = keyvalues::scalar(app_state, "appid")
                .map(str::to_owned)
                .or_else(|| app_id_from_filename(filename));
            let Some(app_id) = app_id else {
                continue;
            };
            let name = keyvalues::scalar(app_state, "name")
                .filter(|name| !name.trim().is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("App {app_id}"));
            metadata.apps.insert(app_id.clone(), AppInfo { name });

            if let Some(depots) = keyvalues::object(app_state, "InstalledDepots") {
                for (depot_id, value) in depots {
                    if matches!(value, Value::Object(_))
                        && depot_id.chars().all(|character| character.is_ascii_digit())
                    {
                        metadata.depot_owners.insert(depot_id.clone(), app_id.clone());
                    }
                }
            }
        }
    }
    Ok(metadata)
}

fn is_steam_root(path: &Path) -> bool {
    path.is_dir() && path.join("steamapps").is_dir()
}

fn contains_path(paths: &[PathBuf], candidate: &Path) -> bool {
    paths.iter().any(|path| {
        let left = path.to_string_lossy();
        let right = candidate.to_string_lossy();
        if cfg!(windows) {
            left.eq_ignore_ascii_case(&right)
        } else {
            left == right
        }
    })
}

fn library_paths(source: &str) -> Vec<PathBuf> {
    let nodes = keyvalues::parse(source);
    let libraries = keyvalues::object(&nodes, "libraryfolders").unwrap_or(&nodes);
    let mut paths = Vec::new();
    for (key, value) in libraries {
        if key == "0" || key.chars().all(|character| character.is_ascii_digit()) {
            match value {
                Value::Scalar(path) => paths.push(PathBuf::from(path)),
                Value::Object(fields) => {
                    if let Some(path) = keyvalues::scalar(fields, "path") {
                        paths.push(PathBuf::from(path));
                    }
                }
            }
        }
    }
    paths
}

fn app_id_from_filename(filename: &str) -> Option<String> {
    filename
        .strip_prefix("appmanifest_")?
        .strip_suffix(".acf")
        .filter(|id| !id.is_empty() && id.chars().all(|character| character.is_ascii_digit()))
        .map(str::to_owned)
}

#[cfg(target_os = "windows")]
fn registry_paths() -> Vec<PathBuf> {
    [
        r"HKCU\Software\Valve\Steam",
        r"HKLM\Software\Wow6432Node\Valve\Steam",
        r"HKLM\Software\Valve\Steam",
    ]
    .into_iter()
    .filter_map(|key| {
        // Suppress the console window that would otherwise flash for
        // every reg.exe invocation.
        let mut command = Command::new("reg");
        command.args(["query", key, "/v", "SteamPath"]);
        command.creation_flags(CREATE_NO_WINDOW);
        let output = command.output().ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8_lossy(&output.stdout).lines().find_map(|line| {
            let (_, value) = line.split_once("REG_SZ")?;
            let value = value.trim();
            (!value.is_empty()).then(|| PathBuf::from(value))
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::app_id_from_filename;

    #[test]
    fn app_id_is_read_from_standard_appmanifest_name() {
        assert_eq!(app_id_from_filename("appmanifest_12345.acf"), Some("12345".into()));
        assert_eq!(app_id_from_filename("appmanifest_test.acf"), None);
    }
}