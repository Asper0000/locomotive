//! Routes UI messages, starts background work, and keeps transient UI state tidy.

pub mod window;

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use iced::window as iced_window;
use iced::Task;

use crate::backend::{archive, file_dialog, importer, library};
use crate::message::Message;
use crate::state::{State, StorageKind, View};

fn with_window(
    f: impl FnOnce(iced_window::Id) -> Task<Message> + Send + 'static,
) -> Task<Message> {
    let mut f = Some(f);
    iced_window::latest().then(move |id| {
        let Some(f) = f.take() else {
            return Task::none();
        };
        match id {
            Some(id) => f(id),
            None => Task::none(),
        }
    })
}

/// Applies one UI message and returns any asynchronous work it requested.
pub fn handle(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Minimize => with_window(|id| iced_window::minimize(id, true)),
        Message::ToggleMaximize => with_window(iced_window::toggle_maximize),
        Message::Close => with_window(iced_window::close),
        Message::StartDrag => with_window(iced_window::drag),
        Message::Resize(direction) => with_window(move |id| iced_window::drag_resize(id, direction)),
        Message::WindowEvent(event) => window::handle_event(state, event),
        Message::Navigate(view) => {
            state.active_view = view;
            if view == View::Library && !state.is_refreshing_library {
                start_library_refresh(state)
            } else {
                Task::none()
            }
        }
        Message::ToggleAdvanced => {
            state.advanced_open = !state.advanced_open;
            Task::none()
        }
        Message::OpenStorageGuide => {
            state.storage_guide_open = true;
            Task::none()
        }
        Message::CloseStorageGuide => {
            state.storage_guide_open = false;
            Task::none()
        }
        Message::StorageGuideBackdropPressed => Task::none(),
        Message::ChooseFiles => Task::perform(async { file_dialog::choose_files() }, Message::FilesChosen),
        Message::FilesChosen(result) => match result {
            Ok(paths) => {
                if paths.is_empty() {
                    Task::none()
                } else {
                    state.stage_paths(paths);
                    show_notice(state, "Files staged. Select Import to copy them into the local library.")
                }
            }
            Err(error) => show_notice(state, error),
        },
        Message::RemovePending(path) => {
            state.pending_files.retain(|file| file.path != path);
            Task::none()
        }
        Message::ClearPending => {
            state.pending_files.clear();
            clear_notice(state);
            Task::none()
        }
        Message::ImportPending => {
            if state.is_importing {
                return Task::none();
            }
            if !state.pending_files.iter().any(|file| file.kind != crate::state::FileKind::Unknown) {
                return show_notice(
                    state,
                    "Choose or drop at least one .manifest, .manifests, or .lua file first.",
                );
            }

            let manifest_dir = PathBuf::from(&state.manifest_dir);
            let lua_dir = PathBuf::from(&state.lua_dir);
            let files = state.pending_files.clone();
            state.is_importing = true;
            let notice_task = show_notice(state, "Importing files into the local library…");
            let import_task = Task::perform(
                async move { importer::import_files(&manifest_dir, &lua_dir, &files) },
                Message::ImportCompleted,
            );
            Task::batch([notice_task, import_task])
        }
        Message::ImportCompleted(outcome) => {
            state.is_importing = false;
            state.library = outcome.snapshot;
            state.pending_files.clear();

            let mut details = Vec::new();
            if outcome.imported > 0 {
                details.push(format!("Added {} file(s)", outcome.imported));
            }
            if outcome.skipped > 0 {
                details.push(format!("skipped {} file(s)", outcome.skipped));
            }
            if !outcome.errors.is_empty() {
                details.push(outcome.errors.join(" · "));
            }
            let notice = if details.is_empty() {
                "No new files were added.".to_owned()
            } else {
                details.join(" — ")
            };
            let notice_task = show_notice(state, notice);
            let refresh_task = start_library_refresh(state);
            Task::batch([notice_task, refresh_task])
        }
        Message::RefreshLibrary => start_library_refresh(state),
        Message::LibraryRefreshed(snapshot) => {
            state.library = snapshot.clone();
            let artwork_task = Task::perform(
                async move {
                    let mut snapshot = snapshot;
                    crate::backend::artwork::enrich_remote(&mut snapshot);
                    snapshot
                },
                Message::LibraryArtworkRefreshed,
            );
            retain_expanded_entries(state);
            artwork_task
        }
        Message::LibraryArtworkRefreshed(snapshot) => {
            state.library = snapshot;
            state.is_refreshing_library = false;
            retain_expanded_entries(state);
            if state.library_refresh_pending {
                state.library_refresh_pending = false;
                start_library_refresh(state)
            } else {
                Task::none()
            }
        }
        Message::ToggleLibraryEntry(key) => {
            if !state.expanded_library_entries.remove(&key) {
                state.expanded_library_entries.insert(key);
            }
            Task::none()
        }
        Message::NoticeExpired(token) => {
            if token == state.notice_sequence {
                state.notice = None;
            }
            Task::none()
        }
        Message::RequestRemove(key) => {
            state.confirm_remove = Some(key);
            Task::none()
        }
        Message::ConfirmRemove => {
            let Some(key) = state.confirm_remove.take() else {
                return Task::none();
            };
            let Some(entry) = state.library.entries.iter().find(|entry| entry.key == key).cloned() else {
                return show_notice(state, "That library entry is no longer available. Refresh and try again.");
            };

            match importer::remove_entry(
                &PathBuf::from(&state.manifest_dir),
                &PathBuf::from(&state.lua_dir),
                &entry,
            ) {
                Ok(removed) => {
                    let notice_task = show_notice(
                        state,
                        format!("Removed {removed} local file(s). The installed game files were not changed."),
                    );
                    let refresh_task = start_library_refresh(state);
                    Task::batch([notice_task, refresh_task])
                }
                Err(error) => show_notice(state, error),
            }
        }
        Message::CancelRemove => {
            state.confirm_remove = None;
            Task::none()
        }
        Message::ManifestDirChanged(path) => {
            state.manifest_dir = path;
            Task::none()
        }
        Message::LuaDirChanged(path) => {
            state.lua_dir = path;
            Task::none()
        }
        Message::BrowseManifestDir => choose_storage_dir(StorageKind::Manifest),
        Message::BrowseLuaDir => choose_storage_dir(StorageKind::Lua),
        Message::StorageDirectoryChosen(kind, result) => match result {
            Ok(Some(path)) => {
                match kind {
                    StorageKind::Manifest => state.manifest_dir = path.to_string_lossy().into_owned(),
                    StorageKind::Lua => state.lua_dir = path.to_string_lossy().into_owned(),
                }
                Task::none()
            }
            Ok(None) => Task::none(),
            Err(error) => show_notice(state, error),
        },
        Message::SaveStoragePaths => match archive::save_settings(
            &PathBuf::from(state.manifest_dir.trim()),
            &PathBuf::from(state.lua_dir.trim()),
        ) {
            Ok(settings) => {
                state.manifest_dir = settings.manifest_dir.to_string_lossy().into_owned();
                state.lua_dir = settings.lua_dir.to_string_lossy().into_owned();
                let notice_task = show_notice(
                    state,
                    "Storage folders saved. Existing files stay in their current locations.",
                );
                let refresh_task = start_library_refresh(state);
                Task::batch([notice_task, refresh_task])
            }
            Err(error) => show_notice(state, error),
        },
    }
}

fn retain_expanded_entries(state: &mut State) {
    let current_keys: HashSet<String> = state
        .library
        .entries
        .iter()
        .map(|entry| entry.key.clone())
        .collect();
    state
        .expanded_library_entries
        .retain(|key| current_keys.contains(key));
}

fn start_library_refresh(state: &mut State) -> Task<Message> {
    if state.is_refreshing_library {
        state.library_refresh_pending = true;
        return Task::none();
    }

    let settings = match archive::normalize_and_validate(
        PathBuf::from(state.manifest_dir.trim()),
        PathBuf::from(state.lua_dir.trim()),
    ) {
        Ok(settings) => settings,
        Err(error) => return show_notice(state, error),
    };

    state.manifest_dir = settings.manifest_dir.to_string_lossy().into_owned();
    state.lua_dir = settings.lua_dir.to_string_lossy().into_owned();
    state.is_refreshing_library = true;

    let manifest_dir = settings.manifest_dir;
    let lua_dir = settings.lua_dir;
    Task::perform(
        async move { library::refresh(&manifest_dir, &lua_dir) },
        Message::LibraryRefreshed,
    )
}

fn show_notice(state: &mut State, notice: impl Into<String>) -> Task<Message> {
    state.notice_sequence = state.notice_sequence.wrapping_add(1);
    let token = state.notice_sequence;
    state.notice = Some(notice.into());

    Task::perform(
        async move {
            let (sender, receiver) = iced::futures::channel::oneshot::channel();
            let timer = std::thread::Builder::new()
                .name("locomotive-notice-timer".to_owned())
                .spawn(move || {
                    std::thread::sleep(Duration::from_secs(5));
                    let _ = sender.send(token);
                });
            if timer.is_err() {
                return token;
            }
            receiver.await.unwrap_or(token)
        },
        Message::NoticeExpired,
    )
}

fn clear_notice(state: &mut State) {
    state.notice_sequence = state.notice_sequence.wrapping_add(1);
    state.notice = None;
}

fn choose_storage_dir(kind: StorageKind) -> Task<Message> {
    Task::perform(async { file_dialog::choose_directory() }, move |result| {
        Message::StorageDirectoryChosen(kind, result)
    })
}
