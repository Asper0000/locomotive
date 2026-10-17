//! The custom frameless titlebar. Since the window is declared with
//! `decorations: false`, this file is responsible for the brand, the
//! window-drag region, and minimize / maximize / close.

use iced::widget::{container, mouse_area, row, text, Space};
use iced::{Alignment, Border, Element, Length};

use crate::app::APP_NAME;
use crate::message::Message;
use crate::state::State;
use crate::theme;
use crate::views::components;

/// Renders the custom titlebar and native window controls.
pub fn view(_state: &State) -> Element<'_, Message> {
    let drag_region = mouse_area(
        container(
            row![
                components::brand_icon(),
                text(APP_NAME).size(13).color(theme::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .padding(iced::Padding {
            top: 0.0,
            right: 14.0,
            bottom: 0.0,
            left: 14.0,
        }),
    )
    .on_press(Message::StartDrag);

    // Only the rightmost button rounds its top-right corner, so its hover
    // fill follows the window frame's rounding.
    let controls = row![
        components::titlebar_button(
            lucide_icons::iced::icon_minus()
                .size(theme::TITLEBAR_ICON_SIZE)
                .into(),
            Message::Minimize,
            false,
            false,
        ),
        components::titlebar_button(
            lucide_icons::iced::icon_square()
                .size(theme::TITLEBAR_ICON_SIZE)
                .into(),
            Message::ToggleMaximize,
            false,
            false,
        ),
        components::titlebar_button(
            lucide_icons::iced::icon_x()
                .size(theme::TITLEBAR_ICON_SIZE)
                .into(),
            Message::Close,
            true,
            true,
        ),
    ];

    container(row![drag_region, controls].align_y(Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fixed(theme::TITLEBAR_HEIGHT))
        .style(|_theme| container::Style {
            background: Some(theme::BG_ELEVATED.into()),
            border: Border {
                color: theme::BORDER_SUBTLE,
                width: 1.0,
                radius: iced::border::Radius {
                    top_left: theme::CORNER_RADIUS,
                    top_right: theme::CORNER_RADIUS,
                    bottom_left: 0.0,
                    bottom_right: 0.0,
                },
            },
            ..Default::default()
        })
        .into()
}