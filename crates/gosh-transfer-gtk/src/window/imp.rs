// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Main Window Implementation

use crate::application::GoshTransferApplication;
use crate::views::{AboutView, ReceiveView, SendView, SettingsView, TransfersView};
use gosh_lan_transfer::EngineEvent;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::CompositeTemplate;
use libadwaita as adw;
use libadwaita::subclass::prelude::*;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default, CompositeTemplate)]
#[template(string = r#"
<?xml version="1.0" encoding="UTF-8"?>
<interface>
  <template class="GoshTransferWindow" parent="AdwApplicationWindow">
    <property name="default-width">1024</property>
    <property name="default-height">768</property>
    <property name="title">Gosh Transfer</property>
    <property name="content">
      <object class="AdwToolbarView">
        <child type="top">
          <object class="AdwHeaderBar">
            <property name="title-widget">
              <object class="AdwWindowTitle">
                <property name="title">Gosh Transfer</property>
              </object>
            </property>
          </object>
        </child>
        <property name="content">
          <object class="GtkBox">
            <property name="orientation">horizontal</property>
            <!-- Sidebar -->
            <child>
              <object class="GtkBox" id="sidebar">
                <property name="orientation">vertical</property>
                <property name="width-request">220</property>
                <style>
                  <class name="sidebar-pane"/>
                </style>
                <!-- Header -->
                <child>
                  <object class="GtkBox">
                    <property name="orientation">horizontal</property>
                    <property name="spacing">12</property>
                    <property name="margin-start">16</property>
                    <property name="margin-end">16</property>
                    <property name="margin-top">16</property>
                    <property name="margin-bottom">16</property>
                    <child>
                      <object class="GtkImage">
                        <property name="icon-name">folder-download-symbolic</property>
                        <property name="pixel-size">24</property>
                      </object>
                    </child>
                    <child>
                      <object class="GtkLabel">
                        <property name="label">Gosh Transfer</property>
                        <style>
                          <class name="title-4"/>
                        </style>
                      </object>
                    </child>
                  </object>
                </child>
                <child>
                  <object class="GtkSeparator"/>
                </child>
                <!-- Navigation -->
                <child>
                  <object class="GtkScrolledWindow">
                    <property name="vexpand">true</property>
                    <property name="hscrollbar-policy">never</property>
                    <child>
                      <object class="GtkListBox" id="nav_list">
                        <property name="selection-mode">single</property>
                        <style>
                          <class name="navigation-sidebar"/>
                        </style>
                      </object>
                    </child>
                  </object>
                </child>
                <child>
                  <object class="GtkSeparator"/>
                </child>
                <!-- Server Status -->
                <child>
                  <object class="GtkBox" id="status_box">
                    <property name="orientation">horizontal</property>
                    <property name="spacing">8</property>
                    <property name="margin-start">16</property>
                    <property name="margin-end">16</property>
                    <property name="margin-top">12</property>
                    <property name="margin-bottom">12</property>
                    <child>
                      <object class="GtkBox" id="status_indicator">
                        <property name="width-request">8</property>
                        <property name="height-request">8</property>
                        <property name="valign">center</property>
                        <style>
                          <class name="status-dot"/>
                          <class name="online"/>
                        </style>
                      </object>
                    </child>
                    <child>
                      <object class="GtkLabel" id="status_label">
                        <property name="label">Port 53317</property>
                        <style>
                          <class name="dim-label"/>
                        </style>
                      </object>
                    </child>
                  </object>
                </child>
              </object>
            </child>
            <!-- Separator -->
            <child>
              <object class="GtkSeparator">
                <property name="orientation">vertical</property>
              </object>
            </child>
            <!-- Main Content -->
            <child>
              <object class="GtkStack" id="content_stack">
                <property name="hexpand">true</property>
                <property name="vexpand">true</property>
                <property name="transition-type">crossfade</property>
              </object>
            </child>
          </object>
        </property>
      </object>
    </property>
  </template>
</interface>
"#)]
pub struct GoshTransferWindow {
    #[template_child]
    pub nav_list: TemplateChild<gtk4::ListBox>,
    #[template_child]
    pub content_stack: TemplateChild<gtk4::Stack>,
    #[template_child]
    pub status_label: TemplateChild<gtk4::Label>,
    #[template_child]
    pub status_indicator: TemplateChild<gtk4::Box>,

    pub send_view: RefCell<Option<SendView>>,
    pub receive_view: RefCell<Option<ReceiveView>>,
    pub transfers_view: RefCell<Option<TransfersView>>,
    pub settings_view: RefCell<Option<SettingsView>>,
    pub about_view: RefCell<Option<AboutView>>,

    pub receive_badge: RefCell<Option<gtk4::Label>>,
    pub pending_count: Cell<u32>,
    current_view: Cell<usize>,
}

#[glib::object_subclass]
impl ObjectSubclass for GoshTransferWindow {
    const NAME: &'static str = "GoshTransferWindow";
    type Type = super::GoshTransferWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl GoshTransferWindow {
    fn setup_navigation(&self) {
        let nav_items = [
            ("document-send-symbolic", "Send", false),
            ("document-save-symbolic", "Receive", true), // Has badge
            ("folder-download-symbolic", "Transfers", false),
            ("preferences-system-symbolic", "Settings", false),
            ("help-about-symbolic", "About", false),
        ];

        for (icon_name, label, has_badge) in nav_items {
            let row = gtk4::ListBoxRow::new();
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);

            let icon = gtk4::Image::from_icon_name(icon_name);
            let text = gtk4::Label::new(Some(label));
            text.set_halign(gtk4::Align::Start);
            text.set_hexpand(true);

            hbox.append(&icon);
            hbox.append(&text);

            // Add badge for Receive
            if has_badge {
                let badge = gtk4::Label::new(None);
                badge.set_halign(gtk4::Align::End);
                badge.set_valign(gtk4::Align::Center);
                badge.set_visible(false);

                // Style the badge as a circular notification indicator
                let css_provider = gtk4::CssProvider::new();
                css_provider.load_from_data(
                    r#"
                    label.notification-badge {
                        background-color: @accent_bg_color;
                        color: @accent_fg_color;
                        border-radius: 10px;
                        padding: 2px 6px;
                        font-size: 11px;
                        font-weight: bold;
                        min-width: 18px;
                        min-height: 18px;
                    }
                    "#,
                );
                badge
                    .style_context()
                    .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
                badge.add_css_class("notification-badge");

                hbox.append(&badge);
                *self.receive_badge.borrow_mut() = Some(badge);
            }

            row.set_child(Some(&hbox));
            self.nav_list.append(&row);
        }

        // Select first row by default
        if let Some(row) = self.nav_list.row_at_index(0) {
            self.nav_list.select_row(Some(&row));
        }
    }

    /// Update the pending transfer badge count
    pub fn update_pending_badge(&self, count: u32) {
        self.pending_count.set(count);
        if let Some(badge) = self.receive_badge.borrow().as_ref() {
            if count > 0 {
                badge.set_text(&count.to_string());
                badge.set_visible(true);
            } else {
                badge.set_visible(false);
            }
        }
    }

    /// Increment pending count
    pub fn increment_pending(&self) {
        let count = self.pending_count.get() + 1;
        self.update_pending_badge(count);
    }

    /// Decrement pending count
    pub fn decrement_pending(&self) {
        let count = self.pending_count.get().saturating_sub(1);
        self.update_pending_badge(count);
    }

    fn setup_views(&self) {
        // Create views
        let send_view = SendView::new();
        let receive_view = ReceiveView::new();
        let transfers_view = TransfersView::new();
        let settings_view = SettingsView::new();
        let about_view = AboutView::new();

        // Add to stack
        self.content_stack.add_named(&send_view, Some("send"));
        self.content_stack.add_named(&receive_view, Some("receive"));
        self.content_stack
            .add_named(&transfers_view, Some("transfers"));
        self.content_stack
            .add_named(&settings_view, Some("settings"));
        self.content_stack.add_named(&about_view, Some("about"));

        // Store references
        *self.send_view.borrow_mut() = Some(send_view);
        *self.receive_view.borrow_mut() = Some(receive_view);
        *self.transfers_view.borrow_mut() = Some(transfers_view);
        *self.settings_view.borrow_mut() = Some(settings_view);
        *self.about_view.borrow_mut() = Some(about_view);

        // Show first view
        self.content_stack.set_visible_child_name("send");
    }

    fn setup_signals(&self) {
        // Navigation selection
        self.nav_list.connect_row_activated(glib::clone!(
            #[weak(rename_to = this)]
            self,
            move |_, row| {
                let view_name = match row.index() {
                    0 => "send",
                    1 => "receive",
                    2 => "transfers",
                    3 => "settings",
                    4 => "about",
                    _ => return,
                };
                this.content_stack.set_visible_child_name(view_name);
                this.current_view.set(row.index() as usize);
            }
        ));
    }

    pub fn setup_engine_events(&self, app: &GoshTransferApplication) {
        // Start server
        app.engine_bridge().start_server();

        // Update status with port
        let settings = app.settings();
        self.status_label
            .set_text(&format!("Port {}", settings.port));

        // Load settings into settings view
        if let Some(settings_view) = self.settings_view.borrow().as_ref() {
            settings_view.load_settings(app);
        }

        // Load favorites into send view
        if let Some(send_view) = self.send_view.borrow().as_ref() {
            send_view.load_favorites(app);
        }

        // Load receive view data
        if let Some(receive_view) = self.receive_view.borrow().as_ref() {
            receive_view.load_data(app);
        }

        // Load transfer history
        if let Some(transfers_view) = self.transfers_view.borrow().as_ref() {
            transfers_view.load_history(app);
        }

        // Subscribe to engine events
        let event_rx = app.engine_bridge().event_receiver();
        let receive_view = self.receive_view.borrow().clone();
        let transfers_view = self.transfers_view.borrow().clone();
        let receive_badge = self.receive_badge.borrow().clone();
        let pending_count = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let app_weak = app.downgrade();

        // Track pending transfers for title lookup and badge state
        let pending_info: std::rc::Rc<
            std::cell::RefCell<std::collections::HashMap<String, String>>,
        > = std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
        // Track which transfers have been converted to active (to avoid double badge decrement)
        let active_transfers: std::rc::Rc<std::cell::RefCell<std::collections::HashSet<String>>> =
            std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashSet::new()));
        // Track which transfers were manually handled by the user (to avoid double badge decrement)
        let ui_handled_transfers: std::rc::Rc<
            std::cell::RefCell<std::collections::HashSet<String>>,
        > = std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashSet::new()));

        // Wire up callback for when user manually accepts/rejects transfers
        if let Some(view) = receive_view.as_ref() {
            let pending_count_clone = pending_count.clone();
            let receive_badge_clone = receive_badge.clone();
            let pending_info_clone = pending_info.clone();
            let ui_handled_clone = ui_handled_transfers.clone();

            view.set_on_pending_handled(move |ids| {
                let count = ids.len() as u32;
                let current = pending_count_clone.get();
                let new_count = current.saturating_sub(count);
                pending_count_clone.set(new_count);

                if let Some(badge) = receive_badge_clone.as_ref() {
                    if new_count > 0 {
                        badge.set_text(&new_count.to_string());
                    } else {
                        badge.set_visible(false);
                    }
                }

                // Remove from pending_info and mark as handled
                let mut info = pending_info_clone.borrow_mut();
                let mut handled = ui_handled_clone.borrow_mut();
                for id in ids {
                    info.remove(id);
                    handled.insert(id.clone());
                }
            });
        }

        glib::spawn_future_local(glib::clone!(
            #[strong]
            pending_info,
            #[strong]
            pending_count,
            #[strong]
            active_transfers,
            #[strong]
            ui_handled_transfers,
            async move {
                // Helper to decrement badge
                let decrement_badge =
                    |count: &std::cell::Cell<u32>, badge: &Option<gtk4::Label>| {
                        let new_count = count.get().saturating_sub(1);
                        count.set(new_count);
                        if let Some(b) = badge.as_ref() {
                            if new_count > 0 {
                                b.set_text(&new_count.to_string());
                            } else {
                                b.set_visible(false);
                            }
                        }
                    };

                while let Ok(event) = event_rx.recv().await {
                    let Some(app) = app_weak.upgrade() else {
                        break;
                    };

                    match event {
                        EngineEvent::TransferRequest(transfer) => {
                            tracing::info!("UI received transfer request: {}", transfer.id);

                            // Store title for later use
                            let title = if transfer.files.len() == 1 {
                                transfer.files[0].name.clone()
                            } else {
                                format!("{} files", transfer.files.len())
                            };
                            pending_info.borrow_mut().insert(transfer.id.clone(), title);

                            // Update badge - increment
                            let count = pending_count.get() + 1;
                            pending_count.set(count);
                            if let Some(badge) = receive_badge.as_ref() {
                                badge.set_text(&count.to_string());
                                badge.set_visible(true);
                            }

                            if let Some(view) = receive_view.as_ref() {
                                view.add_pending_transfer(&transfer, &app);
                            }
                        }
                        EngineEvent::TransferProgress(progress) => {
                            tracing::debug!(
                                "Transfer progress: {} - {}/{}",
                                progress.transfer_id,
                                progress.bytes_transferred,
                                progress.total_bytes
                            );

                            // Check if this is a new active transfer (first progress event)
                            let is_new = !active_transfers.borrow().contains(&progress.transfer_id);
                            if is_new {
                                active_transfers
                                    .borrow_mut()
                                    .insert(progress.transfer_id.clone());
                                // Only decrement badge if not already handled by UI action
                                if !ui_handled_transfers
                                    .borrow()
                                    .contains(&progress.transfer_id)
                                {
                                    decrement_badge(&pending_count, &receive_badge);
                                }
                            }

                            if let Some(view) = receive_view.as_ref() {
                                // Get title from pending info
                                let title = pending_info
                                    .borrow()
                                    .get(&progress.transfer_id)
                                    .cloned()
                                    .unwrap_or_else(|| "Transfer".to_string());

                                // Add to active if not already there, then update progress
                                view.add_active_transfer(&progress.transfer_id, &title, &app);
                                view.update_transfer_progress(
                                    &progress.transfer_id,
                                    progress.bytes_transferred,
                                    progress.total_bytes,
                                    progress.speed_bps,
                                );
                            }
                        }
                        EngineEvent::TransferComplete { transfer_id } => {
                            tracing::info!("Transfer completed: {}", transfer_id);
                            pending_info.borrow_mut().remove(&transfer_id);
                            active_transfers.borrow_mut().remove(&transfer_id);
                            ui_handled_transfers.borrow_mut().remove(&transfer_id);

                            if let Some(view) = receive_view.as_ref() {
                                view.remove_pending_transfer(&transfer_id);
                                view.mark_transfer_complete(&transfer_id);
                            }

                            // Refresh history view
                            if let Some(view) = transfers_view.as_ref() {
                                view.load_history(&app);
                            }
                        }
                        EngineEvent::TransferFailed { transfer_id, error } => {
                            tracing::error!("Transfer failed: {} - {}", transfer_id, error);

                            // If it failed while still pending (not yet active), decrement badge
                            // but only if not already handled by UI action
                            if !active_transfers.borrow().contains(&transfer_id)
                                && !ui_handled_transfers.borrow().contains(&transfer_id)
                            {
                                decrement_badge(&pending_count, &receive_badge);
                            }

                            pending_info.borrow_mut().remove(&transfer_id);
                            active_transfers.borrow_mut().remove(&transfer_id);
                            ui_handled_transfers.borrow_mut().remove(&transfer_id);

                            if let Some(view) = receive_view.as_ref() {
                                view.remove_pending_transfer(&transfer_id);
                                view.mark_transfer_failed(&transfer_id, &error);
                            }
                        }
                        EngineEvent::ServerStarted { port } => {
                            tracing::info!("Server started on port {}", port);
                        }
                        EngineEvent::ServerStopped => {
                            tracing::info!("Server stopped");
                        }
                        EngineEvent::TransferRetry {
                            transfer_id,
                            attempt,
                            max_attempts,
                            ..
                        } => {
                            tracing::info!(
                                "Transfer retry: {} - attempt {}/{}",
                                transfer_id,
                                attempt,
                                max_attempts
                            );
                            if let Some(view) = receive_view.as_ref() {
                                view.update_transfer_retry(&transfer_id, attempt, max_attempts);
                            }
                        }
                        EngineEvent::PortChanged { old_port, new_port } => {
                            tracing::info!("Port changed from {} to {}", old_port, new_port);
                        }
                    }
                }
            }
        ));
    }
}

impl ObjectImpl for GoshTransferWindow {
    fn constructed(&self) {
        self.parent_constructed();
        self.setup_navigation();
        self.setup_views();
        self.setup_signals();
        // Note: setup_engine_events is called from application.rs after window is fully constructed
    }
}

impl WidgetImpl for GoshTransferWindow {}
impl WindowImpl for GoshTransferWindow {}
impl ApplicationWindowImpl for GoshTransferWindow {}
impl AdwApplicationWindowImpl for GoshTransferWindow {}
