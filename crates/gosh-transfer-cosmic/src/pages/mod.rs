// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Pages module

pub mod about;
pub mod receive;
pub mod send;
pub mod settings;
pub mod transfers;

#[derive(Debug, Clone, PartialEq)]
pub enum PageId {
    Send,
    Receive,
    Transfers,
    Settings,
    About,
}
