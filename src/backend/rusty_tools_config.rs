//! Read-only helper for `<Steam>\rustytools.toml`.
//!
//! RustyTools is the program that actually loads `.lua` files into Steam.
//! Its `[lua] dir` setting is therefore the source of truth for "which
//! directory contains live lua files". Locomotive must not scan any
//! other directory as if it were the primary library; leftover files in
//! `stplug-in` from a previous configuration do not belong there.
//!
//! This module parses exactly one key. Everything else in
//! `rustytools.toml` is owned by RustyTools and is never read here.

use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "rustytools.toml";
const DEFAULT_LUA_SUBDIR: &str = "config\\stplug-in";

/// The manifest directory Steam always reads from, independent of any
/// RustyTools setting. RustyTools has no `[manifest] dir`; Steam's
/// depot reader is hardcoded to `<Steam>\depotcache`.
pub(crate) fn depotcache_dir(steam_root: &Path) -> PathBuf {
    steam_root.join("depotcache")
}

/// The Lua directory RustyTools loads files from.
///
/// Returns `<steam>\config\stplug-in` when the config is missing, has no
/// `[lua]` section, or has an empty `dir`. This matches the fallback in
/// `rusty_tools/src/utils/config/config.rs::get_lua_dir`.
pub(crate) fn configured_lua_dir(steam_root: &Path) -> PathBuf {
    explicit_lua_dir(steam_root).unwrap_or_else(|| steam_root.join(DEFAULT_LUA_SUBDIR))
}

/// Returns `Some(path)` when `rustytools.toml` has an explicit `[lua] dir`.
///
/// A relative value is resolved against `steam_root`, matching
/// RustyTools' own behaviour. Absolute values are returned as-is so a
/// lua directory that lives outside the Steam tree is honoured.
pub(crate) fn explicit_lua_dir(steam_root: &Path) -> Option<PathBuf> {
    let config_path = steam_root.join(CONFIG_FILE);
    let content = std::fs::read_to_string(&config_path).ok()?;
    parse_lua_dir(&content, steam_root)
}

fn parse_lua_dir(toml: &str, steam_root: &Path) -> Option<PathBuf> {
    let mut in_lua = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_lua = trimmed == "[lua]";
            continue;
        }
        if !in_lua {
            continue;
        }
        let Some(eq) = trimmed.find('=') else { continue };
        let key = trimmed[..eq].trim();
        if key != "dir" {
            continue;
        }
        let raw = trimmed[eq + 1..].trim();
        // Strip a matching quote pair, then unescape `\\` and `\"` the
        // same way `SaveLuaDir` in `update/mod.rs` escapes them on write.
        let value = raw
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| raw.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(raw);
        let unescaped = value.replace("\\\\", "\\").replace("\\\"", "\"");
        if unescaped.is_empty() {
            return None;
        }
        let path = Path::new(&unescaped);
        return Some(if path.is_absolute() {
            path.to_path_buf()
        } else {
            steam_root.join(path)
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_lua_dir;
    use std::path::PathBuf;

    fn root() -> PathBuf { PathBuf::from("C:\\Steam") }

    #[test]
    fn missing_lua_section_returns_none() {
        assert!(parse_lua_dir("[log]\nlevel = \"info\"\n", &root()).is_none());
    }

    #[test]
    fn empty_dir_returns_none() {
        assert!(parse_lua_dir("[lua]\ndir = \"\"\n", &root()).is_none());
    }

    #[test]
    fn absolute_path_is_returned_as_is() {
        let got = parse_lua_dir("[lua]\ndir = \"C:\\\\Other\\\\lua\"\n", &root()).unwrap();
        assert_eq!(got, PathBuf::from("C:\\Other\\lua"));
    }

    #[test]
    fn relative_path_is_joined_to_steam_root() {
        let got = parse_lua_dir("[lua]\ndir = \"my_lua\"\n", &root()).unwrap();
        assert_eq!(got, root().join("my_lua"));
    }

    #[test]
    fn single_quoted_value_is_accepted() {
        let got = parse_lua_dir("[lua]\ndir = 'C:\\\\Other\\\\lua'\n", &root()).unwrap();
        assert_eq!(got, PathBuf::from("C:\\Other\\lua"));
    }

    #[test]
    fn later_sections_do_not_override_lua() {
        let src = "[lua]\ndir = \"C:\\\\keep\"\n[other]\ndir = \"ignored\"\n";
        let got = parse_lua_dir(src, &root()).unwrap();
        assert_eq!(got, PathBuf::from("C:\\keep"));
    }

    #[test]
    fn comment_inside_lua_section_is_ignored() {
        let src = "[lua]\n# dir = \"ignored\"\ndir = \"C:\\\\real\"\n";
        let got = parse_lua_dir(src, &root()).unwrap();
        assert_eq!(got, PathBuf::from("C:\\real"));
    }
}