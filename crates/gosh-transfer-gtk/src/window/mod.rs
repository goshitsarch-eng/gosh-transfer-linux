// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Main Window

mod imp;

use crate::application::GoshTransferApplication;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use libadwaita as adw;

glib::wrapper! {
    pub struct GoshTransferWindow(ObjectSubclass<imp::GoshTransferWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget,
                    gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl GoshTransferWindow {
    pub fn new(app: &GoshTransferApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn setup_engine_events(&self, app: &GoshTransferApplication) {
        self.imp().setup_engine_events(app);
    }
}
