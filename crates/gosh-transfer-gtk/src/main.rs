// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - GTK4/Libadwaita frontend

mod application;
mod services;
mod views;
mod widgets;
mod window;

use gtk4::prelude::*;

const APP_ID: &str = "com.gosh.Transfer";

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gosh_transfer_gtk=info".parse().unwrap())
                .add_directive("gosh_transfer_core=info".parse().unwrap())
                .add_directive("gosh_lan_transfer=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Gosh Transfer GTK v{}", env!("CARGO_PKG_VERSION"));

    // Create and run application
    let app = application::GoshTransferApplication::new(APP_ID);
    app.run()
}
