//! Options screen and the contextual guide for configured storage locations.

use iced::widget::{column, container, mouse_area, row, scrollable, stack, text, text_input, Space};
use iced::{Alignment, Border, Element, Length, Padding};

use crate::message::Message;
use crate::state::State;
use crate::theme;
use crate::views::components;

/// Renders storage configuration and progressive-disclosure controls.
pub fn view(state: &State) -> Element<'_, Message> {
    let intro = row![
        text("Choose where Locomotive stores each file type.")
            .size(12)
            .color(theme::TEXT_SECONDARY)
            .width(Length::Fill),
        components::help_button(Message::OpenStorageGuide),
    ]
    .align_y(Alignment::Center);

    let mut body = column![
        text("Options").size(20).color(theme::TEXT_PRIMARY),
        components::separator(),
        intro,
        Space::new().height(Length::Fixed(2.0)),
        components::card(column![
            text("Manifest storage folder").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("Used for .manifest and .manifests files.")
                .size(11)
                .color(theme::TEXT_MUTED),
            row![
                text_input("Choose a local folder", &state.manifest_dir)
                    .on_input(Message::ManifestDirChanged)
                    .padding(9)
                    .size(12)
                    .width(Length::Fill),
                components::action_button("Browse", Message::BrowseManifestDir),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)),
        components::card(column![
            text("Lua storage folder").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("Used for .lua files.").size(11).color(theme::TEXT_MUTED),
            row![
                text_input("Choose a local folder", &state.lua_dir)
                    .on_input(Message::LuaDirChanged)
                    .padding(9)
                    .size(12)
                    .width(Length::Fill),
                components::action_button("Browse", Message::BrowseLuaDir),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)),
        row![components::primary_button("Save storage paths", Message::SaveStoragePaths)]
            .align_y(Alignment::Center),
        components::card(column![
            components::toggle_row(
                "Advanced options",
                "Reveal additional library details.",
                state.advanced_open,
                Message::ToggleAdvanced,
            ),
            components::separator(),
            text("Keep this closed unless you need the extra behavior notes.")
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8)),
    ]
    .spacing(10);

    if let Some(notice) = &state.notice {
        body = body.push(text(notice.clone()).size(11).color(theme::TEXT_SECONDARY));
    }

    if state.advanced_open {
        body = body.push(components::card(column![
            text("Library behavior").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("Imports are copied, not moved. The library is automatically rescanned when you open it and after storage changes. Removing an entry deletes only the copies represented by that entry in the configured folders.")
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8)));
    }

    let padded_content = container(body)
        .width(Length::Fill)
        .padding(Padding {
            top: 0.0,
            right: 18.0,
            bottom: 0.0,
            left: 0.0,
        });
    let page: Element<'_, Message> = container(
        scrollable(padded_content)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if state.storage_guide_open {
        stack![page, storage_guide_overlay()]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        page
    }
}

fn storage_guide_overlay() -> Element<'static, Message> {
    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(theme::MODAL_BACKDROP.into()),
                ..Default::default()
            }),
    )
    .on_press(Message::StorageGuideBackdropPressed);

    let guide_content = column![
        guide_section(
            "Where files are stored",
            "Manifest files and Lua files use separate folders because they serve different purposes. When Steam is detected, manifests default to <Steam>\\depotcache and Lua files default to <Steam>\\config\\stplug-in. If Steam cannot be detected, Locomotive uses a private library under your local application-data folder, with separate manifests and lua subfolders. You can choose different folders for either file type in Options.",
        ),
        guide_section(
            "Manifest files",
            "Files ending in .manifest or .manifests contain Steam depot content metadata. Locomotive copies them into the Manifest storage folder and reads the depot ID from the filename. If a Lua file or Steam metadata maps that depot to an App ID, the manifest is grouped under the corresponding game; otherwise it remains a separate depot entry until a relationship can be identified.",
        ),
        guide_section(
            "Lua files",
            "Files ending in .lua are copied into the Lua storage folder. Locomotive examines their metadata and filename to identify a Steam App ID and any declared depot IDs. That information is used to group the Lua file with the right manifests and show one compact game entry instead of separate entries for every file.",
        ),
        guide_section(
            "Imports, refreshes, and removal",
            "Importing copies files; it does not move or alter the source files. Saving new storage paths does not automatically move files already stored in the old folders. The Library is rescanned automatically when opened and after storage changes. Removing an entry deletes only the stored copies represented by that entry in your configured library folders.",
        ),
        guide_section(
            "Game names and cover artwork",
            "When an entry has a numeric Steam App ID, Locomotive asks the public Steam Store App Details endpoint for the real game name and its official header image. If the Store endpoint does not return a name, it also tries the public SteamCMD app-info service. For artwork, it first uses the exact image URL returned by Steam, then tries Steam's library-capsule images as a fallback. Names and images are cached locally after lookup, so later launches are faster and cached artwork remains available offline. Some private or unknown App IDs may have no public name or image; those entries keep their filename-derived label and neutral placeholder.",
        ),
    ]
    .spacing(14);

    let panel = container(column![
        row![
            text("Storage guide").size(17).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            components::action_button("Close", Message::CloseStorageGuide),
        ]
        .align_y(Alignment::Center),
        components::separator(),
        scrollable(guide_content)
            .height(Length::Fixed(250.0))
            .width(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            components::primary_button("Got it", Message::CloseStorageGuide),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(12))
    .width(Length::Fixed(580.0))
    .padding(20)
    .style(|_theme| container::Style {
        background: Some(theme::BG_SURFACE.into()),
        border: Border {
            color: theme::BORDER,
            width: 1.0,
            radius: theme::CORNER_RADIUS.into(),
        },
        ..Default::default()
    });

    let centered_panel = container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    stack![backdrop, centered_panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn guide_section(title: &'static str, body: &'static str) -> Element<'static, Message> {
    column![
        text(title).size(13).color(theme::TEXT_PRIMARY),
        components::separator(),
        text(body).size(11).color(theme::TEXT_SECONDARY),
    ]
    .spacing(6)
    .into()
}
