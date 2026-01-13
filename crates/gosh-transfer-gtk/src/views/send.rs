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
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct SendView {
        pub dest_row: RefCell<Option<adw::EntryRow>>,
        pub test_button: RefCell<Option<gtk4::Button>>,
        pub test_spinner: RefCell<Option<gtk4::Spinner>>,
        pub favorites_dropdown: RefCell<Option<adw::ComboRow>>,
        pub favorites_list: RefCell<Vec<(String, String, String)>>, // (id, name, address)
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
            favorites_card.add(&favorites_dropdown);
            *self.favorites_dropdown.borrow_mut() = Some(favorites_dropdown.clone());

            // Connect favorite selection
            favorites_dropdown.connect_selected_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |dropdown| {
                    let selected = dropdown.selected() as usize;
                    let favorites = this.favorites_list.borrow();
                    if selected < favorites.len() {
                        let (_, _, address) = &favorites[selected];
                        if let Some(dest_row) = this.dest_row.borrow().as_ref() {
                            dest_row.set_text(address);
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

            // Connect to update send button state
            dest_row.connect_changed(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_send_button_state();
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

                app.engine_bridge().check_peer(address, port, move |reachable| {
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

        fn send_files(&self) {
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
                    list.push((fav.id.clone(), fav.name.clone(), fav.address.clone()));
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
