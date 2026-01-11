// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Engine Bridge

use gosh_lan_transfer::{EngineConfig, EngineEvent, GoshTransferEngine};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::{broadcast, Mutex};

/// Bridge between Qt UI and async engine
pub struct EngineBridge {
    engine: Arc<Mutex<GoshTransferEngine>>,
    event_rx: Arc<Mutex<Option<broadcast::Receiver<EngineEvent>>>>,
    runtime: Runtime,
}

impl EngineBridge {
    pub fn new(config: EngineConfig) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let (engine, event_rx) = GoshTransferEngine::with_channel_events(config);

        Self {
            engine: Arc::new(Mutex::new(engine)),
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            runtime,
        }
    }

    /// Get a clone of the engine for async operations
    pub fn engine(&self) -> Arc<Mutex<GoshTransferEngine>> {
        self.engine.clone()
    }

    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&self) -> Option<broadcast::Receiver<EngineEvent>> {
        self.event_rx.blocking_lock().take()
    }

    /// Get the tokio runtime handle
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
}
