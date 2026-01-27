// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Tauri - Command Handlers

use crate::engine_bridge::EngineCommand;
use crate::state::AppState;
use gosh_transfer_core::{
    AppSettings, Favorite, FavoritesPersistence, NetworkInterface, PendingTransfer, TransferRecord,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

type CommandResult<T> = Result<T, String>;

/// Initialize the engine and start the server
#[tauri::command]
pub async fn initialize(state: State<'_, Arc<AppState>>) -> CommandResult<bool> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::StartServer)
        .await
        .map_err(|e| e.to_string())?;
    Ok(true)
}

/// Start the HTTP server
#[tauri::command]
pub async fn start_server(state: State<'_, Arc<AppState>>) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::StartServer)
        .await
        .map_err(|e| e.to_string())
}

/// Stop the HTTP server
#[tauri::command]
pub async fn stop_server(state: State<'_, Arc<AppState>>) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::StopServer)
        .await
        .map_err(|e| e.to_string())
}

/// Resolve a hostname or IP address
#[tauri::command]
pub async fn resolve_address(
    state: State<'_, Arc<AppState>>,
    address: String,
) -> CommandResult<Value> {
    let tx = state.bridge.command_sender();
    let (reply_tx, reply_rx) = async_channel::bounded(1);

    tx.send(EngineCommand::ResolveAddress {
        address,
        reply: reply_tx,
    })
    .await
    .map_err(|e| e.to_string())?;

    let result = reply_rx.recv().await.map_err(|e| e.to_string())?;
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Check if a peer is reachable
#[tauri::command]
pub async fn check_peer(
    state: State<'_, Arc<AppState>>,
    address: String,
    port: u16,
) -> CommandResult<bool> {
    let tx = state.bridge.command_sender();
    let (reply_tx, reply_rx) = async_channel::bounded(1);

    tx.send(EngineCommand::CheckPeer {
        address,
        port,
        reply: reply_tx,
    })
    .await
    .map_err(|e| e.to_string())?;

    reply_rx.recv().await.map_err(|e| e.to_string())
}

/// Get peer info
#[tauri::command]
pub async fn get_peer_info(
    state: State<'_, Arc<AppState>>,
    address: String,
    port: u16,
) -> CommandResult<Value> {
    let tx = state.bridge.command_sender();
    let (reply_tx, reply_rx) = async_channel::bounded(1);

    tx.send(EngineCommand::GetPeerInfo {
        address,
        port,
        reply: reply_tx,
    })
    .await
    .map_err(|e| e.to_string())?;

    let result = reply_rx.recv().await.map_err(|e| e.to_string())?;
    result.map_err(|e| e.to_string())
}

/// Send files to a peer
#[tauri::command]
pub async fn send_files(
    state: State<'_, Arc<AppState>>,
    address: String,
    port: u16,
    paths: Vec<String>,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();

    tx.send(EngineCommand::SendFiles {
        address,
        port,
        paths,
    })
    .await
    .map_err(|e| e.to_string())
}

/// Send a directory to a peer
#[tauri::command]
pub async fn send_directory(
    state: State<'_, Arc<AppState>>,
    address: String,
    port: u16,
    path: String,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();

    tx.send(EngineCommand::SendDirectory {
        address,
        port,
        path: PathBuf::from(path),
    })
    .await
    .map_err(|e| e.to_string())
}

/// Accept a transfer request
#[tauri::command]
pub async fn accept_transfer(
    state: State<'_, Arc<AppState>>,
    transfer_id: String,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::AcceptTransfer { id: transfer_id })
        .await
        .map_err(|e| e.to_string())
}

/// Reject a transfer request
#[tauri::command]
pub async fn reject_transfer(
    state: State<'_, Arc<AppState>>,
    transfer_id: String,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::RejectTransfer { id: transfer_id })
        .await
        .map_err(|e| e.to_string())
}

/// Accept all pending transfers
#[tauri::command]
pub async fn accept_all(state: State<'_, Arc<AppState>>) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::AcceptAllTransfers)
        .await
        .map_err(|e| e.to_string())
}

/// Reject all pending transfers
#[tauri::command]
pub async fn reject_all(state: State<'_, Arc<AppState>>) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::RejectAllTransfers)
        .await
        .map_err(|e| e.to_string())
}

/// Cancel an active transfer
#[tauri::command]
pub async fn cancel_transfer(
    state: State<'_, Arc<AppState>>,
    transfer_id: String,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::CancelTransfer { id: transfer_id })
        .await
        .map_err(|e| e.to_string())
}

/// Get pending transfer requests
#[tauri::command]
pub async fn get_pending_transfers(
    state: State<'_, Arc<AppState>>,
) -> CommandResult<Vec<PendingTransfer>> {
    let tx = state.bridge.command_sender();
    let (reply_tx, reply_rx) = async_channel::bounded(1);

    tx.send(EngineCommand::GetPendingTransfers { reply: reply_tx })
        .await
        .map_err(|e| e.to_string())?;

    reply_rx.recv().await.map_err(|e| e.to_string())
}

/// Get network interfaces
#[tauri::command]
pub async fn get_interfaces(
    state: State<'_, Arc<AppState>>,
) -> CommandResult<Vec<NetworkInterface>> {
    let tx = state.bridge.command_sender();
    let (reply_tx, reply_rx) = async_channel::bounded(1);

    tx.send(EngineCommand::GetInterfaces { reply: reply_tx })
        .await
        .map_err(|e| e.to_string())?;

    reply_rx.recv().await.map_err(|e| e.to_string())
}

/// Get current settings
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.settings.get()
}

/// Save settings
#[tauri::command]
pub fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> CommandResult<bool> {
    // Update settings store
    state
        .settings
        .update(settings.clone())
        .map_err(|e| e.to_string())?;

    // Update engine config
    let config = settings.to_engine_config();
    let tx = state.bridge.command_sender();
    tx.try_send(EngineCommand::UpdateConfig { config })
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// List all favorites
#[tauri::command]
pub fn list_favorites(state: State<'_, Arc<AppState>>) -> CommandResult<Vec<Favorite>> {
    state.favorites.list().map_err(|e| e.to_string())
}

/// Add a new favorite
#[tauri::command]
pub fn add_favorite(
    state: State<'_, Arc<AppState>>,
    name: String,
    address: String,
) -> CommandResult<Favorite> {
    state
        .favorites
        .add(name, address)
        .map_err(|e| e.to_string())
}

/// Update an existing favorite
#[tauri::command]
pub fn update_favorite(
    state: State<'_, Arc<AppState>>,
    id: String,
    name: Option<String>,
    address: Option<String>,
) -> CommandResult<Favorite> {
    state
        .favorites
        .update(&id, name, address)
        .map_err(|e| e.to_string())
}

/// Delete a favorite
#[tauri::command]
pub fn delete_favorite(state: State<'_, Arc<AppState>>, id: String) -> CommandResult<bool> {
    state.favorites.delete(&id).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Touch a favorite to update last_used
#[tauri::command]
pub fn touch_favorite(state: State<'_, Arc<AppState>>, id: String) -> CommandResult<bool> {
    state.favorites.touch(&id).map_err(|e| e.to_string())?;
    Ok(true)
}

/// List transfer history
#[tauri::command]
pub fn list_history(state: State<'_, Arc<AppState>>) -> Vec<TransferRecord> {
    state.history.list()
}

/// Clear transfer history
#[tauri::command]
pub fn clear_history(state: State<'_, Arc<AppState>>) -> CommandResult<bool> {
    state.history.clear().map_err(|e| e.to_string())?;
    Ok(true)
}

/// Change the server port
#[tauri::command]
pub async fn change_port(
    state: State<'_, Arc<AppState>>,
    port: u16,
    rollback_on_failure: bool,
) -> CommandResult<()> {
    let tx = state.bridge.command_sender();
    tx.send(EngineCommand::ChangePort {
        port,
        rollback_on_failure,
    })
    .await
    .map_err(|e| e.to_string())
}

/// Get application version
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
