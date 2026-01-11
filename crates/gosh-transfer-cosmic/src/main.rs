// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - libcosmic frontend

mod app;
mod config;
mod engine;
mod message;
mod pages;

use cosmic::app::Settings;

const APP_ID: &str = "com.gosh.transfer";

fn main() -> cosmic::iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gosh_transfer_cosmic=info".parse().unwrap())
                .add_directive("gosh_transfer_core=info".parse().unwrap())
                .add_directive("gosh_lan_transfer=info".parse().unwrap()),
        )
        .init();

    tracing::info!(
        "Starting Gosh Transfer COSMIC v{}",
        env!("CARGO_PKG_VERSION")
    );

    let settings = Settings::default()
        .size_limits(cosmic::iced::Limits::NONE.min_width(800.0).min_height(600.0));

    cosmic::app::run::<app::App>(settings, app::Flags::default())
}
