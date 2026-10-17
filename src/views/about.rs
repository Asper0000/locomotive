//! About panel. Shows the application name, version, and a short
//! description of what the tool does.

use iced::widget::{column, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::app::{APP_NAME, APP_VERSION};
use crate::message::Message;
use crate::state::State;
use crate::theme;
use crate::views::components;

/// Renders application details and build information.
pub fn view(_state: &State) -> Element<'_, Message> {
    let header = column![
        text(APP_NAME).size(24).color(theme::TEXT_PRIMARY),
        text(format!("Version {}", APP_VERSION))
            .size(12)
            .color(theme::TEXT_SECONDARY),
        components::separator(),
    ]
    .spacing(4);

    let description = components::card(
        column![
            text("About").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            Space::new().height(Length::Fixed(6.0)),
            text(
                "Locomotive is a cross-platform file manager for Steam depot \
                 manifests and Lua files. Import or drop files into the \
                 configured storage folders, browse grouped entries, and remove \
                 only the manifest and Lua files represented by an entry. It \
                 does not uninstall games or change account entitlements."
                    .to_string(),
            )
            .size(12)
            .color(theme::TEXT_SECONDARY),
        ]
        .spacing(0),
    );

    let build_info = components::card(
        column![
            info_row("Version", APP_VERSION),
            Space::new().height(Length::Fixed(8.0)),
            info_row("Channel", "Alpha"),
            Space::new().height(Length::Fixed(8.0)),
            info_row("Toolkit", "Iced 0.14"),
            Space::new().height(Length::Fixed(8.0)),
            info_row("Renderer", "tiny-skia (CPU)"),
            Space::new().height(Length::Fixed(8.0)),
            info_row("Language", "Rust (edition 2024)"),
        ]
        .spacing(0),
    );

    column![
        header,
        Space::new().height(Length::Fixed(24.0)),
        description,
        Space::new().height(Length::Fixed(12.0)),
        build_info,
        Space::new().height(Length::Fill),
    ]
    .into()
}

/// One label/value pair inside the build info card.
fn info_row(label: &'static str, value: &str) -> Element<'static, Message> {
    row![
        text(label)
            .size(12)
            .color(theme::TEXT_MUTED)
            .width(Length::Fixed(90.0)),
        text(value.to_string()).size(12).color(theme::TEXT_PRIMARY),
    ]
    .align_y(Alignment::Center)
    .into()
}