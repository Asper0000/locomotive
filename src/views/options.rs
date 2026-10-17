//! Options screen and the contextual guide for configured storage locations.

use iced::widget::{column, container, mouse_area, row, scrollable, stack, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

use crate::message::Message;
use crate::state::State;
use crate::theme;
use crate::views::components;

/// Renders storage configuration and progressive-disclosure controls.
pub fn view(state: &State) -> Element<'_, Message> {
    // --- STATUS BAR ---
    let status_bar = if state.config_exists {
        container(
            row![
                text("Configuration file found. Configuration from software available.")
                    .size(12)
                    .color(Color::WHITE),
            ]
            .padding(10)
        )
        .width(Length::Fill)
        .style(|_theme| container::Style {
            // Darker green background, brighter green border
            background: Some(Color::from_rgb(0.05, 0.15, 0.05).into()),
            border: Border { color: Color::from_rgb(0.2, 0.6, 0.2), width: 1.0, radius: 6.0.into() },
            ..Default::default()
        })
    } else {
        container(
            row![
                text("rustytools.toml not found in Steam root. You must create it before editing paths.")
                    .size(12)
                    .color(Color::WHITE)
                    .width(Length::Fill),
                components::action_button("Create Default Config", Message::CreateDefaultConfig),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .padding(10)
        )
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Color::from_rgb(0.7, 0.2, 0.2).into()), // Red warning
            border: Border { color: Color::from_rgb(0.5, 0.1, 0.1), width: 1.0, radius: 6.0.into() },
            ..Default::default()
        })
    };

    // --- LUA PATH EDITOR ---
    let lua_card_content = if state.config_exists {
        column![
            text("Lua storage").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("RustyTools reads .lua files from this folder to unlock games and apply decryption keys.")
                .size(11)
                .color(theme::TEXT_MUTED),
            row![
                text_input("Choose a local folder or leave blank for default", &state.lua_dir_draft)
                    .on_input(Message::LuaDirDraftChanged)
                    .padding(9)
                    .size(12)
                    .width(Length::Fill),
                components::action_button("Save", Message::SaveLuaDir),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
    } else {
        column![
            text("Lua storage").size(13).color(theme::TEXT_MUTED),
            components::separator(),
            text("Locked. Create the configuration file first.")
                .size(11)
                .color(theme::TEXT_MUTED),
            container(text(&state.lua_dir).size(12).color(theme::TEXT_MUTED))
                .width(Length::Fill)
                .padding(8)
                .style(|_theme| container::Style {
                    background: Some(theme::BG_ELEVATED.into()),
                    border: Border { color: theme::BORDER_SUBTLE, width: 1.0, radius: 4.0.into() },
                    ..Default::default()
                }),
        ]
        .spacing(8)
    };

    let mut body = column![
        components::page_header("Options", 20.0),
        Space::new().height(Length::Fixed(10.0)),
        
        status_bar,
        
        Space::new().height(Length::Fixed(10.0)),

        // --- SECTION 1: STORAGE FOLDERS ---
        row![
            text("RustyTools Folders").size(14).color(theme::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            components::help_button(Message::OpenStorageGuide),
        ]
        .align_y(Alignment::Center),
        
        text("These are the folders where RustyTools reads its configuration and manifests.")
            .size(12)
            .color(theme::TEXT_SECONDARY),
            
        Space::new().height(Length::Fixed(2.0)),
        
        // Manifest card (Read Only)
        components::card(column![
            text("Manifest storage").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("Steam reads .manifest files from this folder to download game files.")
                .size(11)
                .color(theme::TEXT_MUTED),
            container(text(&state.manifest_dir).size(12).color(theme::TEXT_SECONDARY))
                .width(Length::Fill)
                .padding(8)
                .style(|_theme| container::Style {
                    background: Some(theme::BG_ELEVATED.into()),
                    border: Border { color: theme::BORDER_SUBTLE, width: 1.0, radius: 4.0.into() },
                    ..Default::default()
                }),
        ]
        .spacing(8)),
        
        // Lua card (Editable if config exists)
        components::card(lua_card_content),
        
        Space::new().height(Length::Fixed(15.0)),
        
        // --- SECTION 2: DIAGNOSTICS ---
        text("Diagnostics").size(14).color(theme::TEXT_PRIMARY),
        text("Advanced settings and internal behavior notes.")
            .size(12)
            .color(theme::TEXT_SECONDARY),
            
        Space::new().height(Length::Fixed(2.0)),
        
        components::card(column![
            components::toggle_row(
                "Advanced details",
                "Reveal additional information about how files are processed.",
                state.advanced_open,
                Message::ToggleAdvanced,
            ),
            components::separator(),
            text("Keep this closed unless you need the extra details.")
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
            text("How it works").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("Files are imported to the paths above. RustyTools reads them automatically when Steam starts. If you add or remove files while Steam is running, you must restart Steam for the changes to take effect.")
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8)));
    }

    if state.rustytools_installed {
        body = body.push(Space::new().height(Length::Fixed(25.0)));
        body = body.push(
            components::card(column![
                text("Danger Zone").size(14).color(theme::DANGER),
                components::separator(),
                text("Removing RustyTools will disable game unlocking until you reinstall it. You must close Steam before removing.")
                    .size(11).color(theme::TEXT_MUTED),
                Space::new().height(Length::Fixed(10.0)),
                components::danger_button("Remove RustyTools DLLs", Message::RemoveRustyTools),
            ].spacing(8))
        );
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

    page
}

pub fn storage_guide_overlay() -> Element<'static, Message> {
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
        guide_note(
            "The basics",
            "RustyTools works by combining Lua configuration files with Steam manifest files. This tricks Steam into downloading and launching games you don't own.",
        ),
        guide_section(
            "1. The two storage locations",
            "RustyTools and Steam read from these folders automatically. You do not need to change them.",
        ),
        row![
            guide_path_card(
                "Lua files",
                "<Steam>\\config\\stplug-in",
                "Unlocks games and provides decryption keys.",
            ),
            guide_path_card(
                "Manifest files",
                "<Steam>\\depotcache",
                "Tells Steam exactly what files to download.",
            ),
        ]
        .spacing(8),
        guide_file_group(),
        // Removed guide_action_group() from here
    ]
    .spacing(12);

    let panel = container(
        column![
            row![
                column![
                    text("Storage guide").size(17).color(theme::TEXT_PRIMARY),
                    text("How RustyTools uses your files.").size(10).color(theme::TEXT_MUTED),
                ]
                .spacing(3),
                Space::new().width(Length::Fill),
                components::action_button("Close", Message::CloseStorageGuide),
            ]
            .align_y(Alignment::Center),
            components::separator(),
            scrollable(guide_content)
                .height(Length::Fill)
                .width(Length::Fill),
            row![
                text("You can reopen this guide from Options at any time.")
                    .size(10)
                    .color(theme::TEXT_MUTED)
                    .width(Length::Fill),
                components::primary_button_sized(
                    "Got it",
                    Message::CloseStorageGuide,
                    112.0,
                    38.0,
                ),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .max_width(660.0)
    .height(Length::Fixed(500.0))
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
        .padding(12)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    stack![backdrop, centered_panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn guide_section(title: &'static str, body: &'static str) -> Element<'static, Message> {
    container(
        column![
            text(title).size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text(body).size(12).color(theme::TEXT_SECONDARY),
        ]
        .spacing(6),
    )
    .width(Length::Fill)
    .padding(10)
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

fn guide_file_group() -> Element<'static, Message> {
    container(
        column![
            text("2. How the files work together").size(13).color(theme::TEXT_PRIMARY),
            components::separator(),
            text("RustyTools intercepts Steam's security checks using the data inside these files.")
                .size(12)
                .color(theme::TEXT_SECONDARY),
            guide_action(
                "Lua files",
                "Provide the App ID (the game), Depot IDs (the pieces of the game), and the AES decryption keys required to read the files.",
            ),
            components::separator(),
            guide_action(
                "Manifest files",
                "Provide the exact file structure. Steam reads these to know which files to download and verify from Valve's servers.",
            ),
            components::separator(),
            guide_action(
                "Why both?",
                "The Lua file tells Steam you own the game. The Manifest file tells Steam how to download it. Without both, the installation will fail.",
            ),
        ]
        .spacing(7),
    )
    .width(Length::Fill)
    .padding(10)
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

// Removed guide_action_group() function entirely

fn guide_action(label: &'static str, detail: &'static str) -> Element<'static, Message> {
    row![
        text(label).size(12).color(theme::TEXT_PRIMARY).width(Length::Fixed(96.0)),
        text(detail)
            .size(12)
            .color(theme::TEXT_SECONDARY)
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Start)
    .into()
}

fn guide_path_card(
    title: &'static str,
    path: &'static str,
    detail: &'static str,
) -> Element<'static, Message> {
    container(
        column![
            text(title).size(12).color(theme::TEXT_PRIMARY),
            text(path).size(10).color(theme::TEXT_SECONDARY),
            text(detail).size(10).color(theme::TEXT_MUTED),
        ]
        .spacing(5),
    )
    .width(Length::Fill)
    .padding(10)
    .style(|_theme| container::Style {
        background: Some(theme::BG_ELEVATED.into()),
        border: Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn guide_note(title: &'static str, body: &'static str) -> Element<'static, Message> {
    container(
        column![
            text(title).size(12).color(theme::TEXT_PRIMARY),
            text(body).size(11).color(theme::TEXT_SECONDARY),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .padding(10)
    .style(|_theme| container::Style {
        background: Some(theme::BG_SURFACE.into()),
        border: Border {
            color: theme::BORDER_SUBTLE,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}