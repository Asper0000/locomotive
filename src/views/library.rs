//! Compact game cards with progressive disclosure for stored manifests and Lua files.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use iced::widget::{column, container, image, row, scrollable, text, Space};
use iced::{Alignment, Border, Element, Length, Padding};

use crate::message::Message;
use crate::state::{State, library_state::LibraryEntry};
use crate::theme;
use crate::views::components;

static COVER_HANDLES: OnceLock<Mutex<HashMap<(std::path::PathBuf, u32, u32), image::Handle>>> = OnceLock::new();

/// Renders the local library and its removal confirmation states.
pub fn view(state: &State) -> Element<'_, Message> {
    let header = components::page_header("Library", 20.0);

    let mut body = column![
        header,
        text("Browse games and expand a card to inspect its stored files.")
            .size(12)
            .color(theme::TEXT_SECONDARY),
        Space::new().height(Length::Fixed(10.0)),
        container(
            row![
                text(format!("{} games", state.library.entries.len()))
                    .size(11)
                    .color(theme::TEXT_MUTED),
                Space::new().width(Length::Fill),
                components::action_button("Refresh", Message::RefreshLibrary),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(Padding { top: 0.0, right: 18.0, bottom: 0.0, left: 0.0 }),
    ]
    .spacing(7);

    if let Some(error) = &state.library.error {
        body = body.push(text(error.clone()).size(11).color(theme::DANGER));
    }
    if let Some(notice) = &state.notice {
        body = body.push(text(notice.clone()).size(11).color(theme::TEXT_SECONDARY));
    }
    body = body.push(Space::new().height(Length::Fixed(5.0)));

    if state.library.entries.is_empty() {
        body = body.push(empty_state());
    } else {
        let mut entries = column![].spacing(8);
        for entry in &state.library.entries {
            let expanded = state.expanded_library_entries.contains(&entry.key);
            let confirm = state.confirm_remove.as_deref() == Some(entry.key.as_str());
            entries = entries.push(entry_card(entry.clone(), expanded, confirm));
        }
        let content = container(entries)
            .width(Length::Fill)
            .padding(Padding { top: 0.0, right: 18.0, bottom: 0.0, left: 0.0 });
        body = body.push(scrollable(content).width(Length::Fill).height(Length::Fill));
    }

    body.into()
}

fn entry_card(entry: LibraryEntry, expanded: bool, confirm: bool) -> Element<'static, Message> {
    let manifest_count = entry.manifest_count();
    let lua_count = entry.lua_count();
    let key = entry.key.clone();

    let title_block = column![
        text(entry.name.clone()).size(14).color(theme::TEXT_PRIMARY),
        components::separator(),
        text(format!("{manifest_count} manifests  ·  {lua_count} Lua files"))
            .size(11)
            .color(theme::TEXT_SECONDARY),
    ]
    .spacing(5)
    .width(Length::Fill);

    let toggle_label = if expanded { "Hide files  ▴" } else { "Show files  ▾" };
    let header = row![
        cover_art(entry.cover_path.clone(), &entry.name),
        title_block,
        column![
            components::action_button(toggle_label, Message::ToggleLibraryEntry(key.clone())),
            if confirm {
                components::danger_button("Delete files", Message::ConfirmRemove)
            } else {
                components::danger_button("Remove", Message::RequestRemove(key.clone()))
            },
        ]
        .spacing(6)
        .align_x(Alignment::End),
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let mut content = column![header].spacing(10);
    if confirm {
        content = content
            .push(components::separator())
            .push(
                text(format!(
                    "Delete the {} locally stored file(s)? This only removes the copies in the configured library folders.",
                    entry.files.len()
                ))
                .size(11)
                .color(theme::TEXT_SECONDARY),
            )
            .push(components::action_button("Cancel", Message::CancelRemove));
    }

    if expanded {
        content = content.push(components::separator());
        if entry.files.is_empty() {
            content = content.push(text("No files found for this game.").size(11).color(theme::TEXT_MUTED));
        } else {
            let mut files = column![].spacing(6);
            for file in &entry.files {
                let file_name = file
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("<unnamed>");
                files = files.push(
                    row![
                        container(text(file.kind.label().to_uppercase()).size(9).color(theme::TEXT_MUTED))
                            .padding(Padding { top: 4.0, right: 6.0, bottom: 4.0, left: 6.0 })
                            .style(|_theme| container::Style {
                                background: Some(theme::BG_ELEVATED.into()),
                                border: Border { color: theme::BORDER_SUBTLE, width: 1.0, radius: 4.0.into() },
                                ..Default::default()
                            }),
                        text(file_name.to_owned()).size(11).color(theme::TEXT_SECONDARY),
                    ]
                    .spacing(9)
                    .align_y(Alignment::Center),
                );
            }
            content = content.push(files);
        }
    }

    container(content)
        .width(Length::Fill)
        .padding(Padding { top: 12.0, right: 16.0, bottom: 12.0, left: 12.0 })
        .style(move |_theme| container::Style {
            background: Some(if confirm { theme::BG_ELEVATED } else { theme::BG_SURFACE }.into()),
            border: Border {
                color: if confirm { theme::TEXT_SECONDARY } else { theme::BORDER },
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

fn cover_art(path: Option<std::path::PathBuf>, name: &str) -> Element<'static, Message> {
    let landscape = path.as_deref().is_some_and(is_landscape_art);
    let (slot_width, slot_height) = if landscape { (102.0, 48.0) } else { (54.0, 72.0) };

    if let Some(path) = path {
        let handle = cached_cover_handle(path, slot_width as u32, slot_height as u32);
        image::Image::new(handle)
            .width(Length::Fixed(slot_width))
            .height(Length::Fixed(slot_height))
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        let initial = name
            .chars()
            .next()
            .map(|character| character.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_owned());
        container(text(initial).size(20).color(theme::TEXT_SECONDARY))
            .width(Length::Fixed(slot_width))
            .height(Length::Fixed(slot_height))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(theme::BG_ELEVATED.into()),
                border: Border {
                    color: theme::BORDER_SUBTLE,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

fn cached_cover_handle(path: std::path::PathBuf, width: u32, height: u32) -> image::Handle {
    let cache = COVER_HANDLES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut handles = cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    
    let key = (path.clone(), width, height);
    if let Some(handle) = handles.get(&key) {
        return handle.clone();
    }
    
    let handle = match ::image::open(&path) {
        Ok(img) => {
            // 1. Smoothly scale the image down to the exact slot size
            let img = img.resize_exact(width, height, ::image::imageops::FilterType::Lanczos3);
            let mut rgba = img.to_rgba8();
            
            // 2. Manually apply the rounded corner mask with Anti-Aliasing
            let fw = width as f32;
            let fh = height as f32;
            let radius = 6.0_f32.min(fw / 2.0).min(fh / 2.0);
            
            for y in 0..height {
                for x in 0..width {
                    let fx = x as f32 + 0.5; // Pixel center X
                    let fy = y as f32 + 0.5; // Pixel center Y
                    
                    let mut alpha_multiplier = 1.0_f32;
                    
                    // Top-Left corner
                    if fx < radius && fy < radius {
                        let dist = ((fx - radius).powi(2) + (fy - radius).powi(2)).sqrt();
                        alpha_multiplier = (radius - dist + 0.5).clamp(0.0, 1.0);
                    } 
                    // Top-Right corner
                    else if fx > fw - radius && fy < radius {
                        let dist = ((fx - (fw - radius)).powi(2) + (fy - radius).powi(2)).sqrt();
                        alpha_multiplier = (radius - dist + 0.5).clamp(0.0, 1.0);
                    } 
                    // Bottom-Left corner
                    else if fx < radius && fy > fh - radius {
                        let dist = ((fx - radius).powi(2) + (fy - (fh - radius)).powi(2)).sqrt();
                        alpha_multiplier = (radius - dist + 0.5).clamp(0.0, 1.0);
                    } 
                    // Bottom-Right corner
                    else if fx > fw - radius && fy > fh - radius {
                        let dist = ((fx - (fw - radius)).powi(2) + (fy - (fh - radius)).powi(2)).sqrt();
                        alpha_multiplier = (radius - dist + 0.5).clamp(0.0, 1.0);
                    }
                    
                    if alpha_multiplier < 1.0 {
                        let pixel = rgba.get_pixel_mut(x, y);
                        // Blend the calculated smooth edge with the existing alpha channel
                        pixel.0[3] = (pixel.0[3] as f32 * alpha_multiplier) as u8;
                    }
                }
            }
            
            image::Handle::from_rgba(width, height, rgba.into_raw())
        }
        Err(_) => {
            // Fallback if image fails to load
            let mut rgba = vec![0u8; (width * height * 4) as usize];
            for i in 0..(width * height) {
                rgba[i as usize * 4] = 30;
                rgba[i as usize * 4 + 1] = 30;
                rgba[i as usize * 4 + 2] = 30;
                rgba[i as usize * 4 + 3] = 255;
            }
            image::Handle::from_rgba(width, height, rgba)
        }
    };
    
    handles.insert(key, handle.clone());
    handle
}



fn is_landscape_art(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().contains("header"))
}

fn empty_state() -> Element<'static, Message> {
    container(
        column![
            text("Nothing here yet").size(14).color(theme::TEXT_SECONDARY),
            components::separator(),
            text("Import manifest and Lua files to populate your library.")
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(220.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(theme::BG_SURFACE.into()),
        border: Border { color: theme::BORDER_SUBTLE, width: 1.0, radius: theme::CORNER_RADIUS.into() },
        ..Default::default()
    })
    .into()
}
