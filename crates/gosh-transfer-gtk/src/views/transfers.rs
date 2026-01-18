// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Transfers View (History)

use crate::application::GoshTransferApplication;
use chrono::{DateTime, Utc};
use gosh_transfer_core::{TransferDirection, TransferRecord, TransferStatus};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct TransfersView {
        pub history_group: RefCell<Option<adw::PreferencesGroup>>,
        pub empty_row: RefCell<Option<adw::ActionRow>>,
        pub history_rows: RefCell<Vec<adw::ActionRow>>,
        pub clear_button: RefCell<Option<gtk4::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TransfersView {
        const NAME: &'static str = "GoshTransfersView";
        type Type = super::TransfersView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for TransfersView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for TransfersView {}
    impl BoxImpl for TransfersView {}

    impl TransfersView {
        fn setup_ui(&self) {
            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(16);
            obj.set_margin_start(24);
            obj.set_margin_end(24);
            obj.set_margin_top(24);
            obj.set_margin_bottom(24);

            // Header with clear button
            let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

            let header = gtk4::Label::new(Some("Transfer History"));
            header.add_css_class("title-1");
            header.set_halign(gtk4::Align::Start);
            header.set_hexpand(true);
            header_box.append(&header);

            let clear_button = gtk4::Button::with_label("Clear History");
            clear_button.add_css_class("destructive-action");
            clear_button.set_sensitive(false); // Disabled when empty
            header_box.append(&clear_button);

            // Connect clear button
            clear_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.clear_history();
                }
            ));

            *self.clear_button.borrow_mut() = Some(clear_button);

            obj.append(&header_box);

            // History list
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_vexpand(true);
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let history_group = adw::PreferencesGroup::new();
            history_group.set_title("Recent Transfers");

            let empty_row = adw::ActionRow::new();
            empty_row.set_title("No transfer history");
            empty_row.set_subtitle("Completed transfers will appear here");
            history_group.add(&empty_row);

            *self.history_group.borrow_mut() = Some(history_group.clone());
            *self.empty_row.borrow_mut() = Some(empty_row);

            scrolled.set_child(Some(&history_group));
            obj.append(&scrolled);
        }

        fn clear_history(&self) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                if let Err(e) = app.history().clear() {
                    tracing::error!("Failed to clear history: {}", e);
                }
                obj.load_history(&app);
            }
        }
    }
}

glib::wrapper! {
    pub struct TransfersView(ObjectSubclass<imp::TransfersView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl TransfersView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Load and display transfer history
    pub fn load_history(&self, app: &GoshTransferApplication) {
        let imp = self.imp();
        let records = app.history().list();

        // Clear existing dynamic rows
        if let Some(group) = imp.history_group.borrow().as_ref() {
            for row in imp.history_rows.borrow_mut().drain(..) {
                group.remove(&row);
            }
        }

        if records.is_empty() {
            // Show empty state
            if let Some(empty_row) = imp.empty_row.borrow().as_ref() {
                empty_row.set_visible(true);
            }
            if let Some(btn) = imp.clear_button.borrow().as_ref() {
                btn.set_sensitive(false);
            }
            return;
        }

        // Hide empty row and enable clear button
        if let Some(empty_row) = imp.empty_row.borrow().as_ref() {
            empty_row.set_visible(false);
        }
        if let Some(btn) = imp.clear_button.borrow().as_ref() {
            btn.set_sensitive(true);
        }

        // Add rows for each record
        let mut new_rows = Vec::new();
        for record in records {
            let row = create_history_row(&record);
            if let Some(group) = imp.history_group.borrow().as_ref() {
                group.add(&row);
            }
            new_rows.push(row);
        }
        *imp.history_rows.borrow_mut() = new_rows;
    }

    fn get_app(&self) -> Option<GoshTransferApplication> {
        self.root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok())
            .and_then(|w| w.application())
            .and_then(|a| a.downcast::<GoshTransferApplication>().ok())
    }
}

impl Default for TransfersView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a row for a transfer record
fn create_history_row(record: &TransferRecord) -> adw::ActionRow {
    let row = adw::ActionRow::new();

    // Direction icon
    let icon_name = match record.direction {
        TransferDirection::Sent => "go-up-symbolic",
        TransferDirection::Received => "go-down-symbolic",
    };
    let icon = gtk4::Image::from_icon_name(icon_name);
    row.add_prefix(&icon);

    // Title: peer address
    row.set_title(&record.peer_address);

    // Subtitle: file info, size, status, time
    let file_text = if record.files.len() == 1 {
        record.files[0].name.clone()
    } else {
        format!("{} files", record.files.len())
    };
    let size_text = format_size(record.total_size);
    let status_text = format_status(&record.status);
    let time_text = format_relative_time(record.started_at);

    row.set_subtitle(&format!(
        "{} \u{2022} {} \u{2022} {} \u{2022} {}",
        file_text, size_text, status_text, time_text
    ));

    // Status indicator suffix
    let status_icon = match &record.status {
        TransferStatus::Completed => {
            let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            icon.add_css_class("success");
            icon
        }
        TransferStatus::Failed | TransferStatus::Rejected => {
            let icon = gtk4::Image::from_icon_name("dialog-error-symbolic");
            icon.add_css_class("error");
            icon
        }
        _ => gtk4::Image::from_icon_name("content-loading-symbolic"),
    };
    row.add_suffix(&status_icon);

    row
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format transfer status as display text
fn format_status(status: &TransferStatus) -> &'static str {
    match status {
        TransferStatus::Pending => "Pending",
        TransferStatus::InProgress => "In Progress",
        TransferStatus::Completed => "Completed",
        TransferStatus::Failed => "Failed",
        TransferStatus::Rejected => "Rejected",
    }
}

/// Format timestamp as relative time
fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        "Just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        if mins == 1 {
            "1 min ago".to_string()
        } else {
            format!("{} mins ago", mins)
        }
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        if days == 1 {
            "Yesterday".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else {
        time.format("%b %d").to_string()
    }
}
