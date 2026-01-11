// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Transfers View (History)

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct TransfersView {
        pub history_list: RefCell<Option<gtk4::ListBox>>,
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
            header_box.append(&clear_button);

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

            scrolled.set_child(Some(&history_group));
            obj.append(&scrolled);
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
}

impl Default for TransfersView {
    fn default() -> Self {
        Self::new()
    }
}
