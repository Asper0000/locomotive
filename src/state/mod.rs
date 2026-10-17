//! Root application state, the navigation enum, and window subscriptions.

use std::collections::HashSet;
use std::path::PathBuf;

use iced::Subscription;

use crate::backend::{archive, library};
use crate::message::Message;

pub mod library_state;

/// A screen in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// File import and staging.
    #[default]
    Import,
    /// Files already stored in the local library.
    Library,
    /// Storage locations and application settings.
    Options,
    /// Application information.
    About,
}

/// The recognized file categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// A Steam depot manifest (`.manifest` or `.manifests`).
    Manifest,
    /// A Lua file (`.lua`).
    Lua,
    /// An unsupported file type.
    Unknown,
}

impl FileKind {
    /// Determines a supported kind from a path's extension.
    pub fn from_path(path: &std::path::Path) -> Self {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension)
                if extension.eq_ignore_ascii_case("manifest")
                    || extension.eq_ignore_ascii_case("manifests") =>
            {
                Self::Manifest
            }
            Some(extension) if extension.eq_ignore_ascii_case("lua") => Self::Lua,
            _ => Self::Unknown,
        }
    }

    /// Returns a short label suitable for the UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Manifest => "manifest",
            Self::Lua => "lua",
            Self::Unknown => "unsupported",
        }
    }
}

/// A source file staged for import into the local library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingFile {
    /// Original path selected by the user or received through drag-and-drop.
    pub path: PathBuf,
    /// File type inferred from the extension.
    pub kind: FileKind,
}

/// The directory setting currently being selected in a native folder dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    /// Location for manifest copies.
    Manifest,
    /// Location for Lua copies.
    Lua,
}

/// All mutable application state used by the Elm-style update/view flow.
#[derive(Debug)]
pub struct State {
    /// Currently visible screen.
    pub active_view: View,
    /// Files awaiting import.
    pub pending_files: Vec<PendingFile>,
    /// Whether the pointer is carrying a file over the window.
    pub is_drag_over: bool,
    /// Whether the advanced options panel is expanded.
    pub advanced_open: bool,
    /// Whether the contextual storage guide dialog is visible.
    pub storage_guide_open: bool,
    /// Folder where manifest copies are stored.
    pub manifest_dir: String,
    /// Folder where Lua copies are stored.
    pub lua_dir: String,
    /// Snapshot reconstructed from the configured local library folders.
    pub library: library_state::LibrarySnapshot,
    /// A transient status or error message displayed by the UI.
    pub notice: Option<String>,
    /// Entry key awaiting removal confirmation.
    pub confirm_remove: Option<String>,
    /// Prevents starting multiple simultaneous imports.
    pub is_importing: bool,
    /// Prevents overlapping filesystem scans and artwork lookups.
    pub is_refreshing_library: bool,
    /// Schedules one follow-up scan when a change occurs during a refresh.
    pub library_refresh_pending: bool,
    /// Entries whose file lists are expanded in the library screen.
    pub expanded_library_entries: HashSet<String>,
    /// Monotonic token used to prevent old timers dismissing newer notices.
    pub notice_sequence: u64,
}

impl Default for State {
    fn default() -> Self {
        let settings = archive::load_settings();
        let snapshot = library::scan(&settings.manifest_dir, &settings.lua_dir);
        Self {
            active_view: View::Import,
            pending_files: Vec::new(),
            is_drag_over: false,
            advanced_open: false,
            storage_guide_open: false,
            manifest_dir: settings.manifest_dir.to_string_lossy().into_owned(),
            lua_dir: settings.lua_dir.to_string_lossy().into_owned(),
            library: snapshot,
            notice: None,
            confirm_remove: None,
            is_importing: false,
            is_refreshing_library: false,
            library_refresh_pending: false,
            expanded_library_entries: HashSet::new(),
            notice_sequence: 0,
        }
    }
}

impl State {
    /// Subscribes to native window events, including file drag-and-drop.
    pub fn subscription(&self) -> Subscription<Message> {
        iced::window::events().map(|(_id, event)| Message::WindowEvent(event))
    }

    /// Adds valid and unsupported drops to the staging list without duplicating paths.
    pub fn stage_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            let already_staged = self.pending_files.iter().any(|file| {
                file.path == path
                    || file
                        .path
                        .canonicalize()
                        .ok()
                        .zip(path.canonicalize().ok())
                        .is_some_and(|(left, right)| left == right)
            });
            if !already_staged {
                self.pending_files.push(PendingFile {
                    kind: FileKind::from_path(&path),
                    path,
                });
            }
        }
    }
}
