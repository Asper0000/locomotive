//! Small, reusable widgets. Anything used on more than one screen belongs
//! here rather than being duplicated into a view module.

use iced::mouse;
use iced::widget::{button, container, mouse_area, row, svg, text, Space};
use iced::window::Direction;
use iced::{Alignment, Border, Color, Element, Length, Padding, Theme};

use crate::assets;
use crate::message::Message;
use crate::state::View;
use crate::theme;

/// An invisible hit area that starts a native window resize when pressed.
/// The cursor changes to the appropriate resize arrow while the pointer is
/// over the zone, so the enlarged 10px border is discoverable.
pub fn resize_zone(
    direction: Direction,
    width: Length,
    height: Length,
) -> Element<'static, Message> {
    let interaction = match direction {
        Direction::North | Direction::South => mouse::Interaction::ResizingVertically,
        Direction::East | Direction::West => mouse::Interaction::ResizingHorizontally,
        Direction::NorthWest | Direction::SouthEast => mouse::Interaction::ResizingDiagonallyDown,
        Direction::NorthEast | Direction::SouthWest => mouse::Interaction::ResizingDiagonallyUp,
    };
    mouse_area(Space::new().width(width).height(height))
        .interaction(interaction)
        .on_press(Message::Resize(direction))
        .into()
}

/// A one-pixel horizontal rule.
pub fn separator() -> Element<'static, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(theme::SEPARATOR.into()),
            ..Default::default()
        })
        .into()
}

/// A window control in the titlebar.
pub fn titlebar_button(
    glyph: Element<'static, Message>,
    message: Message,
    danger: bool,
    round_top_right: bool,
) -> Element<'static, Message> {
    button(
        container(glyph)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fixed(46.0))
    .height(Length::Fixed(theme::TITLEBAR_HEIGHT))
    .padding(0)
    .on_press(message)
    .style(move |_theme: &Theme, status| {
        let hover = if danger { theme::DANGER } else { theme::BG_HOVER };
        let background = match status {
            button::Status::Hovered => hover,
            button::Status::Pressed => theme::BG_HOVER,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(background.into()),
            text_color: theme::TEXT_PRIMARY,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: iced::border::Radius {
                    top_left: 0.0,
                    top_right: if round_top_right { theme::CORNER_RADIUS } else { 0.0 },
                    bottom_left: 0.0,
                    bottom_right: 0.0,
                },
            },
            ..button::Style::default()
        }
    })
    .into()
}

/// A sidebar navigation item with a Lucide icon on the left.
pub fn nav_button(
    icon: Element<'static, Message>,
    label: &'static str,
    view: View,
    active: bool,
) -> Element<'static, Message> {
    let color = if active {
        theme::TEXT_PRIMARY
    } else {
        theme::TEXT_SECONDARY
    };

    button(
        row![
            container(icon)
                .width(Length::Fixed(theme::NAV_ICON_SLOT))
                .align_y(Alignment::Center),
            text(label).size(13).color(color),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Padding {
        top: 8.0,
        right: 12.0,
        bottom: 8.0,
        left: 10.0,
    })
    .on_press(Message::Navigate(view))
    .style(move |_theme: &Theme, status| {
        let background = if active {
            theme::BG_ELEVATED
        } else {
            match status {
                button::Status::Hovered => theme::BG_SURFACE,
                _ => Color::TRANSPARENT,
            }
        };
        button::Style {
            background: Some(background.into()),
            text_color: color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}


/// A consistent page title and divider. The subtitle belongs immediately below this block.
pub fn page_header(title: &'static str, size: f32) -> Element<'static, Message> {
    iced::widget::column![
        text(title).size(size).color(theme::TEXT_PRIMARY),
        separator(),
    ]
    .spacing(10)
    .into()
}

/// A small all-caps label used to separate groups in the sidebar.
#[allow(dead_code)]
pub fn section_label(label: &'static str) -> Element<'static, Message> {
    container(text(label).size(10).color(theme::TEXT_MUTED))
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 10.0,
        })
        .into()
}

/// A destructive action. Muted at rest, red only on hover.
pub fn danger_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding(Padding {
            top: 6.0,
            right: 14.0,
            bottom: 6.0,
            left: 14.0,
        })
        .on_press(message)
        .style(|_theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some(
                    if hovered { theme::DANGER } else { theme::BG_SURFACE }.into(),
                ),
                text_color: if hovered {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_MUTED
                },
                border: Border {
                    color: if hovered { theme::DANGER } else { theme::BORDER },
                    width: 1.0,
                    radius: theme::CORNER_RADIUS.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}


/// A compact square icon button. Used where a full text label would be
/// too noisy but the action still needs to be reachable and discoverable.
/// `size` is the edge length of the square, so caller decides between a
/// compact 30 px control and a larger 36/40 px one.
pub fn icon_button(
    icon: impl Into<Element<'static, Message>>,
    message: Message,
    size: f32,
) -> Element<'static, Message> {
    let icon: Element<'static, Message> = icon.into();
    button(
        container(icon)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .padding(0)
    .on_press(message)
    .style(|_theme: &Theme, status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some((if hovered { theme::BG_HOVER } else { theme::BG_SURFACE }).into()),
            text_color: theme::TEXT_PRIMARY,
            border: Border {
                color: if hovered { theme::TEXT_SECONDARY } else { theme::BORDER },
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}
/// A compact square help control for contextual guidance that should stay out of the way.
pub fn help_button(message: Message) -> Element<'static, Message> {
    button(
        container(
            text("?")
                .size(14)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0)
        .on_press(message)
        .style(|_theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some((if hovered { theme::BG_HOVER } else { theme::BG_SURFACE }).into()),
                text_color: theme::TEXT_PRIMARY,
                border: Border {
                    color: if hovered { theme::TEXT_SECONDARY } else { theme::BORDER },
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

/// A standard action button with the existing monochrome interaction styling.
pub fn action_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding(Padding {
            top: 7.0,
            right: 14.0,
            bottom: 7.0,
            left: 14.0,
        })
        .on_press(message)
        .style(|_theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some((if hovered { theme::BG_HOVER } else { theme::BG_SURFACE }).into()),
                text_color: theme::TEXT_PRIMARY,
                border: Border {
                    color: if hovered { theme::TEXT_SECONDARY } else { theme::BORDER },
                    width: 1.0,
                    radius: theme::CORNER_RADIUS.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

/// A high-contrast action button used for the primary action on a screen.
pub fn primary_button(label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding(Padding {
            top: 7.0,
            right: 16.0,
            bottom: 7.0,
            left: 16.0,
        })
        .on_press(message)
        .style(|_theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: Some((if hovered { theme::TEXT_PRIMARY } else { theme::ACCENT }).into()),
                text_color: theme::BG_BASE,
                border: Border {
                    color: theme::ACCENT,
                    width: 1.0,
                    radius: theme::CORNER_RADIUS.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

/// A primary action with a predictable width, useful in modal footers.
pub fn primary_button_sized(
    label: &'static str,
    message: Message,
    width: f32,
    height: f32,
) -> Element<'static, Message> {
    button(
        container(text(label).size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .padding(0)
    .on_press(message)
    .style(|_theme: &Theme, status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some((if hovered { theme::TEXT_PRIMARY } else { theme::ACCENT }).into()),
            text_color: theme::BG_BASE,
            border: Border {
                color: theme::ACCENT,
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..button::Style::default()
        }
    })
    .into()
}

/// A bordered panel used to group related controls.
pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .padding(16)
        .style(|_theme| container::Style {
            background: Some(theme::BG_SURFACE.into()),
            border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

/// A label on the left and an on/off control on the right.
pub fn toggle_row(
    label: &'static str,
    description: &'static str,
    value: bool,
    message: Message,
) -> Element<'static, Message> {
    let text_block = iced::widget::column![
        text(label).size(13).color(theme::TEXT_PRIMARY),
        text(description).size(11).color(theme::TEXT_MUTED),
    ]
    .spacing(2);

    let state_label = if value { "On" } else { "Off" };

    let toggle: Element<'static, Message> = button(text(state_label).size(11))
        .padding(Padding {
            top: 4.0,
            right: 14.0,
            bottom: 4.0,
            left: 14.0,
        })
        .on_press(message)
        .style(move |_theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            let background = if value {
                theme::SUCCESS
            } else if hovered {
                theme::BG_HOVER
            } else {
                theme::BG_SURFACE
            };
            let border_color = if value { theme::SUCCESS } else { theme::BORDER };
            button::Style {
                background: Some(background.into()),
                text_color: if value || hovered {
                    theme::TEXT_PRIMARY
                } else {
                    theme::TEXT_MUTED
                },
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..button::Style::default()
            }
        })
        .into();

    row![text_block, Space::new().width(Length::Fill), toggle]
        .align_y(Alignment::Center)
        .into()
}

/// The brand mark in the titlebar, loaded from `assets/icon.svg`.
pub fn brand_icon() -> Element<'static, Message> {
    svg(assets::brand_handle())
        .width(Length::Fixed(theme::BRAND_ICON_SIZE))
        .height(Length::Fixed(theme::BRAND_ICON_SIZE))
        .into()
}