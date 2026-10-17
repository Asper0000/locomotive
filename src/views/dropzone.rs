//! File import screen with native selection, drag-and-drop and a staged-file queue.

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Border, Element, Length, Padding};

use crate::message::Message;
use crate::state::{FileKind, State};
use crate::theme;
use crate::views::components;

/// Renders the import workflow.
pub fn view(state: &State) -> Element<'_, Message> {
    let hint = if state.is_drag_over {
        "Release to add files"
    } else {
        "Drop .manifest, .manifests, and .lua files here"
    };
    let zone = container(
        column![
            text(hint).size(16).color(theme::TEXT_PRIMARY),
            text("Files are copied into the local library folders configured in Options.")
                .size(12)
                .color(theme::TEXT_SECONDARY),
            Space::new().height(Length::Fixed(4.0)),
            components::action_button("Choose files", Message::ChooseFiles),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(170.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(move |_theme| {
        let fill = if state.is_drag_over { theme::BG_HOVER } else { theme::BG_SURFACE };
        let border_color = if state.is_drag_over { theme::TEXT_PRIMARY } else { theme::BORDER };
        container::Style {
            background: Some(fill.into()),
            border: Border {
                color: border_color,
                width: if state.is_drag_over { 2.0 } else { 1.0 },
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    });

    let header = components::page_header("Import files", 20.0);

    let mut body = column![
        header,
        Space::new().height(Length::Fixed(7.0)),
        text("Stage files, check their types, then copy them into your local library.")
            .size(12)
            .color(theme::TEXT_SECONDARY),
        Space::new().height(Length::Fixed(20.0)),
        zone,
    ]
    .spacing(0)
    .width(Length::Fill);

    if let Some(notice) = &state.notice {
        body = body
            .push(Space::new().height(Length::Fixed(12.0)))
            .push(text(notice.clone()).size(11).color(theme::TEXT_SECONDARY));
    }

    if !state.pending_files.is_empty() {
        let actions = row![
            components::primary_button(
                if state.is_importing { "Importing…" } else { "Import to library" },
                Message::ImportPending,
            ),
            components::action_button("Clear list", Message::ClearPending),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        body = body
            .push(Space::new().height(Length::Fixed(18.0)))
            .push(row![
                text(format!("Staged files ({})", state.pending_files.len()))
                    .size(12)
                    .color(theme::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                actions,
            ].align_y(Alignment::Center))
            .push(Space::new().height(Length::Fixed(6.0)))
            .push(components::separator())
            .push(Space::new().height(Length::Fixed(8.0)))
            .push(scrollable(
                container(pending_list(state))
                    .width(Length::Fill)
                    .padding(Padding { top: 0.0, right: 18.0, bottom: 0.0, left: 0.0 }),
            ).width(Length::Fill).height(Length::Fill));
    }

    body.into()
}

fn pending_list(state: &State) -> Element<'_, Message> {
    let mut list = column![].spacing(6);
    for file in &state.pending_files {
        let name = file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unnamed>");
        let kind_color = if file.kind == FileKind::Unknown {
            theme::TEXT_SECONDARY
        } else {
            theme::TEXT_MUTED
        };
        list = list.push(
            container(
                row![
                    column![
                        text(name.to_owned()).size(12).color(theme::TEXT_PRIMARY),
                        text(file.path.parent().map(|path| path.to_string_lossy().into_owned()).unwrap_or_default())
                            .size(10)
                            .color(theme::TEXT_MUTED),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    text(file.kind.label()).size(10).color(kind_color),
                    components::action_button("Remove", Message::RemovePending(file.path.clone())),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding(Padding { top: 8.0, right: 10.0, bottom: 8.0, left: 12.0 })
            .style(|_theme| container::Style {
                background: Some(theme::BG_SURFACE.into()),
                border: Border { color: theme::BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }),
        );
    }
    list.into()
}
