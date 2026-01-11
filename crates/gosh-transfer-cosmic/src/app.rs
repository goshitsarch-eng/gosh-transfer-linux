// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Application

use crate::config::CosmicConfig;
use crate::engine::EngineBridge;
use crate::message::Message;
use crate::pages::{self, PageId};
use cosmic::app::{Core, Task};
use cosmic::iced::Length;
use cosmic::widget::nav_bar;
use cosmic::{executor, Action, Application, Element};
use std::sync::Arc;

pub struct Flags {
    pub config: CosmicConfig,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            config: CosmicConfig::default(),
        }
    }
}

pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    active_page: PageId,

    // Page states
    send_page: pages::send::SendPage,
    receive_page: pages::receive::ReceivePage,
    transfers_page: pages::transfers::TransfersPage,
    settings_page: pages::settings::SettingsPage,
    about_page: pages::about::AboutPage,

    // Engine bridge
    engine: Arc<EngineBridge>,

    // Settings
    config: CosmicConfig,

    // Navigation badge tracking
    receive_nav_id: nav_bar::Id,
    pending_count: usize,
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = Message;

    const APP_ID: &'static str = crate::APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Build navigation model
        let mut nav_model = nav_bar::Model::default();

        nav_model
            .insert()
            .text("Send")
            .icon(cosmic::widget::icon::from_name("document-send-symbolic"))
            .data(PageId::Send);

        let receive_nav_id = nav_model
            .insert()
            .text("Receive")
            .icon(cosmic::widget::icon::from_name("document-save-symbolic"))
            .data(PageId::Receive)
            .id();

        nav_model
            .insert()
            .text("Transfers")
            .icon(cosmic::widget::icon::from_name("folder-download-symbolic"))
            .data(PageId::Transfers);

        nav_model
            .insert()
            .text("Settings")
            .icon(cosmic::widget::icon::from_name("preferences-system-symbolic"))
            .data(PageId::Settings);

        nav_model
            .insert()
            .text("About")
            .icon(cosmic::widget::icon::from_name("help-about-symbolic"))
            .data(PageId::About);

        nav_model.activate_position(0);

        // Initialize engine bridge
        let engine = Arc::new(EngineBridge::new(flags.config.to_engine_config()));

        let app = App {
            core,
            nav_model,
            active_page: PageId::Send,
            send_page: pages::send::SendPage::new(),
            receive_page: pages::receive::ReceivePage::new(),
            transfers_page: pages::transfers::TransfersPage::new(),
            settings_page: pages::settings::SettingsPage::new(&flags.config),
            about_page: pages::about::AboutPage::new(),
            engine,
            config: flags.config,
            receive_nav_id,
            pending_count: 0,
        };

        // Start the server on init
        let start_server: Task<Message> = cosmic::task::future(async {
            Message::Engine(crate::engine::EngineMessage::StartServer)
        });

        (app, start_server)
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);

        if let Some(page_id) = self.nav_model.active_data::<PageId>() {
            self.active_page = page_id.clone();
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<Message> = match self.active_page {
            PageId::Send => self.send_page.view().map(Message::Send),
            PageId::Receive => self.receive_page.view().map(Message::Receive),
            PageId::Transfers => self.transfers_page.view().map(Message::Transfers),
            PageId::Settings => self.settings_page.view().map(Message::Settings),
            PageId::About => self.about_page.view().map(Message::About),
        };

        cosmic::widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Nav(id) => self.on_nav_select(id),
            Message::Send(msg) => self
                .send_page
                .update(msg, &self.engine)
                .map(Message::Send)
                .map(Action::from),
            Message::Receive(msg) => {
                let task = self
                    .receive_page
                    .update(msg, &self.engine)
                    .map(Message::Receive)
                    .map(Action::from);
                self.update_pending_badge();
                task
            }
            Message::Transfers(msg) => self
                .transfers_page
                .update(msg)
                .map(Message::Transfers)
                .map(Action::from),
            Message::Settings(msg) => {
                let task = self.settings_page.update(msg, &mut self.config);
                self.engine.update_config(self.config.to_engine_config());
                task.map(Message::Settings).map(Action::from)
            }
            Message::About(msg) => self
                .about_page
                .update(msg)
                .map(Message::About)
                .map(Action::from),
            Message::Engine(msg) => self.handle_engine_message(msg),
        }
    }
}

impl App {
    fn handle_engine_message(&mut self, msg: crate::engine::EngineMessage) -> Task<Message> {
        match msg {
            crate::engine::EngineMessage::StartServer => {
                let engine = self.engine.clone();
                cosmic::task::future(async move {
                    engine.start_server().await;
                    Message::Engine(crate::engine::EngineMessage::ServerStarted)
                })
            }
            crate::engine::EngineMessage::ServerStarted => {
                tracing::info!("Server started");
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// Update the receive nav item text to show pending count as a badge
    fn update_pending_badge(&mut self) {
        let new_count = self.receive_page.pending_count();
        if new_count != self.pending_count {
            self.pending_count = new_count;
            let badge_text = if new_count > 0 {
                format!("Receive ({})", new_count)
            } else {
                "Receive".to_string()
            };
            self.nav_model.text_set(self.receive_nav_id, badge_text);
        }
    }
}
