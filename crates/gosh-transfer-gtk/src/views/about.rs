// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - About View

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct AboutView {}

    #[glib::object_subclass]
    impl ObjectSubclass for AboutView {
        const NAME: &'static str = "GoshAboutView";
        type Type = super::AboutView;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AboutView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for AboutView {}
    impl BoxImpl for AboutView {}

    impl AboutView {
        fn setup_ui(&self) {
            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Vertical);
            obj.set_spacing(24);
            obj.set_margin_start(24);
            obj.set_margin_end(24);
            obj.set_margin_top(48);
            obj.set_margin_bottom(24);
            obj.set_halign(gtk4::Align::Center);
            obj.set_valign(gtk4::Align::Center);

            // Logo
            let logo = gtk4::Image::from_icon_name("folder-download-symbolic");
            logo.set_pixel_size(128);
            logo.add_css_class("icon-dropshadow");
            obj.append(&logo);

            // App name
            let name = gtk4::Label::new(Some("Gosh Transfer"));
            name.add_css_class("title-1");
            obj.append(&name);

            // Version
            let version = gtk4::Label::new(Some(&format!("Version {}", env!("CARGO_PKG_VERSION"))));
            version.add_css_class("dim-label");
            obj.append(&version);

            // Description
            let desc = gtk4::Label::new(Some(
                "A clean, explicit file transfer application.\nNo cloud. No sync. Just transfer."
            ));
            desc.set_justify(gtk4::Justification::Center);
            desc.add_css_class("body");
            obj.append(&desc);

            // Links
            let links_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            links_box.set_halign(gtk4::Align::Center);

            let website_btn = gtk4::LinkButton::with_label(
                "https://github.com/gosh-sh/gosh-transfer",
                "Website"
            );
            links_box.append(&website_btn);

            let issues_btn = gtk4::LinkButton::with_label(
                "https://github.com/gosh-sh/gosh-transfer/issues",
                "Report Issue"
            );
            links_box.append(&issues_btn);

            obj.append(&links_box);

            // License
            let license = gtk4::Label::new(Some("Licensed under AGPL-3.0"));
            license.add_css_class("dim-label");
            license.add_css_class("caption");
            obj.append(&license);

            // Copyright
            let copyright = gtk4::Label::new(Some("Copyright (c) 2024 Gosh Contributors"));
            copyright.add_css_class("dim-label");
            copyright.add_css_class("caption");
            obj.append(&copyright);
        }
    }
}

glib::wrapper! {
    pub struct AboutView(ObjectSubclass<imp::AboutView>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl AboutView {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for AboutView {
    fn default() -> Self {
        Self::new()
    }
}
