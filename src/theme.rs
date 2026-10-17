//! Every color, every metric, and the single theme constructor.
//! Nothing outside this file may hardcode a color. See AGENTS.md section 4.

use std::sync::Arc;

use iced::theme::{Custom, Palette};
use iced::{Color, Theme};

// --- Surfaces ---------------------------------------------------------------

pub const BG_BASE: Color = Color::from_rgb(0.055, 0.055, 0.055); // #0E0E0E
pub const BG_SURFACE: Color = Color::from_rgb(0.078, 0.078, 0.078); // #141414
pub const BG_ELEVATED: Color = Color::from_rgb(0.102, 0.102, 0.102); // #1A1A1A
pub const BG_HOVER: Color = Color::from_rgb(0.125, 0.125, 0.125); // #202020
pub const MODAL_BACKDROP: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.82);

// --- Lines ------------------------------------------------------------------

pub const BORDER: Color = Color::from_rgb(0.145, 0.145, 0.145); // #252525
pub const BORDER_SUBTLE: Color = Color::from_rgb(0.110, 0.110, 0.110); // #1C1C1C

/// Horizontal rule colour. Same value as BORDER_SUBTLE but named for
/// intent so call sites read clearly.
pub const SEPARATOR: Color = BORDER_SUBTLE;

// --- Text -------------------------------------------------------------------

pub const TEXT_PRIMARY: Color = Color::from_rgb(0.867, 0.867, 0.867); // #DDDDDD
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.533, 0.533, 0.533); // #888888
pub const TEXT_MUTED: Color = Color::from_rgb(0.333, 0.333, 0.333); // #555555

// --- Semantic ---------------------------------------------------------------

pub const ACCENT: Color = Color::from_rgb(0.867, 0.867, 0.867); // #DDDDDD
pub const SUCCESS: Color = Color::from_rgb(0.220, 0.557, 0.235); // #388E3C
pub const DANGER: Color = Color::from_rgb(0.706, 0.243, 0.208); // #B43E35

// --- Metrics ----------------------------------------------------------------

pub const TITLEBAR_HEIGHT: f32 = 40.0;
pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const CORNER_RADIUS: f32 = 8.0;
pub const RESIZE_BORDER: f32 = 6.0;

pub const SIDEBAR_LIBRARY_OFFSET: f32 = 12.0;

/// Font size for the Lucide glyphs in the sidebar.
pub const NAV_ICON_SIZE: f32 = 20.0;

/// Horizontal space reserved for a nav icon.
pub const NAV_ICON_SLOT: f32 = 24.0;

/// Font size for the Lucide glyphs in the titlebar.
pub const TITLEBAR_ICON_SIZE: f32 = 16.0;

/// Rendered size of the brand mark in the titlebar.
pub const BRAND_ICON_SIZE: f32 = 18.0;

// --- Theme ------------------------------------------------------------------

pub fn app_theme() -> Theme {
    let palette = Palette {
        background: BG_BASE,
        text: TEXT_PRIMARY,
        primary: ACCENT,
        success: SUCCESS,
        warning: TEXT_SECONDARY,
        danger: DANGER,
    };

    Theme::Custom(Arc::new(Custom::new("Locomotive".to_string(), palette)))
}