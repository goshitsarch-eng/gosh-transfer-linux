// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Receive View

use crate::application::GoshTransferApplication;
use gosh_lan_transfer::PendingTransfer;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::collections::HashMap;

mod imp {
    use super::*;
    use std::cell::RefCell;

    pub struct ActiveTransferRow {
        pub row: adw::ActionRow,
        pub progress_bar: gtk4::ProgressBar,
        pub status_label: gtk4::Label,
    }

    #[derive(Default)]
    pub struct ReceiveView {
        pub device_row: RefCell<Option<adw::ActionRow>>,
        pub port_row: RefCell<Option<adw::ActionRow>>,
        pub addresses_card: RefCell<Option<adw::PreferencesGroup>>,
        pub address_rows: RefCell<Vec<adw::ActionRow>>,
        pub pending_card: RefCell<Option<adw::PreferencesGroup>>,
        pub empty_row: RefCell<Option<adw::ActionRow>>,
        pub pending_rows: RefCell<HashMap<String, adw::ActionRow>>,
        pub active_card: RefCell<Option<adw::PreferencesGroup>>,
        pub empty_active_row: RefCell<Option<adw::ActionRow>>,
        pub active_rows: RefCell<HashMap<String, ActiveTransferRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReceiveView {
        const NAME: &'static str = "GoshReceiveView";
        type Type = super::ReceiveView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ReceiveView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for ReceiveView {}
    impl BoxImpl for ReceiveView {}

    impl ReceiveView {
        fn setup_ui(&self) {
            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(16);
            obj.set_margin_start(24);
            obj.set_margin_end(24);
            obj.set_margin_top(24);
            obj.set_margin_bottom(24);

            // Header
            let header = gtk4::Label::new(Some("Receive Files"));
            header.add_css_class("title-1");
            header.set_halign(gtk4::Align::Start);
            obj.append(&header);

            // Scrollable content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_vexpand(true);
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);

            // Server status card
            let status_card = adw::PreferencesGroup::new();
            status_card.set_title("Server Status");

            let device_row = adw::ActionRow::new();
            device_row.set_title("Device Name");
            device_row.set_subtitle("Loading...");
            status_card.add(&device_row);
            *self.device_row.borrow_mut() = Some(device_row);

            let port_row = adw::ActionRow::new();
            port_row.set_title("Port");
            port_row.set_subtitle("53317");
            status_card.add(&port_row);
            *self.port_row.borrow_mut() = Some(port_row);

            content.append(&status_card);

            // Your Addresses card
            let addresses_card = adw::PreferencesGroup::new();
            addresses_card.set_title("Your Addresses");
            addresses_card.set_description(Some("Share one of these with the sender"));
            *self.addresses_card.borrow_mut() = Some(addresses_card.clone());

            content.append(&addresses_card);

            // Pending transfers card
            let pending_card = adw::PreferencesGroup::new();
            pending_card.set_title("Pending Transfers");
            pending_card.set_description(Some("Incoming transfer requests will appear here"));

            let empty_row = adw::ActionRow::new();
            empty_row.set_title("No pending transfers");
            empty_row.set_subtitle("Waiting for incoming connections...");
            pending_card.add(&empty_row);
            *self.empty_row.borrow_mut() = Some(empty_row);
            *self.pending_card.borrow_mut() = Some(pending_card.clone());

            content.append(&pending_card);

            // Active transfers card
            let active_card = adw::PreferencesGroup::new();
            active_card.set_title("Active Transfers");

            let empty_active = adw::ActionRow::new();
            empty_active.set_title("No active transfers");
            active_card.add(&empty_active);
            *self.empty_active_row.borrow_mut() = Some(empty_active);
            *self.active_card.borrow_mut() = Some(active_card.clone());

            content.append(&active_card);

            scrolled.set_child(Some(&content));
            obj.append(&scrolled);
        }
    }
}

glib::wrapper! {
    pub struct ReceiveView(ObjectSubclass<imp::ReceiveView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl ReceiveView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Load initial data from the application
    pub fn load_data(&self, app: &GoshTransferApplication) {
        let imp = self.imp();
        let settings = app.settings();

        // Update device name
        if let Some(row) = imp.device_row.borrow().as_ref() {
            row.set_subtitle(&settings.device_name);
        }

        // Update port
        let port = settings.port;
        if let Some(row) = imp.port_row.borrow().as_ref() {
            row.set_subtitle(&port.to_string());
        }

        // Load network addresses
        let addresses_card = imp.addresses_card.borrow().clone();
        app.engine_bridge().get_interfaces(move |interfaces| {
            if let Some(card) = addresses_card.as_ref() {
                if interfaces.is_empty() {
                    let row = adw::ActionRow::new();
                    row.set_title("No network interfaces found");
                    card.add(&row);
                } else {
                    for iface in interfaces {
                        // Skip loopback
                        if iface.name == "lo" {
                            continue;
                        }

                        // Determine interface type for icon/description
                        let (icon, description) = if iface.name.starts_with("tailscale") || iface.name.starts_with("tun") {
                            ("network-vpn-symbolic", "Tailscale VPN")
                        } else if iface.name.starts_with("wl") {
                            ("network-wireless-symbolic", "WiFi")
                        } else if iface.name.starts_with("en") || iface.name.starts_with("eth") {
                            ("network-wired-symbolic", "Ethernet")
                        } else if iface.name.starts_with("docker") || iface.name.starts_with("br-") {
                            ("network-server-symbolic", "Docker")
                        } else {
                            ("network-workgroup-symbolic", &iface.name as &str)
                        };

                        let row = adw::ActionRow::new();

                        // Format address with port
                        let addr_with_port = format!("{}:{}", iface.ip, port);
                        row.set_title(&addr_with_port);
                        row.set_subtitle(description);

                        // Add icon
                        let icon_widget = gtk4::Image::from_icon_name(icon);
                        row.add_prefix(&icon_widget);

                        // Add copy button
                        let copy_btn = gtk4::Button::from_icon_name("edit-copy-symbolic");
                        copy_btn.set_valign(gtk4::Align::Center);
                        copy_btn.add_css_class("flat");
                        copy_btn.set_tooltip_text(Some("Copy to clipboard"));

                        let addr_clone = addr_with_port.clone();
                        copy_btn.connect_clicked(move |btn| {
                            let clipboard = btn.clipboard();
                            clipboard.set_text(&addr_clone);
                        });
                        row.add_suffix(&copy_btn);

                        card.add(&row);
                    }
                }
            }
        });
    }

    /// Add a pending transfer to the UI
    pub fn add_pending_transfer(&self, transfer: &PendingTransfer, app: &GoshTransferApplication) {
        let imp = self.imp();

        // Hide empty row
        if let Some(empty_row) = imp.empty_row.borrow().as_ref() {
            empty_row.set_visible(false);
        }

        // Create transfer row
        let row = adw::ActionRow::new();

        let file_count = transfer.files.len();
        let total_size = transfer.files.iter().map(|f| f.size).sum::<u64>();
        let size_str = format_size(total_size);

        let title = if file_count == 1 {
            transfer.files[0].name.clone()
        } else {
            format!("{} files", file_count)
        };

        row.set_title(&title);
        let sender = transfer.sender_name.as_deref().unwrap_or("Unknown");
        row.set_subtitle(&format!("From {} - {}", sender, size_str));

        // Accept button
        let accept_btn = gtk4::Button::with_label("Accept");
        accept_btn.add_css_class("suggested-action");
        accept_btn.set_valign(gtk4::Align::Center);

        let transfer_id = transfer.id.clone();
        let app_weak = app.downgrade();
        accept_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                if let Some(app) = app_weak.upgrade() {
                    app.engine_bridge().accept_transfer(transfer_id.clone());
                    view.remove_pending_transfer(&transfer_id);
                }
            }
        ));
        row.add_suffix(&accept_btn);

        // Reject button
        let reject_btn = gtk4::Button::with_label("Reject");
        reject_btn.add_css_class("destructive-action");
        reject_btn.set_valign(gtk4::Align::Center);

        let transfer_id = transfer.id.clone();
        let app_weak = app.downgrade();
        reject_btn.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                if let Some(app) = app_weak.upgrade() {
                    app.engine_bridge().reject_transfer(transfer_id.clone());
                    view.remove_pending_transfer(&transfer_id);
                }
            }
        ));
        row.add_suffix(&reject_btn);

        // Add to card
        if let Some(card) = imp.pending_card.borrow().as_ref() {
            card.add(&row);
        }

        // Store reference
        imp.pending_rows.borrow_mut().insert(transfer.id.clone(), row);
    }

    /// Remove a pending transfer from the UI
    pub fn remove_pending_transfer(&self, transfer_id: &str) {
        let imp = self.imp();

        if let Some(row) = imp.pending_rows.borrow_mut().remove(transfer_id) {
            if let Some(card) = imp.pending_card.borrow().as_ref() {
                card.remove(&row);
            }
        }

        // Show empty row if no pending transfers
        if imp.pending_rows.borrow().is_empty() {
            if let Some(empty_row) = imp.empty_row.borrow().as_ref() {
                empty_row.set_visible(true);
            }
        }
    }

    /// Clear all pending transfers
    pub fn clear_pending_transfers(&self) {
        let imp = self.imp();

        for (_, row) in imp.pending_rows.borrow().iter() {
            if let Some(card) = imp.pending_card.borrow().as_ref() {
                card.remove(row);
            }
        }
        imp.pending_rows.borrow_mut().clear();

        // Show empty row
        if let Some(empty_row) = imp.empty_row.borrow().as_ref() {
            empty_row.set_visible(true);
        }
    }

    /// Add an active transfer (when accepted)
    pub fn add_active_transfer(&self, transfer_id: &str, title: &str) {
        let imp = self.imp();

        // Skip if already exists
        if imp.active_rows.borrow().contains_key(transfer_id) {
            return;
        }

        // Hide empty active row
        if let Some(empty_row) = imp.empty_active_row.borrow().as_ref() {
            empty_row.set_visible(false);
        }

        // Create transfer row with progress
        let row = adw::ActionRow::new();
        row.set_title(title);
        row.set_subtitle("Starting transfer...");

        // Progress bar
        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.set_valign(gtk4::Align::Center);
        progress_bar.set_hexpand(true);
        progress_bar.set_width_request(150);
        progress_bar.set_fraction(0.0);
        row.add_suffix(&progress_bar);

        // Status label
        let status_label = gtk4::Label::new(Some("0%"));
        status_label.set_valign(gtk4::Align::Center);
        status_label.add_css_class("dim-label");
        row.add_suffix(&status_label);

        // Add to card
        if let Some(card) = imp.active_card.borrow().as_ref() {
            card.add(&row);
        }

        // Store reference
        imp.active_rows.borrow_mut().insert(
            transfer_id.to_string(),
            imp::ActiveTransferRow {
                row,
                progress_bar,
                status_label,
            },
        );
    }

    /// Update transfer progress
    pub fn update_transfer_progress(&self, transfer_id: &str, bytes_transferred: u64, total_bytes: u64, speed_bps: u64) {
        let imp = self.imp();

        if let Some(active) = imp.active_rows.borrow().get(transfer_id) {
            let fraction = if total_bytes > 0 {
                bytes_transferred as f64 / total_bytes as f64
            } else {
                0.0
            };

            active.progress_bar.set_fraction(fraction);

            let percent = (fraction * 100.0) as u32;
            let speed_str = format_speed(speed_bps);
            let transferred_str = format_size(bytes_transferred);
            let total_str = format_size(total_bytes);

            active.status_label.set_text(&format!("{}%", percent));
            active.row.set_subtitle(&format!("{} / {} - {}", transferred_str, total_str, speed_str));
        }
    }

    /// Remove active transfer (completed or failed)
    pub fn remove_active_transfer(&self, transfer_id: &str) {
        let imp = self.imp();

        if let Some(active) = imp.active_rows.borrow_mut().remove(transfer_id) {
            if let Some(card) = imp.active_card.borrow().as_ref() {
                card.remove(&active.row);
            }
        }

        // Show empty row if no active transfers
        if imp.active_rows.borrow().is_empty() {
            if let Some(empty_row) = imp.empty_active_row.borrow().as_ref() {
                empty_row.set_visible(true);
            }
        }
    }

    /// Mark transfer as completed
    pub fn mark_transfer_complete(&self, transfer_id: &str) {
        let imp = self.imp();

        if let Some(active) = imp.active_rows.borrow().get(transfer_id) {
            active.progress_bar.set_fraction(1.0);
            active.status_label.set_text("Done");
            active.row.set_subtitle("Transfer completed");
            active.row.add_css_class("success");
        }

        // Remove after a delay
        let transfer_id = transfer_id.to_string();
        glib::timeout_add_seconds_local_once(
            3,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move || {
                    view.remove_active_transfer(&transfer_id);
                }
            ),
        );
    }

    /// Mark transfer as failed
    pub fn mark_transfer_failed(&self, transfer_id: &str, error: &str) {
        let imp = self.imp();

        if let Some(active) = imp.active_rows.borrow().get(transfer_id) {
            active.status_label.set_text("Failed");
            active.row.set_subtitle(error);
            active.row.add_css_class("error");
        }

        // Remove after a delay
        let transfer_id = transfer_id.to_string();
        glib::timeout_add_seconds_local_once(
            5,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move || {
                    view.remove_active_transfer(&transfer_id);
                }
            ),
        );
    }
}

impl Default for ReceiveView {
    fn default() -> Self {
        Self::new()
    }
}

fn format_size(bytes: u64) -> String {
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
        format!("{} bytes", bytes)
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}
