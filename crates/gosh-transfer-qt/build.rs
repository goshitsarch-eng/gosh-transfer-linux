// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Build script

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .qml_module(QmlModule {
            uri: "GoshTransfer",
            rust_files: &[
                "rust/src/controllers/mod.rs",
            ],
            qml_files: &[
                "qml/main.qml",
            ],
            ..Default::default()
        })
        .build();
}
