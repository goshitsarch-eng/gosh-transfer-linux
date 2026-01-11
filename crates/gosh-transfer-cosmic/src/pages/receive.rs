// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Receive Page

use crate::engine::EngineBridge;
use cosmic::iced::Length;
use cosmic::widget::{self, button, container, text};
use cosmic::{theme, Element, Task};
use gosh_lan_transfer::PendingTransfer;
use local_ip_address::list_afinet_netifas;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    AcceptTransfer(String),
    RejectTransfer(String),
    TransferAccepted(String),
    TransferRejected(String),
    RefreshPending,
    PendingLoaded(Vec<PendingTransfer>),
    CopyAddress(String),
}

/// Represents an active transfer in progress
#[derive(Debug, Clone)]
pub struct ActiveTransfer {
    pub id: String,
    pub title: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub is_complete: bool,
    pub error: Option<String>,
}

pub struct ReceivePage {
    pending_transfers: Vec<PendingTransfer>,
    active_transfers: Vec<ActiveTransfer>,
    network_addresses: Vec<(String, String)>, // (interface name, ip address)
    is_loading: bool,
    device_name: String,
}

impl ReceivePage {
    pub fn new() -> Self {
        // Load network addresses
        let network_addresses = Self::load_network_addresses();
        let device_name = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "My Computer".to_string());

        Self {
            pending_transfers: Vec::new(),
            active_transfers: Vec::new(),
            network_addresses,
            is_loading: false,
            device_name,
        }
    }

    fn load_network_addresses() -> Vec<(String, String)> {
        let mut addresses = Vec::new();

        if let Ok(netifas) = list_afinet_netifas() {
            for (name, ip) in netifas {
                // Skip loopback
                if ip.is_loopback() {
                    continue;
                }
                addresses.push((name, ip.to_string()));
            }
        }

        // Sort by interface name
        addresses.sort_by(|a, b| a.0.cmp(&b.0));
        addresses
    }

    /// Get interface icon based on interface name
    fn get_interface_icon(if_name: &str) -> &'static str {
        if if_name.starts_with("wl") {
            "network-wireless-symbolic"
        } else if if_name.starts_with("en") || if_name.starts_with("eth") {
            "network-wired-symbolic"
        } else if if_name.starts_with("tailscale") || if_name.starts_with("tun") {
            "network-vpn-symbolic"
        } else if if_name.starts_with("docker") || if_name.starts_with("br-") {
            "container-symbolic"
        } else {
            "network-workgroup-symbolic"
        }
    }

    /// Get interface description based on interface name
    fn get_interface_description(if_name: &str) -> &'static str {
        if if_name.starts_with("wl") {
            "WiFi"
        } else if if_name.starts_with("en") || if_name.starts_with("eth") {
            "Ethernet"
        } else if if_name.starts_with("tailscale") {
            "Tailscale VPN"
        } else if if_name.starts_with("tun") {
            "VPN"
        } else if if_name.starts_with("docker") || if_name.starts_with("br-") {
            "Docker"
        } else {
            "Network"
        }
    }

    /// Add a new active transfer
    pub fn add_active_transfer(&mut self, id: String, title: String, total_bytes: u64) {
        self.active_transfers.push(ActiveTransfer {
            id,
            title,
            bytes_transferred: 0,
            total_bytes,
            speed_bps: 0,
            is_complete: false,
            error: None,
        });
    }

    /// Update transfer progress
    pub fn update_transfer_progress(&mut self, id: &str, bytes_transferred: u64, speed_bps: u64) {
        if let Some(transfer) = self.active_transfers.iter_mut().find(|t| t.id == id) {
            transfer.bytes_transferred = bytes_transferred;
            transfer.speed_bps = speed_bps;
        }
    }

    /// Mark transfer as complete
    pub fn mark_transfer_complete(&mut self, id: &str) {
        if let Some(transfer) = self.active_transfers.iter_mut().find(|t| t.id == id) {
            transfer.is_complete = true;
            transfer.bytes_transferred = transfer.total_bytes;
        }
    }

    /// Mark transfer as failed
    pub fn mark_transfer_failed(&mut self, id: &str, error: String) {
        if let Some(transfer) = self.active_transfers.iter_mut().find(|t| t.id == id) {
            transfer.error = Some(error);
        }
    }

    /// Get pending transfer count
    pub fn pending_count(&self) -> usize {
        self.pending_transfers.len()
    }

    pub fn view(&self) -> Element<'_, ReceiveMessage> {
        let spacing = theme::active().cosmic().spacing;

        // Header
        let header = widget::row()
            .push(text::title3("Receive Files"))
            .push(widget::horizontal_space())
            .push(
                button::icon(widget::icon::from_name("view-refresh-symbolic"))
                    .on_press(ReceiveMessage::RefreshPending),
            );

        // Your Addresses card
        let mut addresses_column = widget::column().spacing(spacing.space_xs);
        for (name, ip) in &self.network_addresses {
            let icon_name = Self::get_interface_icon(name);
            let description = Self::get_interface_description(name);
            let address_with_port = format!("{}:53317", ip);

            let address_row = widget::row()
                .push(widget::icon::from_name(icon_name).size(20))
                .push(
                    widget::column()
                        .push(text::body(address_with_port.clone()))
                        .push(text::caption(description))
                        .spacing(2),
                )
                .push(widget::horizontal_space())
                .push(
                    button::icon(widget::icon::from_name("edit-copy-symbolic"))
                        .on_press(ReceiveMessage::CopyAddress(address_with_port)),
                )
                .align_y(cosmic::iced::Alignment::Center)
                .spacing(spacing.space_s);
            addresses_column = addresses_column.push(address_row);
        }

        let addresses_card = container(
            widget::column()
                .push(text::title4("Your Addresses"))
                .push(text::caption("Share one of these with the sender"))
                .push(if self.network_addresses.is_empty() {
                    widget::column().push(text::body("No network interfaces found"))
                } else {
                    addresses_column
                })
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Server status with device name
        let status_card = container(
            widget::column()
                .push(text::title4("Server Status"))
                .push(
                    widget::row()
                        .push(text::body("Device Name:"))
                        .push(text::body(&self.device_name))
                        .spacing(spacing.space_s),
                )
                .push(
                    widget::row()
                        .push(text::body("Port:"))
                        .push(text::body("53317"))
                        .spacing(spacing.space_s),
                )
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        // Pending transfers
        let pending_content: Element<ReceiveMessage> = if self.pending_transfers.is_empty() {
            container(
                widget::column()
                    .push(widget::icon::from_name("folder-download-symbolic").size(64))
                    .push(text::title4("No pending transfers"))
                    .push(text::body("Incoming transfer requests will appear here"))
                    .spacing(spacing.space_m)
                    .align_x(cosmic::iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding(spacing.space_l)
            .into()
        } else {
            let mut transfer_column = widget::column().spacing(spacing.space_s);

            for transfer in &self.pending_transfers {
                let sender = transfer.sender_name.as_deref().unwrap_or("Unknown");
                let file_count = transfer.files.len();
                let total_size: u64 = transfer.files.iter().map(|f| f.size).sum();
                let size_str = format_bytes(total_size);

                let card = container(
                    widget::column()
                        .push(
                            widget::row()
                                .push(widget::icon::from_name("computer-symbolic"))
                                .push(text::title4(sender))
                                .push(widget::horizontal_space())
                                .push(text::caption(format!("{} files ({})", file_count, size_str)))
                                .spacing(spacing.space_s)
                                .align_y(cosmic::iced::Alignment::Center),
                        )
                        .push(
                            widget::row()
                                .push(
                                    button::destructive("Reject")
                                        .on_press(ReceiveMessage::RejectTransfer(transfer.id.clone())),
                                )
                                .push(
                                    button::suggested("Accept")
                                        .on_press(ReceiveMessage::AcceptTransfer(transfer.id.clone())),
                                )
                                .spacing(spacing.space_s),
                        )
                        .spacing(spacing.space_s),
                )
                .padding(spacing.space_m)
                .class(theme::Container::Card);

                transfer_column = transfer_column.push(card);
            }

            transfer_column.into()
        };

        // Active transfers with progress
        let active_content: Element<ReceiveMessage> = if self.active_transfers.is_empty() {
            container(
                text::body("No active transfers"),
            )
            .padding(spacing.space_m)
            .into()
        } else {
            let mut transfer_column = widget::column().spacing(spacing.space_s);

            for transfer in &self.active_transfers {
                let progress = if transfer.total_bytes > 0 {
                    (transfer.bytes_transferred as f32) / (transfer.total_bytes as f32)
                } else {
                    0.0
                };

                let status_text = if transfer.is_complete {
                    "Completed".to_string()
                } else if let Some(ref error) = transfer.error {
                    format!("Failed: {}", error)
                } else {
                    let transferred = format_bytes(transfer.bytes_transferred);
                    let total = format_bytes(transfer.total_bytes);
                    let speed = format_bytes(transfer.speed_bps);
                    format!("{} / {} ({}/s)", transferred, total, speed)
                };

                let card = container(
                    widget::column()
                        .push(text::body(&transfer.title))
                        .push(widget::progress_bar(0.0..=1.0, progress))
                        .push(text::caption(status_text))
                        .spacing(spacing.space_xs),
                )
                .padding(spacing.space_m)
                .class(theme::Container::Card);

                transfer_column = transfer_column.push(card);
            }

            transfer_column.into()
        };

        // Active transfers card
        let active_card = container(
            widget::column()
                .push(text::title4("Active Transfers"))
                .push(active_content)
                .spacing(spacing.space_s),
        )
        .padding(spacing.space_m)
        .class(theme::Container::Card);

        widget::column()
            .push(header)
            .push(addresses_card)
            .push(status_card)
            .push(pending_content)
            .push(active_card)
            .spacing(spacing.space_m)
            .padding(spacing.space_m)
            .into()
    }

    pub fn update(
        &mut self,
        message: ReceiveMessage,
        engine: &Arc<EngineBridge>,
    ) -> Task<ReceiveMessage> {
        match message {
            ReceiveMessage::AcceptTransfer(id) => {
                let engine = engine.clone();
                let transfer_id = id.clone();

                cosmic::task::future(async move {
                    match engine.accept_transfer(&transfer_id).await {
                        Ok(_) => ReceiveMessage::TransferAccepted(transfer_id),
                        Err(_) => ReceiveMessage::TransferRejected(transfer_id),
                    }
                })
            }
            ReceiveMessage::RejectTransfer(id) => {
                let engine = engine.clone();
                let transfer_id = id.clone();

                cosmic::task::future(async move {
                    let _ = engine.reject_transfer(&transfer_id).await;
                    ReceiveMessage::TransferRejected(transfer_id)
                })
            }
            ReceiveMessage::TransferAccepted(id) | ReceiveMessage::TransferRejected(id) => {
                self.pending_transfers.retain(|t| t.id != id);
                Task::none()
            }
            ReceiveMessage::RefreshPending => {
                self.is_loading = true;
                let engine = engine.clone();

                cosmic::task::future(async move {
                    let pending = engine.get_pending_transfers().await;
                    ReceiveMessage::PendingLoaded(pending)
                })
            }
            ReceiveMessage::PendingLoaded(pending) => {
                self.is_loading = false;
                self.pending_transfers = pending;
                Task::none()
            }
            ReceiveMessage::CopyAddress(address) => {
                cosmic::task::future(async move {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&address);
                    }
                    // No follow-up message needed
                    ReceiveMessage::RefreshPending
                })
            }
        }
    }
}

/// Format bytes into human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
