# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Real-time hostname resolution feedback in send view with visual status

### Changed
- Simplified to GTK4-only build, removing Qt and Cosmic frontends
- Now a pure Rust application using GTK4/Libadwaita
- Moved Pending Transfers above Your Addresses in receive view for quicker access
- Moved Trusted Hosts above Appearance in settings
- Updated port description to "Port for file transfers"
- Updated copyright to 2026 Goshitsarch

### Removed
- Qt6/QML frontend (`gosh-transfer-qt`)
- libcosmic frontend (`gosh-transfer-cosmic`)
- Windows and macOS build workflows (Qt-based)

## [2.0.3] - 2026-01-06

### Added
- macOS DMG build workflow with code signing
- Windows multi-architecture support (x64, ARM64)
- Qt6/QML frontend for Windows/macOS

### Fixed
- macOS signing identity extraction

### Changed
- Switched to `local-ip-address` crate for network interface detection

## [2.0.2] - 2026-01-04

### Fixed
- Increased connect timeout for slower networks
- Updated event field names for consistency

## [2.0.1] - 2026-01-04

### Fixed
- Version display in About view

## [2.0.0] - 2026-01-04

### Added
- GTK4/Libadwaita native Linux frontend
- Persistent transfer history (max 100 entries)
- Linux ARM64 build support
- Flatpak packaging

### Changed
- Modernized to Rust 2021 edition
- Upgraded to GTK4 0.9 and Libadwaita 0.7
- Powered by `gosh-lan-transfer` engine library

### Removed
- Tauri/Svelte frontend
- AppImage builds

## [1.0.0] - 2026-01-03

### Added
- Initial release
- File transfer over LAN/VPN/Tailscale
- Favorites for saved peer addresses
- Trusted hosts for auto-accept
- Receive-only mode
- Dark/light/system theme support
