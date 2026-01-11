// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - About Page

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{theme, Element, Task};

const GITHUB_URL: &str = "https://github.com/goshtransfer/gosh-transfer-linux";
const ISSUES_URL: &str = "https://github.com/goshtransfer/gosh-transfer-linux/issues";

#[derive(Debug, Clone)]
pub enum AboutMessage {
    OpenWebsite,
    OpenIssues,
}

pub struct AboutPage {}

impl AboutPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> Element<'_, AboutMessage> {
        let spacing = theme::active().cosmic().spacing;

        // Link buttons
        let link_buttons = widget::row()
            .push(button::text("Website").on_press(AboutMessage::OpenWebsite))
            .push(button::text("Report Issue").on_press(AboutMessage::OpenIssues))
            .spacing(spacing.space_s);

        container(
            widget::column()
                .push(widget::icon::from_name("folder-download-symbolic").size(128))
                .push(text::title1("Gosh Transfer"))
                .push(text::body(format!("Version {}", env!("CARGO_PKG_VERSION"))))
                .push(text::body("A clean, explicit file transfer application."))
                .push(text::body("No cloud. No sync. Just transfer."))
                .push(link_buttons)
                .push(text::caption("Licensed under AGPL-3.0"))
                .push(text::caption("Copyright (c) 2024 Gosh Contributors"))
                .spacing(spacing.space_m)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(cosmic::iced::alignment::Horizontal::Center)
        .align_y(cosmic::iced::alignment::Vertical::Center)
        .into()
    }

    pub fn update(&mut self, message: AboutMessage) -> Task<AboutMessage> {
        match message {
            AboutMessage::OpenWebsite => {
                let _ = open::that(GITHUB_URL);
                Task::none()
            }
            AboutMessage::OpenIssues => {
                let _ = open::that(ISSUES_URL);
                Task::none()
            }
        }
    }
}
