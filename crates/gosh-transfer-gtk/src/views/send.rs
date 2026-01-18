// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Send View

use crate::application::GoshTransferApplication;
use gosh_lan_transfer::FavoritesPersistence;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    pub struct SendView {
        pub dest_row: RefCell<Option<adw::EntryRow>>,
        pub resolution_label: RefCell<Option<gtk4::Label>>,
        pub resolve_timeout: RefCell<Option<glib::SourceId>>,
        pub resolve_generation: Cell<u64>,
        pub test_button: RefCell<Option<gtk4::Button>>,
        pub test_spinner: RefCell<Option<gtk4::Spinner>>,
        pub favorites_dropdown: RefCell<Option<adw::ComboRow>>,
        #[allow(clippy::type_complexity)]
        pub favorites_list: RefCell<Vec<(String, String, String, Option<String>)>>, // (id, name, address, last_resolved_ip)
        pub favorite_ip_label: RefCell<Option<gtk4::Label>>,
        pub files_list: RefCell<Option<gtk4::ListBox>>,
        pub files_row: RefCell<Option<adw::ActionRow>>,
        pub selected_files: RefCell<Vec<std::path::PathBuf>>,
        pub selected_directory: RefCell<Option<std::path::PathBuf>>,
        pub send_button: RefCell<Option<gtk4::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SendView {
        const NAME: &'static str = "GoshSendView";
        type Type = super::SendView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SendView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for SendView {}
    impl BoxImpl for SendView {}

    impl SendView {
        fn setup_ui(&self) {
            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(16);
            obj.set_margin_start(24);
            obj.set_margin_end(24);
            obj.set_margin_top(24);
            obj.set_margin_bottom(24);

            // Header
            let header = gtk4::Label::new(Some("Send Files"));
            header.add_css_class("title-1");
            header.set_halign(gtk4::Align::Start);
            obj.append(&header);

            // Scrollable content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_vexpand(true);
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 16);

            // Favorites card
            let favorites_card = adw::PreferencesGroup::new();
            favorites_card.set_title("Favorites");
            favorites_card.set_description(Some("Quick access to saved destinations"));

            let favorites_dropdown = adw::ComboRow::new();
            favorites_dropdown.set_title("Select favorite");
            let empty_model = gtk4::StringList::new(&["No favorites saved"]);
            favorites_dropdown.set_model(Some(&empty_model));
            favorites_dropdown.set_sensitive(false);

            // Add manage button to favorites row
            let manage_button = gtk4::Button::from_icon_name("document-edit-symbolic");
            manage_button.set_tooltip_text(Some("Manage favorites"));
            manage_button.set_valign(gtk4::Align::Center);
            manage_button.add_css_class("flat");
            favorites_dropdown.add_suffix(&manage_button);

            manage_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |button| {
                    this.show_manage_favorites_dialog(button);
                }
            ));

            favorites_card.add(&favorites_dropdown);
            *self.favorites_dropdown.borrow_mut() = Some(favorites_dropdown.clone());

            // Label to show last resolved IP when favorite is selected
            let favorite_ip_label = gtk4::Label::new(None);
            favorite_ip_label.set_halign(gtk4::Align::Start);
            favorite_ip_label.set_margin_start(12);
            favorite_ip_label.set_margin_top(4);
            favorite_ip_label.add_css_class("dim-label");
            favorite_ip_label.add_css_class("caption");
            favorite_ip_label.set_visible(false);
            favorites_card.add(&favorite_ip_label);
            *self.favorite_ip_label.borrow_mut() = Some(favorite_ip_label);

            // Connect favorite selection
            favorites_dropdown.connect_selected_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |dropdown| {
                    let selected = dropdown.selected() as usize;
                    let favorites = this.favorites_list.borrow();
                    if selected < favorites.len() {
                        let (_, _, address, last_ip) = &favorites[selected];
                        if let Some(dest_row) = this.dest_row.borrow().as_ref() {
                            dest_row.set_text(address);
                        }

                        // Show last resolved IP if available and different from address
                        if let Some(label) = this.favorite_ip_label.borrow().as_ref() {
                            if let Some(ip) = last_ip {
                                if ip != address {
                                    label.set_text(&format!("Last resolved: {}", ip));
                                    label.set_visible(true);
                                } else {
                                    label.set_visible(false);
                                }
                            } else {
                                label.set_visible(false);
                            }
                        }
                    }
                }
            ));

            content.append(&favorites_card);

            // Destination card
            let dest_card = adw::PreferencesGroup::new();
            dest_card.set_title("Destination");
            dest_card.set_description(Some("Enter the hostname or IP address of the recipient"));

            let dest_row = adw::EntryRow::new();
            dest_row.set_title("Address");

            // Test connection button and spinner
            let test_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
            let test_spinner = gtk4::Spinner::new();
            test_spinner.set_visible(false);
            let test_button = gtk4::Button::from_icon_name("network-transmit-symbolic");
            test_button.set_tooltip_text(Some("Test connection"));
            test_button.set_valign(gtk4::Align::Center);
            test_button.add_css_class("flat");
            test_box.append(&test_spinner);
            test_box.append(&test_button);
            dest_row.add_suffix(&test_box);

            *self.test_button.borrow_mut() = Some(test_button.clone());
            *self.test_spinner.borrow_mut() = Some(test_spinner.clone());

            test_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.test_connection();
                }
            ));

            dest_card.add(&dest_row);
            *self.dest_row.borrow_mut() = Some(dest_row.clone());

            // Resolution status label (below address input)
            let resolution_label = gtk4::Label::new(None);
            resolution_label.set_halign(gtk4::Align::Start);
            resolution_label.set_margin_start(12);
            resolution_label.set_margin_top(4);
            resolution_label.add_css_class("dim-label");
            resolution_label.add_css_class("caption");
            resolution_label.set_visible(false);
            dest_card.add(&resolution_label);
            *self.resolution_label.borrow_mut() = Some(resolution_label);

            // Connect to update send button state and trigger resolution
            dest_row.connect_changed(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_send_button_state();
                    this.schedule_address_resolution();

                    // Hide favorite IP label when address is manually edited
                    let label = this.favorite_ip_label.borrow().clone();
                    if let Some(l) = label {
                        l.set_visible(false);
                    }
                }
            ));

            // Add to favorites button
            let add_fav_row = adw::ActionRow::new();
            add_fav_row.set_title("Save to Favorites");
            add_fav_row.set_subtitle("Save this destination for quick access");
            let add_fav_button = gtk4::Button::from_icon_name("starred-symbolic");
            add_fav_button.set_valign(gtk4::Align::Center);
            add_fav_button.add_css_class("flat");
            add_fav_row.add_suffix(&add_fav_button);
            add_fav_row.set_activatable_widget(Some(&add_fav_button));
            dest_card.add(&add_fav_row);

            add_fav_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |button| {
                    this.show_add_favorite_dialog(button);
                }
            ));

            content.append(&dest_card);

            // Files card
            let files_card = adw::PreferencesGroup::new();
            files_card.set_title("Files or Folder");
            files_card.set_description(Some("Select files or a folder to send"));

            let files_row = adw::ActionRow::new();
            files_row.set_title("Select files or folder");
            files_row.set_subtitle("Nothing selected");

            // Button box for file and folder pickers
            let picker_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

            let browse_button = gtk4::Button::from_icon_name("document-open-symbolic");
            browse_button.set_tooltip_text(Some("Select files"));
            browse_button.set_valign(gtk4::Align::Center);
            browse_button.add_css_class("flat");

            let folder_button = gtk4::Button::from_icon_name("folder-symbolic");
            folder_button.set_tooltip_text(Some("Select folder"));
            folder_button.set_valign(gtk4::Align::Center);
            folder_button.add_css_class("flat");

            picker_box.append(&browse_button);
            picker_box.append(&folder_button);
            files_row.add_suffix(&picker_box);

            files_card.add(&files_row);
            *self.files_row.borrow_mut() = Some(files_row.clone());

            browse_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |button| {
                    this.show_file_chooser(button);
                }
            ));

            folder_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |button| {
                    this.show_folder_chooser(button);
                }
            ));

            content.append(&files_card);

            scrolled.set_child(Some(&content));
            obj.append(&scrolled);

            // Send button
            let send_button = gtk4::Button::with_label("Send Files");
            send_button.add_css_class("suggested-action");
            send_button.add_css_class("pill");
            send_button.set_halign(gtk4::Align::Center);
            send_button.set_sensitive(false);
            send_button.connect_clicked(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.send_files();
                }
            ));
            obj.append(&send_button);

            *self.send_button.borrow_mut() = Some(send_button);
        }

        fn update_send_button_state(&self) {
            let has_dest = self
                .dest_row
                .borrow()
                .as_ref()
                .map(|r| !r.text().is_empty())
                .unwrap_or(false);

            let has_files = !self.selected_files.borrow().is_empty();
            let has_directory = self.selected_directory.borrow().is_some();

            if let Some(button) = self.send_button.borrow().as_ref() {
                button.set_sensitive(has_dest && (has_files || has_directory));
            }
        }

        fn schedule_address_resolution(&self) {
            // Cancel any pending resolution
            if let Some(source_id) = self.resolve_timeout.borrow_mut().take() {
                source_id.remove();
            }

            // Increment generation - invalidates all in-flight callbacks
            let generation = self.resolve_generation.get().wrapping_add(1);
            self.resolve_generation.set(generation);

            let address = self
                .dest_row
                .borrow()
                .as_ref()
                .map(|r| r.text().to_string())
                .unwrap_or_default();

            // Clear label if address is empty
            if address.is_empty() {
                if let Some(label) = self.resolution_label.borrow().as_ref() {
                    label.set_visible(false);
                }
                return;
            }

            // Show "Resolving..." immediately and clear previous state
            if let Some(label) = self.resolution_label.borrow().as_ref() {
                label.set_text("Resolving...");
                label.remove_css_class("success");
                label.remove_css_class("error");
                label.set_visible(true);
            }

            // Schedule resolution with 300ms debounce
            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(300),
                glib::clone!(
                    #[weak(rename_to = this)]
                    self,
                    move || {
                        this.resolve_timeout.borrow_mut().take();
                        this.resolve_address(generation);
                    }
                ),
            );
            *self.resolve_timeout.borrow_mut() = Some(source_id);
        }

        fn resolve_address(&self, expected_generation: u64) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                let address = self
                    .dest_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.text().to_string())
                    .unwrap_or_default();

                if address.is_empty() {
                    return;
                }

                let this_weak = obj.downgrade();
                let favorites_store = app.favorites_store().clone();
                let address_for_update = address.clone();

                app.engine_bridge().resolve_address(address, move |result| {
                    let Some(this_obj) = this_weak.upgrade() else {
                        return;
                    };
                    let this = this_obj.imp();

                    // Check if result is still relevant (generation matches)
                    if this.resolve_generation.get() != expected_generation {
                        tracing::debug!("Ignoring stale resolution result");
                        return;
                    }

                    let label = this.resolution_label.borrow().clone();
                    if let Some(label) = label.as_ref() {
                        if result.success && !result.ips.is_empty() {
                            let ip = &result.ips[0];
                            label.set_text(&format!("Resolved to {}", ip));
                            label.remove_css_class("error");
                            label.add_css_class("success");

                            // Update favorite's last_resolved_ip if this address is a favorite
                            if let Err(e) =
                                favorites_store.update_resolved_ip(&address_for_update, ip)
                            {
                                tracing::debug!("Could not update resolved IP for favorite: {}", e);
                            }
                        } else {
                            let error_msg = result
                                .error
                                .as_deref()
                                .unwrap_or("Could not resolve hostname");
                            label.set_text(error_msg);
                            label.remove_css_class("success");
                            label.add_css_class("error");
                        }
                        label.set_visible(true);
                    }
                });
            }
        }

        fn show_file_chooser(&self, button: &gtk4::Button) {
            let window = button
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());

            let dialog = gtk4::FileChooserDialog::new(
                Some("Select Files to Send"),
                window.as_ref(),
                gtk4::FileChooserAction::Open,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Select", gtk4::ResponseType::Accept),
                ],
            );
            dialog.set_modal(true);
            dialog.set_select_multiple(true);

            dialog.connect_response(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |dialog, response| {
                    if response == gtk4::ResponseType::Accept {
                        let files = dialog.files();
                        let mut paths = Vec::new();

                        for i in 0..files.n_items() {
                            if let Some(file) = files.item(i) {
                                if let Some(gfile) = file.downcast_ref::<gtk4::gio::File>() {
                                    if let Some(path) = gfile.path() {
                                        paths.push(path);
                                    }
                                }
                            }
                        }

                        if !paths.is_empty() {
                            let count = paths.len();
                            // Clear directory selection when files are selected
                            *this.selected_directory.borrow_mut() = None;
                            *this.selected_files.borrow_mut() = paths;

                            if let Some(row) = this.files_row.borrow().as_ref() {
                                let subtitle = if count == 1 {
                                    "1 file selected".to_string()
                                } else {
                                    format!("{} files selected", count)
                                };
                                row.set_subtitle(&subtitle);
                            }

                            this.update_send_button_state();
                        }
                    }
                    dialog.close();
                }
            ));

            dialog.show();
        }

        fn show_folder_chooser(&self, button: &gtk4::Button) {
            let window = button
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());

            let dialog = gtk4::FileChooserDialog::new(
                Some("Select Folder to Send"),
                window.as_ref(),
                gtk4::FileChooserAction::SelectFolder,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Select", gtk4::ResponseType::Accept),
                ],
            );
            dialog.set_modal(true);

            dialog.connect_response(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |dialog, response| {
                    if response == gtk4::ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                // Clear files selection when folder is selected
                                this.selected_files.borrow_mut().clear();
                                *this.selected_directory.borrow_mut() = Some(path.clone());

                                if let Some(row) = this.files_row.borrow().as_ref() {
                                    let name = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "folder".to_string());
                                    row.set_subtitle(&format!("Folder: {}", name));
                                }

                                this.update_send_button_state();
                            }
                        }
                    }
                    dialog.close();
                }
            ));

            dialog.show();
        }

        fn test_connection(&self) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                let address = self
                    .dest_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.text().to_string())
                    .unwrap_or_default();

                if address.is_empty() {
                    return;
                }

                let port = app.settings().port;

                // Show spinner
                if let Some(spinner) = self.test_spinner.borrow().as_ref() {
                    spinner.set_visible(true);
                    spinner.start();
                }
                if let Some(button) = self.test_button.borrow().as_ref() {
                    button.set_sensitive(false);
                }

                let test_button = self.test_button.borrow().clone();
                let test_spinner = self.test_spinner.borrow().clone();

                app.engine_bridge()
                    .check_peer(address, port, move |reachable| {
                        // Hide spinner
                        if let Some(spinner) = test_spinner.as_ref() {
                            spinner.stop();
                            spinner.set_visible(false);
                        }
                        if let Some(button) = test_button.as_ref() {
                            button.set_sensitive(true);
                            if reachable {
                                button.set_icon_name("emblem-ok-symbolic");
                                button.add_css_class("success");
                            } else {
                                button.set_icon_name("dialog-error-symbolic");
                                button.add_css_class("error");
                            }
                            // Reset icon after 3 seconds
                            glib::timeout_add_seconds_local_once(
                                3,
                                glib::clone!(
                                    #[weak]
                                    button,
                                    move || {
                                        button.set_icon_name("network-transmit-symbolic");
                                        button.remove_css_class("success");
                                        button.remove_css_class("error");
                                    }
                                ),
                            );
                        }
                    });
            }
        }

        fn show_add_favorite_dialog(&self, button: &gtk4::Button) {
            let address = self
                .dest_row
                .borrow()
                .as_ref()
                .map(|r| r.text().to_string())
                .unwrap_or_default();

            if address.is_empty() {
                return;
            }

            let window = button
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());

            let dialog = adw::MessageDialog::new(
                window.as_ref(),
                Some("Add to Favorites"),
                Some("Enter a name for this destination"),
            );

            let entry = gtk4::Entry::new();
            entry.set_text(&address); // Default to address as name
            entry.set_placeholder_text(Some("Name"));
            entry.set_margin_start(12);
            entry.set_margin_end(12);
            dialog.set_extra_child(Some(&entry));

            dialog.add_response("cancel", "Cancel");
            dialog.add_response("add", "Add");
            dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);
            dialog.set_default_response(Some("add"));

            dialog.connect_response(
                None,
                glib::clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[weak]
                    entry,
                    move |_, response| {
                        if response == "add" {
                            let name = entry.text().to_string();
                            if !name.is_empty() {
                                this.add_favorite(&name, &address);
                            }
                        }
                    }
                ),
            );

            dialog.present();
        }

        fn add_favorite(&self, name: &str, address: &str) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                let store = app.favorites_store();
                match store.add(name.to_string(), address.to_string()) {
                    Ok(_) => {
                        obj.load_favorites(&app);
                        tracing::info!("Added favorite: {} ({})", name, address);
                    }
                    Err(e) => {
                        tracing::error!("Failed to add favorite: {}", e);
                    }
                }
            }
        }

        fn show_manage_favorites_dialog(&self, button: &gtk4::Button) {
            let obj = self.obj();
            let Some(app) = obj.get_app() else {
                return;
            };

            let window = button
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());

            let dialog = adw::Window::new();
            dialog.set_title(Some("Manage Favorites"));
            dialog.set_default_size(400, 300);
            dialog.set_modal(true);
            if let Some(ref w) = window {
                dialog.set_transient_for(Some(w));
            }

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

            // Header bar
            let header = adw::HeaderBar::new();
            content.append(&header);

            // Scrolled content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_vexpand(true);
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let list_box = gtk4::ListBox::new();
            list_box.set_selection_mode(gtk4::SelectionMode::None);
            list_box.add_css_class("boxed-list");
            list_box.set_margin_start(12);
            list_box.set_margin_end(12);
            list_box.set_margin_top(12);
            list_box.set_margin_bottom(12);

            // Get favorites and populate list
            let favorites = self.favorites_list.borrow().clone();

            if favorites.is_empty() {
                let empty_label = gtk4::Label::new(Some("No favorites saved"));
                empty_label.add_css_class("dim-label");
                empty_label.set_margin_top(24);
                empty_label.set_margin_bottom(24);
                list_box.append(&empty_label);
            } else {
                for (id, name, address, _) in favorites {
                    let row = adw::ActionRow::new();
                    row.set_title(&name);
                    row.set_subtitle(&address);

                    let delete_button = gtk4::Button::from_icon_name("user-trash-symbolic");
                    delete_button.set_tooltip_text(Some("Remove favorite"));
                    delete_button.set_valign(gtk4::Align::Center);
                    delete_button.add_css_class("flat");
                    delete_button.add_css_class("error");
                    row.add_suffix(&delete_button);

                    let dialog_weak = dialog.downgrade();
                    let obj_weak = obj.downgrade();
                    let store = app.favorites_store().clone();
                    let id_clone = id.clone();

                    delete_button.connect_clicked(move |_| {
                        match store.delete(&id_clone) {
                            Ok(_) => {
                                tracing::info!("Deleted favorite: {}", id_clone);
                                // Reload favorites in main view
                                if let Some(obj) = obj_weak.upgrade() {
                                    if let Some(app) = obj.get_app() {
                                        obj.load_favorites(&app);
                                    }
                                }
                                // Close dialog to refresh
                                if let Some(dlg) = dialog_weak.upgrade() {
                                    dlg.close();
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to delete favorite: {}", e);
                            }
                        }
                    });

                    list_box.append(&row);
                }
            }

            scrolled.set_child(Some(&list_box));
            content.append(&scrolled);

            dialog.set_content(Some(&content));
            dialog.present();
        }

        fn send_files(&self) {
            // Wrap in catch_unwind to prevent app crash from panic in folder send
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.send_files_inner();
            })) {
                tracing::error!("Panic in send_files: {:?}", e);
                // Reset UI state after panic
                if let Some(row) = self.files_row.borrow().as_ref() {
                    row.set_subtitle("Send failed - please try again");
                }
                *self.selected_directory.borrow_mut() = None;
                self.selected_files.borrow_mut().clear();
                self.update_send_button_state();
            }
        }

        fn send_files_inner(&self) {
            let obj = self.obj();
            if let Some(app) = obj.get_app() {
                let address = self
                    .dest_row
                    .borrow()
                    .as_ref()
                    .map(|r| r.text().to_string())
                    .unwrap_or_default();

                if address.is_empty() {
                    return;
                }

                let port = app.settings().port;
                let engine = app.engine_bridge();

                // Check if we have a directory to send
                if let Some(dir) = self.selected_directory.borrow().clone() {
                    engine.send_directory(address, port, dir);
                    *self.selected_directory.borrow_mut() = None;
                } else {
                    // Send files
                    let files = self.selected_files.borrow().clone();
                    if !files.is_empty() {
                        engine.send_files(address, port, files);
                        self.selected_files.borrow_mut().clear();
                    }
                }

                // Clear selection after sending
                if let Some(row) = self.files_row.borrow().as_ref() {
                    row.set_subtitle("Nothing selected");
                }
                self.update_send_button_state();
            }
        }
    }
}

glib::wrapper! {
    pub struct SendView(ObjectSubclass<imp::SendView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SendView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Load favorites from the application
    pub fn load_favorites(&self, app: &GoshTransferApplication) {
        let imp = self.imp();
        let store = app.favorites_store();

        match store.list() {
            Ok(favorites) => {
                let mut names = Vec::new();
                let mut list = Vec::new();

                for fav in favorites {
                    names.push(format!("{} ({})", fav.name, fav.address));
                    list.push((
                        fav.id.clone(),
                        fav.name.clone(),
                        fav.address.clone(),
                        fav.last_resolved_ip.clone(),
                    ));
                }

                *imp.favorites_list.borrow_mut() = list;

                if let Some(dropdown) = imp.favorites_dropdown.borrow().as_ref() {
                    if names.is_empty() {
                        let model = gtk4::StringList::new(&["No favorites saved"]);
                        dropdown.set_model(Some(&model));
                        dropdown.set_sensitive(false);
                    } else {
                        let model = gtk4::StringList::new(
                            &names.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        );
                        dropdown.set_model(Some(&model));
                        dropdown.set_sensitive(true);
                        // Reset selection to trigger the notify signal
                        // Setting to INVALID_LIST_POSITION first ensures setting to 0 fires the signal
                        dropdown.set_selected(gtk4::INVALID_LIST_POSITION);
                        dropdown.set_selected(0);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load favorites: {}", e);
            }
        }
    }

    fn get_app(&self) -> Option<GoshTransferApplication> {
        self.root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok())
            .and_then(|w| w.application())
            .and_then(|a| a.downcast::<GoshTransferApplication>().ok())
    }
}

impl Default for SendView {
    fn default() -> Self {
        Self::new()
    }
}
