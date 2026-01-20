// SPDX-License-Identifier: AGPL-3.0
// Qt bridge between Rust engine and Qt Widgets UI.

use crate::engine_bridge::{EngineBridge, EngineCommand};
use cxx_qt::{CxxQtThread, Threading};
use cxx_qt_lib::QString;
use gosh_lan_transfer::FavoritesPersistence;
use gosh_transfer_core::{FileFavoritesStore, SettingsStore, TransferHistory};
use once_cell::sync::OnceCell;
use serde_json::json;
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;

struct QtEngineState {
    bridge: EngineBridge,
    settings: SettingsStore,
    favorites: FileFavoritesStore,
    history: Arc<TransferHistory>,
}

static ENGINE_STATE: OnceCell<QtEngineState> = OnceCell::new();

fn to_qstring(value: &str) -> QString {
    QString::from(value)
}

fn qstring_to_string(value: &QString) -> String {
    value.to_string()
}

fn json_to_qstring(value: &Value) -> QString {
    let json = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    to_qstring(&json)
}

fn engine_event_to_json(event: &gosh_lan_transfer::EngineEvent) -> Value {
    match event {
        gosh_lan_transfer::EngineEvent::TransferRequest(transfer) => {
            json!({ "TransferRequest": transfer })
        }
        gosh_lan_transfer::EngineEvent::TransferProgress(progress) => {
            json!({ "TransferProgress": progress })
        }
        gosh_lan_transfer::EngineEvent::TransferComplete { transfer_id } => {
            json!({ "TransferComplete": { "transfer_id": transfer_id } })
        }
        gosh_lan_transfer::EngineEvent::TransferFailed { transfer_id, error } => {
            json!({ "TransferFailed": { "transfer_id": transfer_id, "error": error } })
        }
        gosh_lan_transfer::EngineEvent::TransferRetry {
            transfer_id,
            attempt,
            max_attempts,
            error,
        } => json!({
            "TransferRetry": {
                "transfer_id": transfer_id,
                "attempt": attempt,
                "max_attempts": max_attempts,
                "error": error
            }
        }),
        gosh_lan_transfer::EngineEvent::ServerStarted { port } => {
            json!({ "ServerStarted": { "port": port } })
        }
        gosh_lan_transfer::EngineEvent::ServerStopped => json!({ "ServerStopped": {} }),
        gosh_lan_transfer::EngineEvent::PortChanged { old_port, new_port } => {
            json!({ "PortChanged": { "old_port": old_port, "new_port": new_port } })
        }
    }
}

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        type EngineBridgeQt = super::EngineBridgeQtRust;

        #[qinvokable]
        fn initialize(self: Pin<&mut EngineBridgeQt>) -> bool;
        #[qinvokable]
        fn start_server(self: Pin<&mut EngineBridgeQt>);
        #[qinvokable]
        fn stop_server(self: Pin<&mut EngineBridgeQt>);
        #[qinvokable]
        fn resolve_address(self: Pin<&mut EngineBridgeQt>, address: &QString) -> QString;
        #[qinvokable]
        fn check_peer(self: Pin<&mut EngineBridgeQt>, address: &QString, port: i32) -> bool;
        #[qinvokable]
        fn get_peer_info(self: Pin<&mut EngineBridgeQt>, address: &QString, port: i32) -> QString;
        #[qinvokable]
        fn send_files(self: Pin<&mut EngineBridgeQt>, address: &QString, port: i32, paths_json: &QString);
        #[qinvokable]
        fn send_directory(self: Pin<&mut EngineBridgeQt>, address: &QString, port: i32, path: &QString);
        #[qinvokable]
        fn accept_transfer(self: Pin<&mut EngineBridgeQt>, transfer_id: &QString);
        #[qinvokable]
        fn reject_transfer(self: Pin<&mut EngineBridgeQt>, transfer_id: &QString);
        #[qinvokable]
        fn accept_all(self: Pin<&mut EngineBridgeQt>);
        #[qinvokable]
        fn reject_all(self: Pin<&mut EngineBridgeQt>);
        #[qinvokable]
        fn cancel_transfer(self: Pin<&mut EngineBridgeQt>, transfer_id: &QString);
        #[qinvokable]
        fn get_pending_transfers(self: Pin<&mut EngineBridgeQt>) -> QString;
        #[qinvokable]
        fn get_interfaces(self: Pin<&mut EngineBridgeQt>) -> QString;
        #[qinvokable]
        fn get_settings(self: Pin<&mut EngineBridgeQt>) -> QString;
        #[qinvokable]
        fn save_settings(self: Pin<&mut EngineBridgeQt>, settings_json: &QString) -> bool;
        #[qinvokable]
        fn list_favorites(self: Pin<&mut EngineBridgeQt>) -> QString;
        #[qinvokable]
        fn add_favorite(self: Pin<&mut EngineBridgeQt>, name: &QString, address: &QString) -> QString;
        #[qinvokable]
        fn update_favorite(self: Pin<&mut EngineBridgeQt>, id: &QString, name: &QString, address: &QString) -> QString;
        #[qinvokable]
        fn delete_favorite(self: Pin<&mut EngineBridgeQt>, id: &QString) -> bool;
        #[qinvokable]
        fn touch_favorite(self: Pin<&mut EngineBridgeQt>, id: &QString) -> bool;
        #[qinvokable]
        fn list_history(self: Pin<&mut EngineBridgeQt>) -> QString;
        #[qinvokable]
        fn clear_history(self: Pin<&mut EngineBridgeQt>) -> bool;
        #[qinvokable]
        fn change_port(self: Pin<&mut EngineBridgeQt>, port: i32, rollback_on_failure: bool);
        #[qinvokable]
        fn get_version(self: Pin<&mut EngineBridgeQt>) -> QString;

        #[qsignal]
        fn engine_event(self: Pin<&mut EngineBridgeQt>, event_json: &QString);
        #[qsignal]
        fn engine_error(self: Pin<&mut EngineBridgeQt>, message: &QString);
    }

    impl cxx_qt::Threading for EngineBridgeQt {}
}

fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&QtEngineState) -> R,
{
    ENGINE_STATE.get().map(f)
}

#[derive(Default)]
pub struct EngineBridgeQtRust;

impl ffi::EngineBridgeQt {
    fn initialize(self: Pin<&mut Self>) -> bool {
        if ENGINE_STATE.get().is_some() {
            return true;
        }

        let settings = match SettingsStore::new() {
            Ok(store) => store,
            Err(err) => {
                let msg = to_qstring(&format!("Failed to load settings: {}", err));
                self.engine_error(&msg);
                return false;
            }
        };

        let favorites = match FileFavoritesStore::new() {
            Ok(store) => store,
            Err(err) => {
                let msg = to_qstring(&format!("Failed to load favorites: {}", err));
                self.engine_error(&msg);
                return false;
            }
        };

        let history = match TransferHistory::new() {
            Ok(store) => Arc::new(store),
            Err(err) => {
                let msg = to_qstring(&format!("Failed to load history: {}", err));
                self.engine_error(&msg);
                return false;
            }
        };

        let config = settings.get().to_engine_config();
        let bridge = EngineBridge::new(config, Some(history.clone()));

        if ENGINE_STATE
            .set(QtEngineState {
                bridge,
                settings,
                favorites,
                history,
            })
            .is_err()
        {
            return false;
        }

        let qt_thread: CxxQtThread<ffi::EngineBridgeQt> = self.qt_thread();
        let event_rx = with_state(|state| state.bridge.event_receiver());
        if let Some(event_rx) = event_rx {
            thread::spawn(move || {
                while let Ok(event) = event_rx.recv_blocking() {
                    let json = engine_event_to_json(&event);
                    let event_json = json_to_qstring(&json);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut ffi::EngineBridgeQt>| {
                        obj.as_mut().engine_event(&event_json);
                    });
                }
            });
        }

        true
    }

    fn start_server(self: Pin<&mut Self>) {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::StartServer);
        }
    }

    fn stop_server(self: Pin<&mut Self>) {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::StopServer);
        }
    }

    fn resolve_address(self: Pin<&mut Self>, address: &QString) -> QString {
        let address = qstring_to_string(address);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let (reply_tx, reply_rx) = async_channel::bounded(1);
            let _ = tx.try_send(EngineCommand::ResolveAddress {
                address,
                reply: reply_tx,
            });
            if let Ok(result) = reply_rx.recv_blocking() {
                if result.success {
                    if let Some(ip) = result.ips.first() {
                        let _ = with_state(|state| state.favorites.update_resolved_ip(&result.hostname, ip));
                    }
                }
                let value = serde_json::to_value(&result).unwrap_or(Value::Null);
                return json_to_qstring(&value);
            }
        }
        to_qstring("{}")
    }

    fn check_peer(self: Pin<&mut Self>, address: &QString, port: i32) -> bool {
        let address = qstring_to_string(address);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let (reply_tx, reply_rx) = async_channel::bounded(1);
            let _ = tx.try_send(EngineCommand::CheckPeer {
                address,
                port: port as u16,
                reply: reply_tx,
            });
            if let Ok(result) = reply_rx.recv_blocking() {
                return result;
            }
        }
        false
    }

    fn get_peer_info(self: Pin<&mut Self>, address: &QString, port: i32) -> QString {
        let address = qstring_to_string(address);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let (reply_tx, reply_rx) = async_channel::bounded(1);
            let _ = tx.try_send(EngineCommand::GetPeerInfo {
                address,
                port: port as u16,
                reply: reply_tx,
            });
            if let Ok(result) = reply_rx.recv_blocking() {
                let value = match result {
                    Ok(info) => info,
                    Err(err) => Value::String(err),
                };
                return json_to_qstring(&value);
            }
        }
        to_qstring("{}")
    }

    fn send_files(self: Pin<&mut Self>, address: &QString, port: i32, paths_json: &QString) {
        let address = qstring_to_string(address);
        let paths_json = qstring_to_string(paths_json);
        let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
        let paths: Vec<std::path::PathBuf> = paths.into_iter().map(std::path::PathBuf::from).collect();
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::SendFiles {
                address,
                port: port as u16,
                paths,
            });
        }
    }

    fn send_directory(self: Pin<&mut Self>, address: &QString, port: i32, path: &QString) {
        let address = qstring_to_string(address);
        let path = qstring_to_string(path);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::SendDirectory {
                address,
                port: port as u16,
                path: std::path::PathBuf::from(path),
            });
        }
    }

    fn accept_transfer(self: Pin<&mut Self>, transfer_id: &QString) {
        let transfer_id = qstring_to_string(transfer_id);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::AcceptTransfer { id: transfer_id });
        }
    }

    fn reject_transfer(self: Pin<&mut Self>, transfer_id: &QString) {
        let transfer_id = qstring_to_string(transfer_id);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::RejectTransfer { id: transfer_id });
        }
    }

    fn accept_all(self: Pin<&mut Self>) {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::AcceptAllTransfers);
        }
    }

    fn reject_all(self: Pin<&mut Self>) {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::RejectAllTransfers);
        }
    }

    fn cancel_transfer(self: Pin<&mut Self>, transfer_id: &QString) {
        let transfer_id = qstring_to_string(transfer_id);
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::CancelTransfer { id: transfer_id });
        }
    }

    fn get_pending_transfers(self: Pin<&mut Self>) -> QString {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let (reply_tx, reply_rx) = async_channel::bounded(1);
            let _ = tx.try_send(EngineCommand::GetPendingTransfers { reply: reply_tx });
            if let Ok(result) = reply_rx.recv_blocking() {
                let value = serde_json::to_value(result).unwrap_or(Value::Null);
                return json_to_qstring(&value);
            }
        }
        to_qstring("[]")
    }

    fn get_interfaces(self: Pin<&mut Self>) -> QString {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let (reply_tx, reply_rx) = async_channel::bounded(1);
            let _ = tx.try_send(EngineCommand::GetInterfaces { reply: reply_tx });
            if let Ok(result) = reply_rx.recv_blocking() {
                let value = serde_json::to_value(result).unwrap_or(Value::Null);
                return json_to_qstring(&value);
            }
        }
        to_qstring("[]")
    }

    fn get_settings(self: Pin<&mut Self>) -> QString {
        if let Some(settings) = with_state(|state| state.settings.get()) {
            let value = serde_json::to_value(settings).unwrap_or(Value::Null);
            return json_to_qstring(&value);
        }
        to_qstring("{}")
    }

    fn save_settings(self: Pin<&mut Self>, settings_json: &QString) -> bool {
        let settings_json = qstring_to_string(settings_json);
        let parsed = serde_json::from_str(&settings_json);
        let Ok(settings) = parsed else { return false; };

        let result = with_state(|state| state.settings.update(settings));
        match result {
            Some(Ok(())) => {}
            _ => return false,
        }

        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            if let Some(config) = with_state(|state| state.settings.get().to_engine_config()) {
                let _ = tx.try_send(EngineCommand::UpdateConfig { config });
            }
        }

        true
    }

    fn list_favorites(self: Pin<&mut Self>) -> QString {
        if let Some(result) = with_state(|state| state.favorites.list()) {
            if let Ok(favorites) = result {
                let value = serde_json::to_value(favorites).unwrap_or(Value::Null);
                return json_to_qstring(&value);
            }
        }
        to_qstring("[]")
    }

    fn add_favorite(self: Pin<&mut Self>, name: &QString, address: &QString) -> QString {
        let name = qstring_to_string(name);
        let address = qstring_to_string(address);
        let result = with_state(|state| state.favorites.add(name, address));
        if let Some(Ok(favorite)) = result {
            let value = serde_json::to_value(favorite).unwrap_or(Value::Null);
            return json_to_qstring(&value);
        }
        to_qstring("{}")
    }

    fn update_favorite(self: Pin<&mut Self>, id: &QString, name: &QString, address: &QString) -> QString {
        let id = qstring_to_string(id);
        let name = qstring_to_string(name);
        let address = qstring_to_string(address);
        let result = with_state(|state| state.favorites.update(&id, Some(name), Some(address)));
        if let Some(Ok(favorite)) = result {
            let value = serde_json::to_value(favorite).unwrap_or(Value::Null);
            return json_to_qstring(&value);
        }
        to_qstring("{}")
    }

    fn delete_favorite(self: Pin<&mut Self>, id: &QString) -> bool {
        let id = qstring_to_string(id);
        let result = with_state(|state| state.favorites.delete(&id));
        result.map(|r| r.is_ok()).unwrap_or(false)
    }

    fn touch_favorite(self: Pin<&mut Self>, id: &QString) -> bool {
        let id = qstring_to_string(id);
        let result = with_state(|state| state.favorites.touch(&id));
        result.map(|r| r.is_ok()).unwrap_or(false)
    }

    fn list_history(self: Pin<&mut Self>) -> QString {
        if let Some(records) = with_state(|state| state.history.list()) {
            let value = serde_json::to_value(records).unwrap_or(Value::Null);
            return json_to_qstring(&value);
        }
        to_qstring("[]")
    }

    fn clear_history(self: Pin<&mut Self>) -> bool {
        let result = with_state(|state| state.history.clear());
        result.map(|r| r.is_ok()).unwrap_or(false)
    }

    fn change_port(self: Pin<&mut Self>, port: i32, rollback_on_failure: bool) {
        if let Some(tx) = with_state(|state| state.bridge.command_sender()) {
            let _ = tx.try_send(EngineCommand::ChangePort {
                port: port as u16,
                rollback_on_failure,
            });
        }
    }

    fn get_version(self: Pin<&mut Self>) -> QString {
        to_qstring(env!("CARGO_PKG_VERSION"))
    }
}
