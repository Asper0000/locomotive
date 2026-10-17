//! Brand asset loading.
//!
//! The SVG is loaded from `assets/icon.svg` at compile time and its handle
//! is cached in a `OnceLock`. Every call to `brand_handle` returns a cheap
//! clone of the same handle, which keeps the renderer's cache key stable.
//!
//! Building a fresh handle per `view()` call would give the renderer a new
//! cache key per frame, and the SVG would be re-rasterized continuously.
//! That is the same failure mode that previously pushed idle memory from
//! 6 MB to over 20 MB with the image widget.

use std::sync::OnceLock;

use iced::widget::svg;

static BRAND_HANDLE: OnceLock<svg::Handle> = OnceLock::new();

/// The titlebar brand mark handle. Built once, cached, and cloned cheaply.
pub fn brand_handle() -> svg::Handle {
    BRAND_HANDLE
        .get_or_init(|| {
            let bytes: &'static [u8] = include_bytes!("../assets/icon.svg");
            svg::Handle::from_memory(bytes)
        })
        .clone()
}