//! Native window event handling, including drag-and-drop staging.

use iced::Task;
use iced::window as iced_window;

use crate::message::Message;
use crate::state::{State, View};

/// Handles window events and stages dropped paths for review.
pub fn handle_event(state: &mut State, event: iced_window::Event) -> Task<Message> {
    match event {
        iced_window::Event::FileDropped(path) => {
            state.stage_paths([path]);
            state.is_drag_over = false;
            state.active_view = View::Import;
            Task::none()
        }
        iced_window::Event::FileHovered(_) => {
            state.is_drag_over = true;
            Task::none()
        }
        iced_window::Event::FilesHoveredLeft => {
            state.is_drag_over = false;
            Task::none()
        }
        iced_window::Event::CloseRequested => {
            let mut done = false;
            iced_window::latest().then(move |id| {
                if done {
                    return Task::none();
                }
                done = true;
                match id {
                    Some(id) => iced_window::close(id),
                    None => Task::none(),
                }
            })
        }
        _ => Task::none(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::state::FileKind;

    #[test]
    fn both_manifest_extensions_are_classified() {
        assert_eq!(FileKind::from_path(&PathBuf::from("/tmp/12345_67890.manifest")), FileKind::Manifest);
        assert_eq!(FileKind::from_path(&PathBuf::from("/tmp/12345_67890.manifests")), FileKind::Manifest);
    }

    #[test]
    fn lua_is_classified_case_insensitively() {
        assert_eq!(FileKind::from_path(&PathBuf::from("C:/drops/App.LUA")), FileKind::Lua);
    }

    #[test]
    fn unrelated_extensions_are_not_imported() {
        assert_eq!(FileKind::from_path(&PathBuf::from("readme.txt")), FileKind::Unknown);
    }
}
