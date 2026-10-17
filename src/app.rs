//! Application construction: boot function, window settings, and the
//! one-time wiring of the update/view/subscription functions into Iced.

use iced::window::{self, settings::platform::CornerPreference};
use iced::{Font, Size};

use crate::state::State;
use crate::theme;
use crate::update;
use crate::views;

/// The user-facing application name. Used in the titlebar and About panel.
pub const APP_NAME: &str = "Locomotive";

/// The user-facing version string. Shown in the About panel.
pub const APP_VERSION: &str = "0.0.1 Alpha";

/// Builds and runs the application.
pub fn run() -> iced::Result {
    iced::application(State::default, update::handle, views::root)
        .title(APP_NAME)
        .theme(theme_for)
        .default_font(Font::with_name("Fira Sans"))
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .subscription(State::subscription)
        .window(window::Settings {
            decorations: false,
            transparent: false,
            resizable: true,
            minimizable: true,
            closeable: true,
            size: Size::new(760.0, 520.0),
            min_size: Some(Size::new(640.0, 440.0)),
            platform_specific: windows_platform(),
            ..window::Settings::default()
        })
        .run()
}

fn theme_for(_state: &State) -> iced::Theme {
    theme::app_theme()
}

fn windows_platform() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        corner_preference: CornerPreference::Round,
        ..Default::default()
    }
}