use iced::widget::{column, container, text, Space};
use iced::{Alignment, Element, Length};
use crate::message::Message;
use crate::state::{State, View};
use crate::theme;
use crate::views::components;

pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![].spacing(10).align_x(Alignment::Center);

    // Force the text widget to take full width and center its internal lines
    let center_text = |txt: &str, size: f32, color: iced::Color| {
        text(txt.to_string())
            .size(size)
            .color(color)
            .width(Length::Fill)
            .align_x(Alignment::Center)
    };

    if let Some(progress) = state.setup_progress {
        if progress < 1.0 {
            let install_title = state
                .rustytools_latest_version
                .as_deref()
                .map(|v| format!("Installing RustyTools {v}..."))
                .unwrap_or_else(|| "Installing RustyTools (Latest)...".to_owned());
            let status_line = state
                .setup_status
                .clone()
                .unwrap_or_else(|| "Starting".to_owned());
            let percent = format!("{}%", (progress * 100.0) as u32);
            content = content.push(
                components::card(column![
                    center_text(&install_title, 16.0, theme::TEXT_PRIMARY),
                    Space::new().height(Length::Fixed(10.0)),
                    center_text(&status_line, 12.0, theme::TEXT_SECONDARY),
                    Space::new().height(Length::Fixed(4.0)),
                    center_text(&percent, 11.0, theme::TEXT_MUTED),
                    Space::new().height(Length::Fixed(10.0)),
                    iced::widget::progress_bar(0.0..=1.0, progress),
                ].spacing(5).align_x(Alignment::Center).width(Length::Fill))
            );
        } else {
            content = content.push(
                components::card(column![
                    center_text("Installation Complete!", 16.0, theme::SUCCESS),
                    Space::new().height(Length::Fixed(10.0)),
                    center_text("RustyTools has been successfully installed in your Steam directory.", 12.0, theme::TEXT_SECONDARY),
                    Space::new().height(Length::Fixed(20.0)),
                    if state.steam_was_running {
                        column![
                            center_text("Steam is currently running. A restart is required for the changes to take effect.", 12.0, theme::TEXT_PRIMARY),
                            Space::new().height(Length::Fixed(10.0)),
                            components::primary_button("Restart Steam Now", Message::RestartSteam),
                            Space::new().height(Length::Fixed(10.0)),
                            components::action_button("I'll restart it later", Message::Navigate(View::Import)),
                        ].spacing(5).align_x(Alignment::Center).width(Length::Fill)
                    } else {
                        column![
                            center_text("You can now launch Steam and start using RustyTools.", 12.0, theme::TEXT_PRIMARY),
                            Space::new().height(Length::Fixed(10.0)),
                            components::primary_button("Continue to App", Message::Navigate(View::Import)),
                        ].spacing(5).align_x(Alignment::Center).width(Length::Fill)
                    }
                ].spacing(5).align_x(Alignment::Center).width(Length::Fill))
            );
        }
    } else if state.rustytools_installed {
        content = content.push(
            components::card(column![
                center_text("Already Installed", 16.0, theme::SUCCESS),
                Space::new().height(Length::Fixed(10.0)),
                center_text("RustyTools is already installed in your Steam directory.", 12.0, theme::TEXT_SECONDARY),
                Space::new().height(Length::Fixed(20.0)),
                components::action_button("Continue to App", Message::Navigate(View::Import)),
            ].spacing(5).align_x(Alignment::Center).width(Length::Fill))
        );
    } else if let Some(error) = &state.setup_error {
        content = content.push(
            components::card(column![
                center_text("Installation Failed", 16.0, theme::DANGER),
                Space::new().height(Length::Fixed(10.0)),
                center_text(error, 12.0, theme::TEXT_SECONDARY),
                Space::new().height(Length::Fixed(20.0)),
                components::primary_button("Retry Installation", Message::SetupInstall),
            ].spacing(5).align_x(Alignment::Center).width(Length::Fill))
        );
    } else {
        let install_title = state
            .rustytools_latest_version
            .as_deref()
            .map(|v| format!("Install RustyTools {v}"))
            .unwrap_or_else(|| "Install RustyTools (Latest)".to_owned());
        content = content.push(
            components::card(column![
                center_text(&install_title, 16.0, theme::TEXT_PRIMARY),
                Space::new().height(Length::Fixed(10.0)),
                center_text("RustyTools is optional, but necessary for games and DLCs to be genuinely unlocked. Without it, you can still import .manifest and .lua files and manage your library fully.", 12.0, theme::TEXT_SECONDARY),
                Space::new().height(Length::Fixed(20.0)),
                components::primary_button("Download & Install", Message::SetupInstall),
            ].spacing(5).align_x(Alignment::Center).width(Length::Fill))
        );
    }

    let card_container = container(content).width(Length::Fill).max_width(500.0);

    container(card_container)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}