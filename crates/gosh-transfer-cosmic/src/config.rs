// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer COSMIC - Configuration

use gosh_lan_transfer::EngineConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmicConfig {
    pub port: u16,
    pub device_name: String,
    pub download_dir: PathBuf,
    pub trusted_hosts: Vec<String>,
    pub receive_only: bool,
    pub notifications_enabled: bool,
    pub theme: String,
}

impl Default for CosmicConfig {
    fn default() -> Self {
        let download_dir = directories::UserDirs::new()
            .and_then(|d: directories::UserDirs| d.download_dir().map(|p: &std::path::Path| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            port: 53317,
            device_name: hostname::get()
                .map(|h: std::ffi::OsString| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "COSMIC Device".to_string()),
            download_dir,
            trusted_hosts: Vec::new(),
            receive_only: false,
            notifications_enabled: true,
            theme: "system".to_string(),
        }
    }
}

impl CosmicConfig {
    pub fn to_engine_config(&self) -> EngineConfig {
        EngineConfig::builder()
            .port(self.port)
            .device_name(&self.device_name)
            .download_dir(&self.download_dir)
            .trusted_hosts(self.trusted_hosts.clone())
            .receive_only(self.receive_only)
            .build()
    }
}
