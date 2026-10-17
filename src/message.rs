//! The top-level message enum for every user action and filesystem result.

use std::path::PathBuf;

use iced::window;

use crate::state::{StorageKind, View, library_state::ImportOutcome};

/// An event handled by the application update loop.
#[derive(Debug, Clone)]
pub enum Message {
    /// Native window event, including file drops.
    WindowEvent(window::Event),

    /// Minimize the window.
    Minimize,
    /// Toggle maximized window state.
    ToggleMaximize,
    /// Close the window.
    Close,
    /// Begin dragging the frameless window.
    StartDrag,
    /// Resize the frameless window from an edge or corner.
    Resize(window::Direction),

    /// Navigate to a screen.
    Navigate(View),
    /// Expand or collapse advanced options.
    ToggleAdvanced,
    /// Open the contextual storage guide.
    OpenStorageGuide,
    /// Close the contextual storage guide.
    CloseStorageGuide,
    /// Consume clicks on the dimmed guide backdrop without forwarding them to the page.
    StorageGuideBackdropPressed,

    /// Open a multi-select file chooser.
    ChooseFiles,
    /// Result from the native file chooser.
    FilesChosen(Result<Vec<PathBuf>, String>),
    /// Remove a staged path without importing it.
    RemovePending(PathBuf),
    /// Clear the staging list.
    ClearPending,
    /// Copy staged files into the configured local library.
    ImportPending,
    /// Background import result.
    ImportCompleted(ImportOutcome),
    /// Refresh the filesystem-backed library view without a success notification.
    RefreshLibrary,
    /// Result of a quick, local-only library scan.
    LibraryRefreshed(crate::state::library_state::LibrarySnapshot),
    /// Result of background Steam name and cover-art resolution.
    LibraryArtworkRefreshed(crate::state::library_state::LibrarySnapshot),
    /// Expand or collapse one game entry in the library screen.
    ToggleLibraryEntry(String),
    /// A notice timer elapsed; the token prevents clearing a newer notice.
    NoticeExpired(u64),

    /// Ask for confirmation before deleting one library entry.
    RequestRemove(String),
    /// Delete files in a confirmed library entry.
    ConfirmRemove,
    /// Dismiss the remove confirmation.
    CancelRemove,

    /// Update the manifest storage path field.
    ManifestDirChanged(String),
    /// Update the Lua storage path field.
    LuaDirChanged(String),
    /// Open a folder picker for manifest storage.
    BrowseManifestDir,
    /// Open a folder picker for Lua storage.
    BrowseLuaDir,
    /// Result from a folder chooser.
    StorageDirectoryChosen(StorageKind, Result<Option<PathBuf>, String>),
    /// Validate and persist both storage folders.
    SaveStoragePaths,
}
