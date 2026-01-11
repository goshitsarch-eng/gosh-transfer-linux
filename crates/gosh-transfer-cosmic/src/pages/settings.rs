// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Settings Page

use crate::config::CosmicConfig;
use cosmic::widget::{self, button, container, text, text_input, toggler};
use cosmic::{theme, Element, Task};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    DeviceNameChanged(String),
    DownloadDirChanged(String),
    ReceiveOnlyToggled(bool),
    NotificationsToggled(bool),
    ThemeChanged(String),
    SaveSettings,
    #[allow(dead_code)] // Will be used for async save confirmation
    SettingsSaved,
    // Trusted hosts
    TrustedHostInputChanged(String),
    AddTrustedHost,
    RemoveTrustedHost(usize),
}

pub struct SettingsPage {
    device_name: String,
    download_dir: String,
    receive_only: bool,
    notifications_enabled: bool,
    theme: String,
    is_dirty: bool,
    // Trusted hosts
    trusted_hosts: Vec<String>,
    trusted_host_input: String,
}

impl SettingsPage {
    pub fn new(config: &CosmicConfig) -> Self {
        Self {
            device_name: config.device_name.clone(),
            download_dir: config.download_dir.to_string_lossy().to_string(),
            receive_only: config.receive_only,
            notifications_enabled: config.notifications_enabled,
            theme: config.theme.clone(),
            is_dirty: false,
            trusted_hosts: config.trusted_hosts.clone(),
            trusted_host_input: String::new(),
        }
    }

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let spacing = theme::active().cosmic().spacing;

        // Header
        let header = text::title3("Settings");

        // Device section
        let device_section = container(
            widget::column()
                .push(text::title4("Device"))
                .push(
                    widget::column()
                        .push(text::body("Device Name"))
                        .push(
                            text_input("Device name", &self.device_name)
                                .on_input(SettingsMessage::DeviceNameChanged),
                        )
                        .spacing(spacing.space_xxs),
                )
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Transfer section
        let transfer_section = container(
            widget::column()
                .push(text::title4("Transfers"))
                .push(
                    widget::column()
                        .push(text::body("Download Directory"))
                        .push(
                            text_input("Download directory", &self.download_dir)
                                .on_input(SettingsMessage::DownloadDirChanged),
                        )
                        .spacing(spacing.space_xxs),
                )
                .push(
                    widget::row()
                        .push(text::body("Receive Only Mode"))
                        .push(widget::horizontal_space())
                        .push(toggler(self.receive_only).on_toggle(SettingsMessage::ReceiveOnlyToggled)),
                )
                .push(
                    widget::row()
                        .push(text::body("Show Notifications"))
                        .push(widget::horizontal_space())
                        .push(
                            toggler(self.notifications_enabled)
                                .on_toggle(SettingsMessage::NotificationsToggled),
                        ),
                )
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Appearance section
        let appearance_section = container(
            widget::column()
                .push(text::title4("Appearance"))
                .push(
                    widget::row()
                        .push(
                            button::text("System")
                                .on_press(SettingsMessage::ThemeChanged("system".to_string())),
                        )
                        .push(
                            button::text("Light")
                                .on_press(SettingsMessage::ThemeChanged("light".to_string())),
                        )
                        .push(
                            button::text("Dark")
                                .on_press(SettingsMessage::ThemeChanged("dark".to_string())),
                        )
                        .spacing(spacing.space_xs),
                )
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Trusted hosts section
        let mut trusted_hosts_list = widget::column().spacing(spacing.space_xs);

        if self.trusted_hosts.is_empty() {
            trusted_hosts_list = trusted_hosts_list.push(text::caption("No trusted hosts configured"));
        } else {
            for (idx, host) in self.trusted_hosts.iter().enumerate() {
                let host_row = widget::row()
                    .push(text::body(host))
                    .push(widget::horizontal_space())
                    .push(
                        button::icon(widget::icon::from_name("user-trash-symbolic"))
                            .on_press(SettingsMessage::RemoveTrustedHost(idx)),
                    )
                    .align_y(cosmic::iced::Alignment::Center)
                    .spacing(spacing.space_s);
                trusted_hosts_list = trusted_hosts_list.push(host_row);
            }
        }

        let add_host_button = if !self.trusted_host_input.is_empty() {
            button::text("Add").on_press(SettingsMessage::AddTrustedHost)
        } else {
            button::text("Add")
        };

        let trusted_hosts_section = container(
            widget::column()
                .push(text::title4("Trusted Hosts"))
                .push(text::caption("Transfers from these hosts are auto-accepted"))
                .push(trusted_hosts_list)
                .push(
                    widget::row()
                        .push(
                            text_input("Add trusted host...", &self.trusted_host_input)
                                .on_input(SettingsMessage::TrustedHostInputChanged),
                        )
                        .push(add_host_button)
                        .spacing(spacing.space_s),
                )
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Save button
        let save_button = if self.is_dirty {
            button::suggested("Save Settings").on_press(SettingsMessage::SaveSettings)
        } else {
            button::suggested("Save Settings")
        };

        widget::scrollable(
            widget::column()
                .push(header)
                .push(device_section)
                .push(transfer_section)
                .push(appearance_section)
                .push(trusted_hosts_section)
                .push(save_button)
                .spacing(spacing.space_m)
                .padding(spacing.space_m),
        )
        .into()
    }

    pub fn update(
        &mut self,
        message: SettingsMessage,
        config: &mut CosmicConfig,
    ) -> Task<SettingsMessage> {
        match message {
            SettingsMessage::DeviceNameChanged(name) => {
                self.device_name = name;
                self.is_dirty = true;
                Task::none()
            }
            SettingsMessage::DownloadDirChanged(dir) => {
                self.download_dir = dir;
                self.is_dirty = true;
                Task::none()
            }
            SettingsMessage::ReceiveOnlyToggled(val) => {
                self.receive_only = val;
                self.is_dirty = true;
                Task::none()
            }
            SettingsMessage::NotificationsToggled(val) => {
                self.notifications_enabled = val;
                self.is_dirty = true;
                Task::none()
            }
            SettingsMessage::ThemeChanged(theme) => {
                self.theme = theme;
                self.is_dirty = true;
                Task::none()
            }
            SettingsMessage::SaveSettings => {
                config.device_name = self.device_name.clone();
                // Port is fixed at 53317 (not configurable yet)
                config.download_dir = std::path::PathBuf::from(&self.download_dir);
                config.receive_only = self.receive_only;
                config.notifications_enabled = self.notifications_enabled;
                config.theme = self.theme.clone();
                config.trusted_hosts = self.trusted_hosts.clone();

                self.is_dirty = false;
                Task::none()
            }
            SettingsMessage::SettingsSaved => Task::none(),
            // Trusted hosts handlers
            SettingsMessage::TrustedHostInputChanged(input) => {
                self.trusted_host_input = input;
                Task::none()
            }
            SettingsMessage::AddTrustedHost => {
                if !self.trusted_host_input.is_empty() {
                    self.trusted_hosts.push(self.trusted_host_input.clone());
                    self.trusted_host_input.clear();
                    self.is_dirty = true;
                }
                Task::none()
            }
            SettingsMessage::RemoveTrustedHost(idx) => {
                if idx < self.trusted_hosts.len() {
                    self.trusted_hosts.remove(idx);
                    self.is_dirty = true;
                }
                Task::none()
            }
        }
    }
}
