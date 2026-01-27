// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Tauri - Application State

use crate::engine_bridge::EngineBridge;
use gosh_transfer_core::{FileFavoritesStore, SettingsStore, TransferHistory};
use std::sync::Arc;

/// Global application state managed by Tauri
pub struct AppState {
    pub bridge: EngineBridge,
    pub settings: SettingsStore,
    pub favorites: FileFavoritesStore,
    pub history: Arc<TransferHistory>,
}

impl AppState {
    /// Create new application state with all stores initialized
    pub fn new() -> Result<Self, gosh_transfer_core::AppError> {
        let settings = SettingsStore::new()?;
        let favorites = FileFavoritesStore::new()?;
        let history = Arc::new(TransferHistory::new()?);

        let config = settings.get().to_engine_config();
        let bridge = EngineBridge::new(config, Some(history.clone()));

        Ok(Self {
            bridge,
            settings,
            favorites,
            history,
        })
    }
}
