// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Tauri - Engine Bridge
//
// Bridges the async GoshTransferEngine with the Tauri frontend.

use async_channel::{Receiver, Sender};
use gosh_lan_transfer::{
    EngineConfig, EngineEvent, GoshTransferEngine, NetworkInterface, PendingTransfer, ResolveResult,
};
use gosh_transfer_core::TransferHistory;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Commands that can be sent to the engine
#[derive(Debug)]
pub enum EngineCommand {
    StartServer,
    StopServer,
    ResolveAddress {
        address: String,
        reply: Sender<ResolveResult>,
    },
    SendFiles {
        address: String,
        port: u16,
        paths: Vec<PathBuf>,
    },
    SendDirectory {
        address: String,
        port: u16,
        path: PathBuf,
    },
    AcceptTransfer {
        id: String,
    },
    RejectTransfer {
        id: String,
    },
    AcceptAllTransfers,
    RejectAllTransfers,
    CancelTransfer {
        id: String,
    },
    CheckPeer {
        address: String,
        port: u16,
        reply: Sender<bool>,
    },
    GetPeerInfo {
        address: String,
        port: u16,
        reply: Sender<Result<Value, String>>,
    },
    GetPendingTransfers {
        reply: Sender<Vec<PendingTransfer>>,
    },
    GetInterfaces {
        reply: Sender<Vec<NetworkInterface>>,
    },
    UpdateConfig {
        config: EngineConfig,
    },
    ChangePort {
        port: u16,
        rollback_on_failure: bool,
    },
}

/// Bridge between Tauri frontend and async engine
pub struct EngineBridge {
    command_tx: Sender<EngineCommand>,
    event_rx: Receiver<EngineEvent>,
    _runtime: Arc<Runtime>,
}

impl EngineBridge {
    pub fn new(config: EngineConfig, history: Option<Arc<TransferHistory>>) -> Self {
        let (command_tx, command_rx) = async_channel::bounded::<EngineCommand>(32);
        let (event_tx, event_rx) = async_channel::bounded::<EngineEvent>(64);

        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime"),
        );

        let rt = runtime.clone();
        runtime.spawn(async move {
            Self::run_engine(config, command_rx, event_tx, history).await;
        });

        Self {
            command_tx,
            event_rx,
            _runtime: rt,
        }
    }

    async fn run_engine(
        config: EngineConfig,
        command_rx: Receiver<EngineCommand>,
        event_tx: Sender<EngineEvent>,
        history: Option<Arc<TransferHistory>>,
    ) {
        let (engine, mut engine_events) = if let Some(history) = history {
            GoshTransferEngine::with_channel_events_and_history(config, history)
        } else {
            GoshTransferEngine::with_channel_events(config)
        };
        let engine = Arc::new(Mutex::new(engine));

        loop {
            tokio::select! {
                cmd = command_rx.recv() => {
                    match cmd {
                        Ok(EngineCommand::StartServer) => {
                            let mut eng = engine.lock().await;
                            if let Err(e) = eng.start_server().await {
                                tracing::error!("Failed to start server: {}", e);
                            }
                        }
                        Ok(EngineCommand::StopServer) => {
                            let mut eng = engine.lock().await;
                            let _ = eng.stop_server().await;
                        }
                        Ok(EngineCommand::ResolveAddress { address, reply }) => {
                            let result = GoshTransferEngine::resolve_address(&address);
                            let _ = reply.send(result).await;
                        }
                        Ok(EngineCommand::SendFiles { address, port, paths }) => {
                            let eng = engine.lock().await;
                            if let Err(e) = eng.send_files(&address, port, paths).await {
                                tracing::error!("Send failed: {}", e);
                            }
                        }
                        Ok(EngineCommand::SendDirectory { address, port, path }) => {
                            let eng = engine.lock().await;
                            if let Err(e) = eng.send_directory(&address, port, path).await {
                                tracing::error!("Send directory failed: {}", e);
                            }
                        }
                        Ok(EngineCommand::AcceptTransfer { id }) => {
                            let eng = engine.lock().await;
                            if let Err(e) = eng.accept_transfer(&id).await {
                                tracing::error!("Accept failed: {}", e);
                            }
                        }
                        Ok(EngineCommand::RejectTransfer { id }) => {
                            let eng = engine.lock().await;
                            if let Err(e) = eng.reject_transfer(&id).await {
                                tracing::error!("Reject failed: {}", e);
                            }
                        }
                        Ok(EngineCommand::AcceptAllTransfers) => {
                            let eng = engine.lock().await;
                            let results = eng.accept_all_transfers().await;
                            for (id, result) in results {
                                if let Err(e) = result {
                                    tracing::error!("Accept {} failed: {}", id, e);
                                }
                            }
                        }
                        Ok(EngineCommand::RejectAllTransfers) => {
                            let eng = engine.lock().await;
                            let results = eng.reject_all_transfers().await;
                            for (id, result) in results {
                                if let Err(e) = result {
                                    tracing::error!("Reject {} failed: {}", id, e);
                                }
                            }
                        }
                        Ok(EngineCommand::CancelTransfer { id }) => {
                            let eng = engine.lock().await;
                            if let Err(e) = eng.cancel_transfer(&id).await {
                                tracing::error!("Cancel failed: {}", e);
                            }
                        }
                        Ok(EngineCommand::CheckPeer { address, port, reply }) => {
                            let eng = engine.lock().await;
                            let reachable = eng.check_peer(&address, port).await.unwrap_or(false);
                            let _ = reply.send(reachable).await;
                        }
                        Ok(EngineCommand::GetPeerInfo { address, port, reply }) => {
                            let eng = engine.lock().await;
                            let result = eng.get_peer_info(&address, port).await.map_err(|e| e.to_string());
                            let _ = reply.send(result).await;
                        }
                        Ok(EngineCommand::GetPendingTransfers { reply }) => {
                            let eng = engine.lock().await;
                            let pending = eng.get_pending_transfers().await;
                            let _ = reply.send(pending).await;
                        }
                        Ok(EngineCommand::GetInterfaces { reply }) => {
                            let interfaces = GoshTransferEngine::get_network_interfaces();
                            let _ = reply.send(interfaces).await;
                        }
                        Ok(EngineCommand::UpdateConfig { config }) => {
                            let mut eng = engine.lock().await;
                            eng.update_config(config).await;
                        }
                        Ok(EngineCommand::ChangePort { port, rollback_on_failure }) => {
                            let mut eng = engine.lock().await;
                            if rollback_on_failure {
                                let _ = eng.change_port(port).await;
                            } else {
                                let _ = eng.change_port_with_options(port, false).await;
                            }
                        }
                        Err(_) => break,
                    }
                }
                event = engine_events.recv() => {
                    if let Ok(event) = event {
                        if event_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn command_sender(&self) -> Sender<EngineCommand> {
        self.command_tx.clone()
    }

    pub fn event_receiver(&self) -> Receiver<EngineEvent> {
        self.event_rx.clone()
    }
}
