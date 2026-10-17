//! Left navigation rail. Deliberately sparse: four destinations.

use iced::widget::{column, container, Space};
use iced::{Border, Element, Length, Padding};

use crate::message::Message;
use crate::state::{State, View};
use crate::theme;
use crate::views::components;

/// Renders the primary navigation rail.
pub fn view(state: &State) -> Element<'_, Message> {
    let icon = |glyph: iced::widget::Text<'static, iced::Theme, iced::Renderer>| {
        glyph.size(theme::NAV_ICON_SIZE).into()
    };

    let mut nav = column![
        components::section_label("Workspace"),
        Space::new().height(Length::Fixed(6.0)),
        components::separator(),
        Space::new().height(Length::Fixed(theme::SIDEBAR_LIBRARY_OFFSET)),
    ].spacing(4);

    if !state.rustytools_installed {
        nav = nav.push(components::nav_button(
            icon(lucide_icons::iced::icon_download()),
            "Setup",
            View::Setup,
            state.active_view == View::Setup,
        ));
    }

    nav = nav.push(components::nav_button(
        icon(lucide_icons::iced::icon_folder_input()),
        "Import files",
        View::Import,
        state.active_view == View::Import,
    ));
    nav = nav.push(components::nav_button(
        icon(lucide_icons::iced::icon_library()),
        "Library",
        View::Library,
        state.active_view == View::Library,
    ));
    nav = nav.push(components::nav_button(
        icon(lucide_icons::iced::icon_settings()),
        "Options",
        View::Options,
        state.active_view == View::Options,
    ));
    nav = nav.push(components::nav_button(
        icon(lucide_icons::iced::icon_info()),
        "About",
        View::About,
        state.active_view == View::About,
    ));

    container(
        column![nav, Space::new().height(Length::Fill)]
            .height(Length::Fill),
    )
    .width(Length::Fixed(theme::SIDEBAR_WIDTH))
    .height(Length::Fill)
    .padding(Padding {
        top: 16.0,
        right: 12.0,
        bottom: 16.0,
        left: 12.0,
    })
    .style(|_theme| container::Style {
        background: Some(theme::BG_SURFACE.into()),
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_left: theme::CORNER_RADIUS,
                bottom_right: theme::CORNER_RADIUS,
            },
        },
        ..Default::default()
    })
    .into()
}