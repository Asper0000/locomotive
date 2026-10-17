//! Root layout. The window is frameless, so the eight resize handles
//! around the edges are drawn here rather than supplied by the OS.

pub mod about;
pub mod components;
pub mod dropzone;
pub mod library;
pub mod options;
pub mod setup;
pub mod sidebar;
pub mod titlebar;

use iced::widget::{column, container, row, stack};
use iced::window::Direction;
use iced::{Element, Length};

use crate::message::Message;
use crate::state::{State, View};
use crate::theme;

/// Builds the complete window layout.
pub fn root(state: &State) -> Element<'_, Message> {
    let shell = column![titlebar::view(state), body(state)].height(Length::Fill);

    let bordered = column![
        row![
            resize_handle(Direction::NorthWest, Length::Fixed(theme::RESIZE_BORDER), Length::Fixed(theme::RESIZE_BORDER)),
            resize_handle(Direction::North, Length::Fill, Length::Fixed(theme::RESIZE_BORDER)),
            resize_handle(Direction::NorthEast, Length::Fixed(theme::RESIZE_BORDER), Length::Fixed(theme::RESIZE_BORDER)),
        ],
        row![
            resize_handle(Direction::West, Length::Fixed(theme::RESIZE_BORDER), Length::Fill),
            container(shell).width(Length::Fill).height(Length::Fill),
            resize_handle(Direction::East, Length::Fixed(theme::RESIZE_BORDER), Length::Fill),
        ]
        .height(Length::Fill),
        row![
            resize_handle(Direction::SouthWest, Length::Fixed(theme::RESIZE_BORDER), Length::Fixed(theme::RESIZE_BORDER)),
            resize_handle(Direction::South, Length::Fill, Length::Fixed(theme::RESIZE_BORDER)),
            resize_handle(Direction::SouthEast, Length::Fixed(theme::RESIZE_BORDER), Length::Fixed(theme::RESIZE_BORDER)),
        ],
    ];

    let window_content: Element<'_, Message> = container(bordered)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(theme::BG_BASE.into()),
            ..Default::default()
        })
        .into();

    if state.storage_guide_open {
        stack![window_content, options::storage_guide_overlay()]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        window_content
    }
}

fn resize_handle(
    direction: Direction,
    width: Length,
    height: Length,
) -> Element<'static, Message> {
    components::resize_zone(direction, width, height)
}

fn body(state: &State) -> Element<'_, Message> {
    let layout = row![
        sidebar::view(state),
        container(content(state))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(|_theme| container::Style {
                background: Some(theme::BG_BASE.into()),
                ..Default::default()
            }),
    ]
    .height(Length::Fill);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(theme::BG_BASE.into()),
            ..Default::default()
        })
        .into()
}

fn content(state: &State) -> Element<'_, Message> {
    match state.active_view {
        View::Import => dropzone::view(state),
        View::Library => library::view(state),
        View::Options => options::view(state),
        View::Setup => setup::view(state),
        View::About => about::view(state),
    }
}