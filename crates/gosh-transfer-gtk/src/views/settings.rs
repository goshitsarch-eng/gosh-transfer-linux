// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Settings View

use crate::application::GoshTransferApplication;
use gosh_transfer_core::AppSettings;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::gio;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SettingsView {
        pub name_row: RefCell<Option<adw::EntryRow>>,
        pub port_row: RefCell<Option<adw::SpinRow>>,
        pub download_row: RefCell<Option<adw::ActionRow>>,
        pub download_path: RefCell<PathBuf>,
        pub receive_only_row: RefCell<Option<adw::SwitchRow>>,
        pub theme_row: RefCell<Option<adw::ComboRow>>,
        pub notifications_row: RefCell<Option<adw::SwitchRow>>,
        pub trusted_hosts_group: RefCell<Option<adw::PreferencesGroup>>,
        pub trusted_host_rows: RefCell<Vec<adw::ActionRow>>,
        pub add_host_row: RefCell<Option<adw::EntryRow>>,
        pub max_retries_row: RefCell<Option<adw::SpinRow>>,
        pub retry_delay_row: RefCell<Option<adw::SpinRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsView {
        const NAME: &'static str = "GoshSettingsView";
        type Type = super::SettingsView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SettingsView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for SettingsView {}
    impl BoxImpl for SettingsView {}

    impl SettingsView {
        fn setup_ui(&self) {
            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(16);
            obj.set_margin_start(24);
            obj.set_margin_end(24);
            obj.set_margin_top(24);
            obj.set_margin_bottom(24);

            // Header
            let header = gtk4::Label::new(Some("Settings"));
            header.add_css_class("title-1");
            header.set_halign(gtk4::Align::Start);
            obj.append(&header);

            // Scrollable content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_vexpand(true);
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);

            // Device settings
            let device_group = adw::PreferencesGroup::new();
            device_group.set_title("Device");

            let name_row = adw::EntryRow::new();
            name_row.set_title("Device Name");
            device_group.add(&name_row);
            *self.name_row.borrow_mut() = Some(name_row);

            let port_row = adw::SpinRow::with_range(1024.0, 65535.0, 1.0);
            port_row.set_title("Server Port");
            port_row.set_subtitle("Port for receiving transfers");
            port_row.set_value(53317.0);
            device_group.add(&port_row);
            *self.port_row.borrow_mut() = Some(port_row);

            content.append(&device_group);

            // Transfer settings
            let transfer_group = adw::PreferencesGroup::new();
            transfer_group.set_title("Transfers");

            let download_row = adw::ActionRow::new();
            download_row.set_title("Download Directory");
            download_row.set_subtitle("~/Downloads");
            let folder_button = gtk4::Button::from_icon_name("folder-open-symbolic");
            folder_button.set_valign(gtk4::Align::Center);
            folder_button.add_css_class("flat");
            download_row.add_suffix(&folder_button);
            download_row.set_activatable_widget(Some(&folder_button));
            transfer_group.add(&download_row);
            *self.download_row.borrow_mut() = Some(download_row.clone());

            // Connect folder button click
            folder_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |button| {
                    this.show_folder_chooser(button);
                }
            ));

            let receive_only_row = adw::SwitchRow::new();
            receive_only_row.set_title("Receive Only Mode");
            receive_only_row.set_subtitle("Disable sending files to others");
            transfer_group.add(&receive_only_row);
            *self.receive_only_row.borrow_mut() = Some(receive_only_row);

            let max_retries_row = adw::SpinRow::with_range(0.0, 10.0, 1.0);
            max_retries_row.set_title("Max Retries");
            max_retries_row.set_subtitle("Retry attempts for failed transfers");
            max_retries_row.set_value(3.0);
            transfer_group.add(&max_retries_row);
            *self.max_retries_row.borrow_mut() = Some(max_retries_row);

            let retry_delay_row = adw::SpinRow::with_range(500.0, 10000.0, 100.0);
            retry_delay_row.set_title("Retry Delay (ms)");
            retry_delay_row.set_subtitle("Delay between retry attempts");
            retry_delay_row.set_value(1000.0);
            transfer_group.add(&retry_delay_row);
            *self.retry_delay_row.borrow_mut() = Some(retry_delay_row);

            content.append(&transfer_group);

            // Appearance settings
            let appearance_group = adw::PreferencesGroup::new();
            appearance_group.set_title("Appearance");

            let theme_row = adw::ComboRow::new();
            theme_row.set_title("Theme");
            let themes = gtk4::StringList::new(&["System", "Light", "Dark"]);
            theme_row.set_model(Some(&themes));
            appearance_group.add(&theme_row);
            *self.theme_row.borrow_mut() = Some(theme_row);

            let notifications_row = adw::SwitchRow::new();
            notifications_row.set_title("Show Notifications");
            notifications_row.set_subtitle("Display system notifications for transfers");
            notifications_row.set_active(true);
            appearance_group.add(&notifications_row);
            *self.notifications_row.borrow_mut() = Some(notifications_row);

            content.append(&appearance_group);

            // Trusted hosts
            let trusted_group = adw::PreferencesGroup::new();
            trusted_group.set_title("Trusted Hosts");
            trusted_group.set_description(Some("Transfers from these hosts are auto-accepted"));
            *self.trusted_hosts_group.borrow_mut() = Some(trusted_group.clone());

            content.append(&trusted_group);

            scrolled.set_child(Some(&content));
            obj.append(&scrolled);

            // Save button
            let save_button = gtk4::Button::with_label("Save Settings");
            save_button.add_css_class("suggested-action");
            save_button.add_css_class("pill");
            save_button.set_halign(gtk4::Align::Center);
            save_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.save_settings();
                }
            ));
            obj.append(&save_button);
        }

        fn show_folder_chooser(&self, button: &gtk4::Button) {
            let window = button
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());

            let dialog = gtk4::FileChooserDialog::new(
                Some("Select Download Directory"),
                window.as_ref(),
                gtk4::FileChooserAction::SelectFolder,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Select", gtk4::ResponseType::Accept),
                ],
            );
            dialog.set_modal(true);

            // Set initial folder to current download path
            let current_path = self.download_path.borrow().clone();
            if current_path.exists() {
                let file = gio::File::for_path(&current_path);
                let _ = dialog.set_current_folder(Some(&file));
            }

            dialog.connect_response(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |dialog, response| {
                    if response == gtk4::ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                *this.download_path.borrow_mut() = path.clone();
                                if let Some(row) = this.download_row.borrow().as_ref() {
                                    let display_path = path.to_string_lossy();
                                    // Abbreviate home directory
                                    let home = std::env::var("HOME").unwrap_or_default();
                                    let subtitle = if display_path.starts_with(&home) {
                                        display_path.replacen(&home, "~", 1)
                                    } else {
                                        display_path.to_string()
                                    };
                                    row.set_subtitle(&subtitle);
                                }
                            }
                        }
                    }
                    dialog.close();
                }
            ));

            dialog.show();
        }

        fn save_settings(&self) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                let name = self
                    .name_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.text().to_string())
                    .unwrap_or_default();

                let port = self
                    .port_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.value() as u16)
                    .unwrap_or(53317);

                let download_dir = self.download_path.borrow().clone();

                let receive_only = self
                    .receive_only_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.is_active())
                    .unwrap_or(false);

                let theme = self
                    .theme_row
                    .borrow()
                    .as_ref()
                    .map(|r| match r.selected() {
                        1 => "light".to_string(),
                        2 => "dark".to_string(),
                        _ => "system".to_string(),
                    })
                    .unwrap_or_else(|| "system".to_string());

                let notifications_enabled = self
                    .notifications_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.is_active())
                    .unwrap_or(true);

                // Get current trusted hosts (preserve existing)
                let trusted_hosts = app.settings().trusted_hosts;

                let max_retries = self
                    .max_retries_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.value() as u32)
                    .unwrap_or(3);

                let retry_delay_ms = self
                    .retry_delay_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.value() as u64)
                    .unwrap_or(1000);

                let new_settings = AppSettings {
                    port,
                    device_name: name,
                    download_dir,
                    trusted_hosts,
                    receive_only,
                    notifications_enabled,
                    theme: theme.clone(),
                    max_retries,
                    retry_delay_ms,
                };

                // Create engine config before moving new_settings
                let engine_config = new_settings.to_engine_config();

                if let Err(e) = app.settings_store().update(new_settings) {
                    tracing::error!("Failed to save settings: {}", e);
                    // Show error toast
                    if let Some(window) = obj.root().and_then(|r| r.downcast::<adw::ApplicationWindow>().ok()) {
                        let toast = adw::Toast::new("Failed to save settings");
                        if let Some(overlay) = window.content().and_then(|c| c.downcast::<adw::ToastOverlay>().ok()) {
                            overlay.add_toast(toast);
                        }
                    }
                } else {
                    // Apply theme immediately
                    app.apply_theme(&theme);

                    // Propagate settings to engine (device_name, download_dir, trusted_hosts, receive_only)
                    app.engine_bridge().update_config(engine_config);

                    // Show success toast
                    if let Some(window) = obj.root().and_then(|r| r.downcast::<adw::ApplicationWindow>().ok()) {
                        let toast = adw::Toast::new("Settings saved");
                        if let Some(overlay) = window.content().and_then(|c| c.downcast::<adw::ToastOverlay>().ok()) {
                            overlay.add_toast(toast);
                        }
                    }
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct SettingsView(ObjectSubclass<imp::SettingsView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SettingsView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Load settings from the application and populate UI
    pub fn load_settings(&self, app: &GoshTransferApplication) {
        let imp = self.imp();
        let settings = app.settings();

        // Device name
        if let Some(row) = imp.name_row.borrow().as_ref() {
            row.set_text(&settings.device_name);
        }

        // Port
        if let Some(row) = imp.port_row.borrow().as_ref() {
            row.set_value(settings.port as f64);
        }

        // Download directory
        *imp.download_path.borrow_mut() = settings.download_dir.clone();
        if let Some(row) = imp.download_row.borrow().as_ref() {
            let display_path = settings.download_dir.to_string_lossy();
            let home = std::env::var("HOME").unwrap_or_default();
            let subtitle = if display_path.starts_with(&home) {
                display_path.replacen(&home, "~", 1)
            } else {
                display_path.to_string()
            };
            row.set_subtitle(&subtitle);
        }

        // Receive only mode
        if let Some(row) = imp.receive_only_row.borrow().as_ref() {
            row.set_active(settings.receive_only);
        }

        // Theme
        if let Some(row) = imp.theme_row.borrow().as_ref() {
            let index = match settings.theme.as_str() {
                "light" => 1,
                "dark" => 2,
                _ => 0, // system
            };
            row.set_selected(index);
        }

        // Notifications
        if let Some(row) = imp.notifications_row.borrow().as_ref() {
            row.set_active(settings.notifications_enabled);
        }

        // Max retries
        if let Some(row) = imp.max_retries_row.borrow().as_ref() {
            row.set_value(settings.max_retries as f64);
        }

        // Retry delay
        if let Some(row) = imp.retry_delay_row.borrow().as_ref() {
            row.set_value(settings.retry_delay_ms as f64);
        }

        // Trusted hosts
        self.load_trusted_hosts(app);
    }

    fn load_trusted_hosts(&self, app: &GoshTransferApplication) {
        let imp = self.imp();
        let settings = app.settings();

        if let Some(group) = imp.trusted_hosts_group.borrow().as_ref() {
            // Remove existing host rows
            for row in imp.trusted_host_rows.borrow().iter() {
                group.remove(row);
            }
            imp.trusted_host_rows.borrow_mut().clear();

            // Remove add row if exists
            if let Some(add_row) = imp.add_host_row.borrow().as_ref() {
                group.remove(add_row);
            }

            // Add existing trusted hosts
            let mut new_rows = Vec::new();
            for host in &settings.trusted_hosts {
                let row = adw::ActionRow::new();
                row.set_title(host);

                let remove_button = gtk4::Button::from_icon_name("user-trash-symbolic");
                remove_button.set_valign(gtk4::Align::Center);
                remove_button.add_css_class("flat");
                remove_button.add_css_class("error");

                let host_clone = host.clone();
                remove_button.connect_clicked(glib::clone!(
                    #[weak(rename_to = view)]
                    self,
                    move |_| {
                        if let Some(app) = view.get_app() {
                            if let Err(e) = app.settings_store().remove_trusted_host(&host_clone) {
                                tracing::error!("Failed to remove trusted host: {}", e);
                            } else {
                                view.load_trusted_hosts(&app);
                            }
                        }
                    }
                ));

                row.add_suffix(&remove_button);
                group.add(&row);
                new_rows.push(row);
            }
            *imp.trusted_host_rows.borrow_mut() = new_rows;

            // Add "add new host" row
            let add_row = adw::EntryRow::new();
            add_row.set_title("Add trusted host");
            add_row.connect_apply(glib::clone!(
                #[weak(rename_to = view)]
                self,
                move |entry| {
                    let host = entry.text().to_string();
                    if !host.is_empty() {
                        if let Some(app) = view.get_app() {
                            if let Err(e) = app.settings_store().add_trusted_host(host) {
                                tracing::error!("Failed to add trusted host: {}", e);
                            } else {
                                entry.set_text("");
                                view.load_trusted_hosts(&app);
                            }
                        }
                    }
                }
            ));
            group.add(&add_row);
            *imp.add_host_row.borrow_mut() = Some(add_row);
        }
    }

    fn get_app(&self) -> Option<GoshTransferApplication> {
        self.root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok())
            .and_then(|w| w.application())
            .and_then(|a| a.downcast::<GoshTransferApplication>().ok())
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}
