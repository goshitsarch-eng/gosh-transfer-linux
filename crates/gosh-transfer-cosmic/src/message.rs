// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Messages

use crate::engine::EngineMessage;
use crate::pages;
use cosmic::widget::nav_bar;

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    #[allow(dead_code)] // Used by nav_bar internally
    Nav(nav_bar::Id),

    // Page-specific messages
    Send(pages::send::SendMessage),
    Receive(pages::receive::ReceiveMessage),
    Transfers(pages::transfers::TransfersMessage),
    Settings(pages::settings::SettingsMessage),
    About(pages::about::AboutMessage),

    // Engine events
    Engine(EngineMessage),
}

impl From<pages::send::SendMessage> for Message {
    fn from(msg: pages::send::SendMessage) -> Self {
        Message::Send(msg)
    }
}

impl From<pages::receive::ReceiveMessage> for Message {
    fn from(msg: pages::receive::ReceiveMessage) -> Self {
        Message::Receive(msg)
    }
}

impl From<EngineMessage> for Message {
    fn from(msg: EngineMessage) -> Self {
        Message::Engine(msg)
    }
}
