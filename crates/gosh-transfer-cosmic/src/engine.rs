// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Engine Bridge

use gosh_lan_transfer::{
    EngineConfig, GoshTransferEngine, NetworkInterface, PendingTransfer, ResolveResult,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants will be used when full event handling is implemented
pub enum EngineMessage {
    // Commands
    StartServer,
    StopServer,
    ServerStarted,

    // Events from engine
    TransferRequest(PendingTransfer),
    TransferProgress {
        transfer_id: String,
        bytes: u64,
        total: u64,
    },
    TransferComplete {
        transfer_id: String,
    },
    TransferFailed {
        transfer_id: String,
        error: String,
    },
}

pub struct EngineBridge {
    engine: Arc<Mutex<GoshTransferEngine>>,
    config: Arc<Mutex<EngineConfig>>,
}

impl EngineBridge {
    pub fn new(config: EngineConfig) -> Self {
        let (engine, _event_rx) = GoshTransferEngine::with_channel_events(config.clone());

        Self {
            engine: Arc::new(Mutex::new(engine)),
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub async fn start_server(&self) {
        let mut engine = self.engine.lock().await;
        if let Err(e) = engine.start_server().await {
            tracing::error!("Failed to start server: {}", e);
        }
    }

    #[allow(dead_code)] // Will be used when full engine control is implemented
    pub async fn stop_server(&self) {
        let mut engine = self.engine.lock().await;
        let _ = engine.stop_server().await;
    }

    pub async fn resolve_address(&self, address: &str) -> ResolveResult {
        GoshTransferEngine::resolve_address(address)
    }

    pub async fn send_files(
        &self,
        address: &str,
        port: u16,
        paths: Vec<PathBuf>,
    ) -> Result<(), gosh_lan_transfer::EngineError> {
        let engine = self.engine.lock().await;
        engine.send_files(address, port, paths).await
    }

    pub async fn accept_transfer(&self, id: &str) -> Result<(), gosh_lan_transfer::EngineError> {
        let engine = self.engine.lock().await;
        engine.accept_transfer(id).await.map(|_| ())
    }

    pub async fn reject_transfer(&self, id: &str) -> Result<(), gosh_lan_transfer::EngineError> {
        let engine = self.engine.lock().await;
        engine.reject_transfer(id).await
    }

    pub async fn get_pending_transfers(&self) -> Vec<PendingTransfer> {
        let engine = self.engine.lock().await;
        engine.get_pending_transfers().await
    }

    #[allow(dead_code)] // Will be used for network interface display
    pub async fn get_interfaces(&self) -> Vec<NetworkInterface> {
        GoshTransferEngine::get_network_interfaces()
    }

    pub fn update_config(&self, config: EngineConfig) {
        let engine = self.engine.clone();
        let config_store = self.config.clone();

        tokio::spawn(async move {
            *config_store.lock().await = config.clone();
            engine.lock().await.update_config(config).await;
        });
    }
}
