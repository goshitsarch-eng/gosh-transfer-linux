// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Application

use crate::services::EngineBridge;
use crate::window::GoshTransferWindow;
use gosh_transfer_core::{AppSettings, FileFavoritesStore, SettingsStore, TransferHistory};
use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::gio;
use libadwaita as adw;
use std::cell::OnceCell;
use std::sync::Arc;

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;
    use libadwaita::subclass::prelude::*;

    #[derive(Default)]
    pub struct GoshTransferApplication {
        pub settings_store: OnceCell<SettingsStore>,
        pub favorites_store: OnceCell<Arc<FileFavoritesStore>>,
        pub history: OnceCell<Arc<TransferHistory>>,
        pub engine_bridge: OnceCell<EngineBridge>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GoshTransferApplication {
        const NAME: &'static str = "GoshTransferApplication";
        type Type = super::GoshTransferApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GoshTransferApplication {}

    impl ApplicationImpl for GoshTransferApplication {
        fn activate(&self) {
            let app = self.obj();

            // Initialize stores if not already done
            if self.settings_store.get().is_none() {
                app.init_stores();
            }

            // Create window
            let window = GoshTransferWindow::new(&app);

            // Setup engine events after window is fully constructed
            window.setup_engine_events(&app);

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            // Set up application actions
            let app = self.obj();
            app.setup_actions();
        }
    }

    impl GtkApplicationImpl for GoshTransferApplication {}
    impl AdwApplicationImpl for GoshTransferApplication {}
}

glib::wrapper! {
    pub struct GoshTransferApplication(ObjectSubclass<imp::GoshTransferApplication>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GoshTransferApplication {
    pub fn new(app_id: &str) -> Self {
        glib::Object::builder()
            .property("application-id", app_id)
            .property("flags", gio::ApplicationFlags::default())
            .build()
    }

    fn init_stores(&self) {
        let imp = self.imp();

        // Initialize settings
        let settings_store = SettingsStore::new().expect("Failed to initialize settings");
        let settings = settings_store.get();

        // Initialize favorites
        let favorites_store =
            Arc::new(FileFavoritesStore::new().expect("Failed to initialize favorites"));

        // Initialize history
        let history = Arc::new(TransferHistory::new().expect("Failed to initialize history"));

        // Initialize engine bridge
        let engine_bridge = EngineBridge::new(settings.to_engine_config());

        // Store all
        let _ = imp.settings_store.set(settings_store);
        let _ = imp.favorites_store.set(favorites_store);
        let _ = imp.history.set(history);
        let _ = imp.engine_bridge.set(engine_bridge);

        // Apply theme
        self.apply_theme(&settings.theme);
    }

    fn setup_actions(&self) {
        // Quit action
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.quit();
            }
        ));
        self.add_action(&quit_action);

        // About action
        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.show_about_dialog();
            }
        ));
        self.add_action(&about_action);

        // Set up keyboard shortcuts
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn show_about_dialog(&self) {
        let window = self.active_window();

        let dialog = adw::AboutWindow::builder()
            .application_name("Gosh Transfer")
            .application_icon("com.gosh.Transfer")
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("Gosh Contributors")
            .license_type(gtk4::License::Agpl30)
            .website("https://github.com/gosh-sh/gosh-transfer")
            .issue_url("https://github.com/gosh-sh/gosh-transfer/issues")
            .build();

        if let Some(win) = window {
            dialog.set_transient_for(Some(&win));
        }
        dialog.present();
    }

    pub fn apply_theme(&self, theme: &str) {
        let style_manager = adw::StyleManager::default();

        let color_scheme = match theme {
            "dark" => adw::ColorScheme::ForceDark,
            "light" => adw::ColorScheme::ForceLight,
            _ => adw::ColorScheme::Default,
        };

        style_manager.set_color_scheme(color_scheme);
    }

    pub fn settings_store(&self) -> &SettingsStore {
        self.imp()
            .settings_store
            .get()
            .expect("Settings not initialized")
    }

    pub fn favorites_store(&self) -> &Arc<FileFavoritesStore> {
        self.imp()
            .favorites_store
            .get()
            .expect("Favorites not initialized")
    }

    pub fn history(&self) -> &Arc<TransferHistory> {
        self.imp().history.get().expect("History not initialized")
    }

    pub fn engine_bridge(&self) -> &EngineBridge {
        self.imp()
            .engine_bridge
            .get()
            .expect("Engine not initialized")
    }

    pub fn settings(&self) -> AppSettings {
        self.settings_store().get()
    }
}
