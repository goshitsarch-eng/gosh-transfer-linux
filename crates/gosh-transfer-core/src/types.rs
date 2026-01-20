// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Core - Type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Network interface category for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceCategory {
    WiFi,
    Ethernet,
    Vpn,
    Docker,
    Other,
}

impl InterfaceCategory {
    /// Determine category from interface name
    pub fn from_name(name: &str) -> Self {
        if name.starts_with("tailscale") || name.starts_with("tun") {
            Self::Vpn
        } else if name.starts_with("wl") {
            Self::WiFi
        } else if name.starts_with("en") || name.starts_with("eth") {
            Self::Ethernet
        } else if name.starts_with("docker") || name.starts_with("br-") {
            Self::Docker
        } else {
            Self::Other
        }
    }

    /// Get icon name for this category
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Vpn => "network-vpn-symbolic",
            Self::WiFi => "network-wireless-symbolic",
            Self::Ethernet => "network-wired-symbolic",
            Self::Docker => "network-server-symbolic",
            Self::Other => "network-workgroup-symbolic",
        }
    }

    /// Get display label for this category
    pub fn display_label<'a>(&self, interface_name: &'a str) -> &'a str {
        match self {
            Self::Vpn => "Tailscale VPN",
            Self::WiFi => "WiFi",
            Self::Ethernet => "Ethernet",
            Self::Docker => "Docker",
            Self::Other => interface_name,
        }
    }
}

/// Filters for which network interface categories to display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceFilters {
    /// Show WiFi interfaces (wl*)
    pub show_wifi: bool,
    /// Show Ethernet interfaces (en*, eth*)
    pub show_ethernet: bool,
    /// Show VPN interfaces (tailscale*, tun*)
    pub show_vpn: bool,
    /// Show Docker/bridge interfaces (docker*, br-*)
    pub show_docker: bool,
    /// Show other unrecognized interfaces
    pub show_other: bool,
}

impl Default for InterfaceFilters {
    fn default() -> Self {
        Self {
            show_wifi: true,
            show_ethernet: true,
            show_vpn: true,
            show_docker: false, // Hidden by default (internal)
            show_other: true,
        }
    }
}

impl InterfaceFilters {
    /// Check if an interface category should be shown
    pub fn should_show(&self, category: InterfaceCategory) -> bool {
        match category {
            InterfaceCategory::WiFi => self.show_wifi,
            InterfaceCategory::Ethernet => self.show_ethernet,
            InterfaceCategory::Vpn => self.show_vpn,
            InterfaceCategory::Docker => self.show_docker,
            InterfaceCategory::Other => self.show_other,
        }
    }

    /// Check if any category is enabled
    pub fn any_enabled(&self) -> bool {
        self.show_wifi || self.show_ethernet || self.show_vpn || self.show_docker || self.show_other
    }
}

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
    /// Maximum retry attempts for failed transfers
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Delay between retry attempts in milliseconds
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,
    /// Optional bandwidth limit (bytes per second). None means unlimited.
    #[serde(default)]
    pub bandwidth_limit_bps: Option<u64>,
    /// Interface category visibility filters
    #[serde(default)]
    pub interface_filters: InterfaceFilters,
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    1000
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
            max_retries: default_max_retries(),
            retry_delay_ms: default_retry_delay_ms(),
            bandwidth_limit_bps: None,
            interface_filters: InterfaceFilters::default(),
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
            .max_retries(self.max_retries)
            .retry_delay_ms(self.retry_delay_ms)
            .bandwidth_limit_bps(self.bandwidth_limit_bps)
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
        assert_eq!(config.port, 53317);
    }
}
