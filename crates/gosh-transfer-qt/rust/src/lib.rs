// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Qt6/QML frontend library

pub mod controllers;
pub mod engine_bridge;

use cxx_qt_lib::QString;

/// Application version
pub fn version() -> QString {
    QString::from(env!("CARGO_PKG_VERSION"))
}
