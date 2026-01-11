// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Core - Type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings (GUI-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Port for the HTTP server (default: 53317)
    pub port: u16,
    /// Device name shown to peers
    pub device_name: String,
    /// Default download directory
    pub download_dir: PathBuf,
    /// Auto-accept from trusted hosts
    pub trusted_hosts: Vec<String>,
    /// Receive-only mode (disable sending)
    pub receive_only: bool,
    /// Show system notifications
    pub notifications_enabled: bool,
    /// Theme preference: "dark", "light", or "system"
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "system".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = directories::UserDirs::new()
            .and_then(|d| d.download_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            port: 53317,
            device_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "Gosh Device".to_string()),
            download_dir,
            trusted_hosts: Vec::new(),
            receive_only: false,
            notifications_enabled: true,
            theme: default_theme(),
        }
    }
}

impl AppSettings {
    /// Convert to engine configuration
    pub fn to_engine_config(&self) -> gosh_lan_transfer::EngineConfig {
        gosh_lan_transfer::EngineConfig::builder()
            .port(self.port)
            .device_name(&self.device_name)
            .download_dir(&self.download_dir)
            .trusted_hosts(self.trusted_hosts.clone())
            .receive_only(self.receive_only)
            .build()
    }
}

/// Error types for the application
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Transfer rejected by peer")]
    TransferRejected,

    #[error("File I/O error: {0}")]
    FileIo(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Server not running")]
    ServerNotRunning,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Engine error: {0}")]
    Engine(String),
}

impl From<gosh_lan_transfer::EngineError> for AppError {
    fn from(err: gosh_lan_transfer::EngineError) -> Self {
        AppError::Engine(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::FileIo(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.port, 53317);
        assert_eq!(settings.theme, "system");
        assert!(!settings.receive_only);
    }

    #[test]
    fn test_engine_config_conversion() {
        let settings = AppSettings::default();
        let config = settings.to_engine_config();
        assert_eq!(config.port(), 53317);
    }
}
