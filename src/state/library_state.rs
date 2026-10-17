//! Types representing the manager's filesystem-backed local library.

use std::path::PathBuf;

use super::FileKind;

/// A supported file stored in one of the configured local library folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryFile {
    /// Full path to the stored copy.
    pub path: PathBuf,
    /// Category inferred from the file extension.
    pub kind: FileKind,
}

/// A problem detected while scanning: this entry has `.lua` files that
/// live outside the configured `[lua] dir`, so RustyTools will not load
/// them even though the app is installed in Steam. The files are still
/// real and safe to move; the entry is shown so the user can choose
/// between fixing and removing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceWarning {
    /// `.lua` files currently sitting in the wrong directory.
    pub current_paths: Vec<PathBuf>,
    /// The directory they need to move into.
    pub configured_dir: PathBuf,
}

/// A group of stored files that appear to belong to the same game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Stable key used to address this entry from UI actions.
    pub key: String,
    /// Display name inferred from filenames and, when available, Steam metadata.
    pub name: String,
    /// Whether the name is only a filename-derived fallback that may be replaced by online metadata.
    pub name_is_fallback: bool,
    /// Path to a local or downloaded cover image, if one is available.
    pub cover_path: Option<PathBuf>,
    /// Files stored for this entry. These are files *inside* the
    /// configured library folders. Orphaned `.lua` files are not in
    /// this list; they are tracked separately in `source_warning`.
    pub files: Vec<LibraryFile>,
    /// Set when the scanner found `.lua` files for this installed app
    /// outside the configured `[lua] dir`. `None` for a healthy entry.
    pub source_warning: Option<SourceWarning>,
}

impl LibraryEntry {
    /// Number of manifest files in this entry.
    pub fn manifest_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.kind == FileKind::Manifest)
            .count()
    }

    /// Number of Lua files in this entry that live inside the configured
    /// library folders. Does not include orphaned files; use
    /// `source_warning.current_paths.len()` for those.
    pub fn lua_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.kind == FileKind::Lua)
            .count()
    }

    /// Number of orphaned Lua files, if any.
    pub fn orphan_lua_count(&self) -> usize {
        self.source_warning
            .as_ref()
            .map_or(0, |warning| warning.current_paths.len())
    }
}

/// A point-in-time scan of the configured local storage folders.
#[derive(Debug, Clone, Default)]
pub struct LibrarySnapshot {
    /// Grouped library entries, rebuilt from the filesystem on refresh.
    pub entries: Vec<LibraryEntry>,
    /// Error encountered while scanning, if any.
    pub error: Option<String>,
}

/// Summary of a completed import operation.
#[derive(Debug, Clone)]
pub struct ImportOutcome {
    /// Library state after the import attempt.
    pub snapshot: LibrarySnapshot,
    /// Files copied successfully.
    pub imported: usize,
    /// Files skipped because they were unsupported, missing, or already present.
    pub skipped: usize,
    /// Errors that explain any partial failure.
    pub errors: Vec<String>,
}