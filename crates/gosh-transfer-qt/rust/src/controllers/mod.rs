// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Controllers module

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, app_name)]
        #[qproperty(QString, version)]
        type AppController = super::AppControllerRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn quit(self: &AppController);
    }
}

use cxx_qt_lib::QString;

/// Rust implementation for AppController
#[derive(Default)]
pub struct AppControllerRust {
    app_name: QString,
    version: QString,
}

impl AppControllerRust {
    pub fn new() -> Self {
        Self {
            app_name: QString::from("Gosh Transfer"),
            version: QString::from(env!("CARGO_PKG_VERSION")),
        }
    }
}

impl qobject::AppController {
    pub fn quit(self: &qobject::AppController) {
        std::process::exit(0);
    }
}
