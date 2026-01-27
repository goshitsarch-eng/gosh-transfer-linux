// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Tauri - Main entry point
//
// Desktop file transfer application using Tauri + React.

#![cfg_attr(
    all(not(debug_assertions), target_os = "linux"),
    windows_subsystem = "windows"
)]

mod commands;
mod engine_bridge;
mod state;

use gosh_lan_transfer::EngineEvent;
use state::AppState;
use std::sync::Arc;
use std::thread;
use tauri::Emitter;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("gosh_transfer_tauri=info".parse().unwrap())
                .add_directive("gosh_lan_transfer=info".parse().unwrap()),
        )
        .init();

    // Create application state
    let app_state = Arc::new(AppState::new().expect("Failed to initialize application state"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let event_rx = app_state.bridge.event_receiver();

            // Spawn event listener thread
            thread::spawn(move || {
                while let Ok(event) = event_rx.recv_blocking() {
                    let event_json = engine_event_to_json(&event);
                    let _ = handle.emit("engine-event", event_json);
                }
            });

            // Auto-start server
            let tx = app_state.bridge.command_sender();
            let _ = tx.try_send(engine_bridge::EngineCommand::StartServer);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::initialize,
            commands::start_server,
            commands::stop_server,
            commands::resolve_address,
            commands::check_peer,
            commands::get_peer_info,
            commands::send_files,
            commands::send_directory,
            commands::accept_transfer,
            commands::reject_transfer,
            commands::accept_all,
            commands::reject_all,
            commands::cancel_transfer,
            commands::get_pending_transfers,
            commands::get_interfaces,
            commands::get_settings,
            commands::save_settings,
            commands::list_favorites,
            commands::add_favorite,
            commands::update_favorite,
            commands::delete_favorite,
            commands::touch_favorite,
            commands::list_history,
            commands::clear_history,
            commands::change_port,
            commands::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Convert engine event to JSON for frontend
fn engine_event_to_json(event: &EngineEvent) -> serde_json::Value {
    match event {
        EngineEvent::TransferRequest(transfer) => {
            serde_json::json!({
                "type": "TransferRequest",
                "transfer": transfer
            })
        }
        EngineEvent::TransferProgress(progress) => {
            serde_json::json!({
                "type": "TransferProgress",
                "progress": progress
            })
        }
        EngineEvent::TransferComplete { transfer_id } => {
            serde_json::json!({
                "type": "TransferComplete",
                "transferId": transfer_id
            })
        }
        EngineEvent::TransferFailed { transfer_id, error } => {
            serde_json::json!({
                "type": "TransferFailed",
                "transferId": transfer_id,
                "error": error
            })
        }
        EngineEvent::TransferRetry {
            transfer_id,
            attempt,
            max_attempts,
            error,
        } => {
            serde_json::json!({
                "type": "TransferRetry",
                "transferId": transfer_id,
                "attempt": attempt,
                "maxAttempts": max_attempts,
                "error": error
            })
        }
        EngineEvent::ServerStarted { port } => {
            serde_json::json!({
                "type": "ServerStarted",
                "port": port
            })
        }
        EngineEvent::ServerStopped => {
            serde_json::json!({
                "type": "ServerStopped"
            })
        }
        EngineEvent::PortChanged { old_port, new_port } => {
            serde_json::json!({
                "type": "PortChanged",
                "oldPort": old_port,
                "newPort": new_port
            })
        }
    }
}
