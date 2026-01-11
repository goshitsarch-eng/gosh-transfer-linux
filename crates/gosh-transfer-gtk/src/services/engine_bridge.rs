// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer GTK - Engine Bridge
//
// Bridges the async GoshTransferEngine with GTK's main loop.

use async_channel::{Receiver, Sender};
use gosh_lan_transfer::{
    EngineConfig, EngineEvent, GoshTransferEngine, NetworkInterface, PendingTransfer,
    ResolveResult,
};
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
    AcceptTransfer {
        id: String,
    },
    RejectTransfer {
        id: String,
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
}

/// Bridge between GTK UI and async engine
pub struct EngineBridge {
    command_tx: Sender<EngineCommand>,
    event_rx: Receiver<EngineEvent>,
    _runtime: Arc<Runtime>,
}

impl EngineBridge {
    pub fn new(config: EngineConfig) -> Self {
        let (command_tx, command_rx) = async_channel::bounded::<EngineCommand>(32);
        let (event_tx, event_rx) = async_channel::bounded::<EngineEvent>(64);

        // Create tokio runtime
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime"),
        );

        // Spawn the engine management task
        let rt = runtime.clone();
        runtime.spawn(async move {
            Self::run_engine(config, command_rx, event_tx).await;
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
    ) {
        let (engine, mut engine_events) = GoshTransferEngine::with_channel_events(config);
        let engine = Arc::new(Mutex::new(engine));

        loop {
            tokio::select! {
                // Handle commands from UI
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
                        Err(_) => break, // Channel closed
                    }
                }
                // Forward engine events to UI
                event = engine_events.recv() => {
                    if let Ok(event) = event {
                        if event_tx.send(event).await.is_err() {
                            break; // Channel closed
                        }
                    }
                }
            }
        }
    }

    /// Start the server
    pub fn start_server(&self) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx.send(EngineCommand::StartServer).await;
        });
    }

    /// Stop the server
    pub fn stop_server(&self) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx.send(EngineCommand::StopServer).await;
        });
    }

    /// Resolve hostname - returns result via callback
    pub fn resolve_address<F>(&self, address: String, callback: F)
    where
        F: FnOnce(ResolveResult) + 'static,
    {
        let (reply_tx, reply_rx) = async_channel::bounded(1);
        let tx = self.command_tx.clone();

        glib::spawn_future_local(async move {
            let _ = tx
                .send(EngineCommand::ResolveAddress {
                    address,
                    reply: reply_tx,
                })
                .await;

            if let Ok(result) = reply_rx.recv().await {
                callback(result);
            }
        });
    }

    /// Send files to peer
    pub fn send_files(&self, address: String, port: u16, paths: Vec<PathBuf>) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx
                .send(EngineCommand::SendFiles {
                    address,
                    port,
                    paths,
                })
                .await;
        });
    }

    /// Accept a pending transfer
    pub fn accept_transfer(&self, id: String) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx.send(EngineCommand::AcceptTransfer { id }).await;
        });
    }

    /// Reject a pending transfer
    pub fn reject_transfer(&self, id: String) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx.send(EngineCommand::RejectTransfer { id }).await;
        });
    }

    /// Get pending transfers
    pub fn get_pending_transfers<F>(&self, callback: F)
    where
        F: FnOnce(Vec<PendingTransfer>) + 'static,
    {
        let (reply_tx, reply_rx) = async_channel::bounded(1);
        let tx = self.command_tx.clone();

        glib::spawn_future_local(async move {
            let _ = tx
                .send(EngineCommand::GetPendingTransfers { reply: reply_tx })
                .await;

            if let Ok(pending) = reply_rx.recv().await {
                callback(pending);
            }
        });
    }

    /// Get network interfaces
    pub fn get_interfaces<F>(&self, callback: F)
    where
        F: FnOnce(Vec<NetworkInterface>) + 'static,
    {
        let (reply_tx, reply_rx) = async_channel::bounded(1);
        let tx = self.command_tx.clone();

        glib::spawn_future_local(async move {
            let _ = tx
                .send(EngineCommand::GetInterfaces { reply: reply_tx })
                .await;

            if let Ok(interfaces) = reply_rx.recv().await {
                callback(interfaces);
            }
        });
    }

    /// Update engine configuration
    pub fn update_config(&self, config: EngineConfig) {
        let tx = self.command_tx.clone();
        glib::spawn_future_local(async move {
            let _ = tx.send(EngineCommand::UpdateConfig { config }).await;
        });
    }

    /// Get event receiver for subscribing to engine events
    pub fn event_receiver(&self) -> Receiver<EngineEvent> {
        self.event_rx.clone()
    }
}
