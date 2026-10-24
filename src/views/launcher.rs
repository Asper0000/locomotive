//! Launcher card grid shown when no application workspace is open.
//!
//! Each app is a 2:3 portrait card with its own branded gradient. To add a
//! new app:
//!   1. Add a variant to `LauncherApp` in `state::mod.rs`.
//!   2. Add a `(name, logo_handle, style_fn)` case to `app_card` below.
//!   3. Append the new `LauncherApp` variant to `APPS`.
//! The grid wraps automatically once it exceeds `CARDS_PER_ROW`.

use std::sync::OnceLock;

use iced::gradient::{ColorStop, Gradient, Linear};
use iced::widget::{button, column, container, row, svg, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Radians, Theme};

use crate::message::Message;
use crate::state::{LauncherApp, State};
use crate::theme;

/// Every application shown in the launcher, in display order.
const APPS: &[LauncherApp] = &[LauncherApp::Steam];

const CARD_WIDTH: f32 = 140.0;
const CARD_HEIGHT: f32 = 210.0; // 2:3 aspect ratio
const CARD_GAP: f32 = 14.0;
const CARDS_PER_ROW: usize = 4;
/// Horizontal inset. Matches the titlebar content inset so the first card
/// starts at the same x as the brand mark and app name.
const GRID_PADDING_H: f32 = 14.0;
/// Vertical breathing room above and below the grid.
const GRID_PADDING_V: f32 = 28.0;

/// Embedded Steam logo. Kept as raw bytes so the launcher has no external
/// asset dependency. `fill="white"` so it tints cleanly against the blue card.
const STEAM_LOGO_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="white"><path d="M11.979 0C5.678 0 .511 4.86.022 11.037l6.432 2.658c.545-.371 1.203-.59 1.912-.59.063 0 .125.004.188.006l2.861-4.142V8.91c0-2.495 2.028-4.524 4.524-4.524 2.494 0 4.524 2.031 4.524 4.527s-2.03 4.525-4.524 4.525h-.105l-4.076 2.911c0 .052.004.105.004.159 0 1.875-1.515 3.396-3.39 3.396-1.635 0-3.016-1.173-3.331-2.727L.436 15.27C1.862 20.307 6.486 24 11.979 24c6.627 0 11.999-5.373 11.999-12S18.605 0 11.979 0zM7.54 18.21l-1.473-.61c.262.543.714.999 1.314 1.25 1.297.539 2.793-.076 3.332-1.375.263-.63.264-1.319.005-1.949s-.75-1.121-1.377-1.383c-.624-.26-1.29-.249-1.878-.03l1.523.63c.956.4 1.409 1.5 1.009 2.455-.397.957-1.497 1.41-2.454 1.012H7.54zm11.415-9.303c0-1.662-1.353-3.015-3.015-3.015-1.665 0-3.015 1.353-3.015 3.015 0 1.665 1.35 3.015 3.015 3.015 1.663 0 3.015-1.35 3.015-3.015zm-5.273-.005c0-1.252 1.013-2.266 2.265-2.266 1.249 0 2.266 1.014 2.266 2.266 0 1.251-1.017 2.265-2.266 2.265-1.253 0-2.265-1.014-2.265-2.265z"/></svg>"#;

static STEAM_HANDLE: OnceLock<svg::Handle> = OnceLock::new();

fn steam_logo_handle() -> svg::Handle {
    STEAM_HANDLE
        .get_or_init(|| svg::Handle::from_memory(STEAM_LOGO_SVG))
        .clone()
}

type CardStyleFn = fn(&Theme, button::Status) -> button::Style;

/// Renders the launcher card grid.
pub fn view(_state: &State) -> Element<'static, Message> {
    let mut grid = column![].spacing(CARD_GAP);
    let mut current_row = row![].spacing(CARD_GAP);
    let mut count_in_row = 0;

    for &app in APPS {
        current_row = current_row.push(app_card(app));
        count_in_row += 1;
        if count_in_row == CARDS_PER_ROW {
            grid = grid.push(current_row);
            current_row = row![].spacing(CARD_GAP);
            count_in_row = 0;
        }
    }
    if count_in_row > 0 {
        grid = grid.push(current_row);
    }

    container(
        container(grid)
            .padding(Padding { top: GRID_PADDING_V, right: GRID_PADDING_H, bottom: GRID_PADDING_V, left: GRID_PADDING_H })
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Start)
    .style(|_theme| container::Style {
        background: Some(theme::BG_BASE.into()),
        ..Default::default()
    })
    .into()
}

fn app_card(app: LauncherApp) -> Element<'static, Message> {
    let (name, logo, style): (&'static str, svg::Handle, CardStyleFn) = match app {
        LauncherApp::Steam => ("Steam", steam_logo_handle(), steam_card_style),
    };

    button(
        container(
            column![
                Space::new().height(Length::Fill),
                svg(logo)
                    .width(Length::Fixed(72.0))
                    .height(Length::Fixed(72.0)),
                Space::new().height(Length::Fixed(14.0)),
                text(name).size(14).color(Color::WHITE),
                Space::new().height(Length::Fill),
            ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: 18.0, right: 18.0, bottom: 18.0, left: 18.0 }),
    )
    .width(Length::Fixed(CARD_WIDTH))
    .height(Length::Fixed(CARD_HEIGHT))
    .padding(0)
    .on_press(Message::OpenApp(app))
    .style(style)
    .into()
}

fn steam_card_style(_theme: &Theme, status: button::Status) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let border_color = if hovered {
        Color::from_rgba(1.0, 1.0, 1.0, 0.38)
    } else {
        Color::from_rgba(1.0, 1.0, 1.0, 0.08)
    };
    button::Style {
        background: Some(Background::Gradient(Gradient::Linear(Linear {
            angle: Radians(std::f32::consts::FRAC_PI_2),
            stops: [
                Some(ColorStop {
                    color: Color::from_rgb(0.16, 0.36, 0.62),
                    offset: 0.0,
                }),
                Some(ColorStop {
                    color: Color::from_rgb(0.06, 0.16, 0.30),
                    offset: 1.0,
                }),
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }))),
        text_color: Color::WHITE,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: theme::CORNER_RADIUS.into(),
        },
        ..button::Style::default()
    }
}