// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Transfers Page (History)

use cosmic::iced::Length;
use cosmic::widget::{self, button, container, text};
use cosmic::{theme, Element, Task};
use gosh_lan_transfer::TransferRecord;

#[derive(Debug, Clone)]
pub enum TransfersMessage {
    ClearHistory,
    #[allow(dead_code)] // Will be used for async clear confirmation
    HistoryCleared,
}

pub struct TransfersPage {
    history: Vec<TransferRecord>,
}

impl TransfersPage {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn view(&self) -> Element<'_, TransfersMessage> {
        let spacing = theme::active().cosmic().spacing;

        // Header
        let header = widget::row()
            .push(text::title3("Transfer History"))
            .push(widget::horizontal_space())
            .push(button::destructive("Clear History").on_press(TransfersMessage::ClearHistory));

        // History content
        let history_content: Element<TransfersMessage> = if self.history.is_empty() {
            container(
                widget::column()
                    .push(widget::icon::from_name("folder-download-symbolic").size(64))
                    .push(text::title4("No transfer history"))
                    .push(text::body("Completed transfers will appear here"))
                    .spacing(spacing.space_m)
                    .align_x(cosmic::iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(spacing.space_l)
            .into()
        } else {
            let mut history_column = widget::column().spacing(spacing.space_xs);

            for record in &self.history {
                let direction_icon = match record.direction {
                    gosh_lan_transfer::TransferDirection::Sent => "document-send-symbolic",
                    gosh_lan_transfer::TransferDirection::Received => "document-save-symbolic",
                };

                let status_text = match record.status {
                    gosh_lan_transfer::TransferStatus::Completed => "Completed",
                    gosh_lan_transfer::TransferStatus::Failed => "Failed",
                    _ => "Unknown",
                };

                let row = widget::row()
                    .push(widget::icon::from_name(direction_icon))
                    .push(
                        widget::column()
                            .push(text::body(&record.peer_address))
                            .push(text::caption(format!("{} files", record.files.len())))
                            .spacing(spacing.space_xxs),
                    )
                    .push(widget::horizontal_space())
                    .push(text::caption(status_text))
                    .spacing(spacing.space_s);

                let card = container(row)
                    .padding(spacing.space_s)
                    .class(theme::Container::Card);

                history_column = history_column.push(card);
            }

            widget::scrollable(history_column).into()
        };

        widget::column()
            .push(header)
            .push(history_content)
            .spacing(spacing.space_m)
            .padding(spacing.space_m)
            .into()
    }

    pub fn update(&mut self, message: TransfersMessage) -> Task<TransfersMessage> {
        match message {
            TransfersMessage::ClearHistory => {
                self.history.clear();
                Task::none()
            }
            TransfersMessage::HistoryCleared => Task::none(),
        }
    }
}
