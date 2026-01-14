// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Core - Shared logic for all frontends
//
// This crate provides:
// - AppSettings and AppError types
// - SettingsStore for persistent settings
// - FileFavoritesStore for persistent favorites
// - TransferHistory for tracking past transfers
//
// Frontend-specific code lives in separate crates.

pub mod favorites;
pub mod history;
pub mod settings;
pub mod types;

// Re-export commonly used items
pub use favorites::FileFavoritesStore;
pub use history::TransferHistory;
pub use settings::SettingsStore;
pub use types::{AppError, AppSettings, InterfaceCategory, InterfaceFilters};

// Re-export engine types for convenience
pub use gosh_lan_transfer::{
    EngineConfig, EngineError, EngineEvent, EngineResult, Favorite, FavoritesPersistence,
    GoshTransferEngine, NetworkInterface, PeerInfo, PendingTransfer, ResolveResult,
    TransferApprovalStatus, TransferDecision, TransferDirection, TransferFile, TransferProgress,
    TransferRecord, TransferRequest, TransferResponse, TransferStatus,
};
