# Product Requirements Document

## Product Overview

**Product Name:** Gosh Transfer Linux

**Description:** A native GTK4/Libadwaita desktop application for explicit peer-to-peer file transfers over local networks, VPNs, and Tailscale.

**Core Principle:** No magic, no cloud, no auto-discovery. Users specify exactly where files go using IP addresses or hostnames.

## Objectives

1. Provide a native Linux desktop experience for file transfers
2. Enable explicit, user-controlled file sharing without cloud dependencies
3. Support modern network configurations (LAN, VPN, Tailscale)
4. Follow GNOME Human Interface Guidelines for familiar UX

## Target Users

### Primary: Linux Desktop Users
- Technical users comfortable with IP addresses and hostnames
- Privacy-conscious individuals avoiding cloud services
- Users on corporate or home networks with multiple devices
- Tailscale and VPN users needing cross-network transfers

### Secondary: Linux Newcomers
- Users migrating from Windows/macOS seeking familiar functionality
- Those exploring Linux who need reliable file sharing tools

## Use Cases

### UC1: Send Files to Known Peer
**Actor:** User with files to send
**Flow:**
1. User opens application
2. User enters destination IP/hostname
3. User selects files via file picker
4. User clicks "Send Files"
5. Recipient receives transfer request
6. Upon acceptance, files transfer with progress display
7. Both users see completion notification

### UC2: Receive Files from Peer
**Actor:** User expecting incoming files
**Flow:**
1. User opens application (server starts automatically)
2. Application displays local IP addresses
3. User shares address with sender
4. Transfer request appears with sender info and file list
5. User accepts or rejects transfer
6. Accepted files download to configured directory
7. Progress displays during transfer

### UC3: Quick Transfer to Favorite
**Actor:** User with frequently-used peer
**Flow:**
1. User opens application
2. User selects peer from Favorites dropdown
3. Address auto-fills
4. User selects files and sends

### UC4: Auto-Accept from Trusted Host
**Actor:** User receiving from trusted source
**Flow:**
1. User adds peer IP to trusted hosts in Settings
2. Future transfers from that IP auto-accept
3. Files download without manual intervention

## Functional Requirements

### FR1: File Sending
- [x] Select multiple files via native file picker
- [x] Select directories for transfer
- [x] Enter destination as IP address or hostname
- [x] Resolve hostnames to IP addresses with visual feedback
- [x] Display transfer progress with speed
- [x] Handle transfer failures gracefully

### FR2: File Receiving
- [x] Start HTTP server on application launch
- [x] Display local network addresses (with category filtering)
- [x] Show pending transfer requests with file details
- [x] Accept or reject individual transfers
- [x] Batch accept/reject all pending transfers
- [x] Cancel in-progress transfers
- [x] Save files to user-configured directory
- [ ] Handle filename conflicts (append number) — handled by engine

### FR3: Favorites Management
- [x] Save peer with custom name and address
- [x] List saved favorites in dropdown
- [x] Delete favorites
- [x] Auto-fill address when favorite selected

### FR4: Settings
- [x] Configure device name (shown to peers)
- [x] Set download directory
- [x] Manage trusted hosts list
- [x] Toggle receive-only mode
- [x] Select theme (system/light/dark)
- [x] Toggle notifications
- [x] Configure retry behavior (max retries, delay)
- [x] Filter interface categories (WiFi, Ethernet, VPN, Docker, Other)

### FR5: Transfer History
- [x] Record all completed/failed transfers — via engine
- [x] Display history with direction, peer, files, status, relative time
- [x] Persist history across sessions (max 100 entries)
- [x] Clear history button

## Non-Functional Requirements

### NFR1: Performance
- Application startup: < 500ms
- Memory usage: < 100MB idle
- Transfer throughput: > 95% of network capacity
- UI remains responsive during transfers

### NFR2: Usability
- Follow GNOME HIG for layout and interactions
- Provide clear feedback for all actions
- Display meaningful error messages
- Support keyboard navigation

### NFR3: Reliability
- Graceful handling of network interruptions
- Automatic retry with exponential backoff
- No data loss on application crash
- Persistent configuration

### NFR4: Security
- No telemetry or analytics
- No cloud connectivity
- Files only written to user-specified directory
- Path traversal prevention

## Constraints

1. **Linux-only**: GTK4/Libadwaita requires Linux (or compatible)
2. **Trusted networks**: No built-in encryption; relies on network security
3. **IPv4 focus**: Primary support for IPv4 addresses
4. **Single instance**: One server per machine on port 53317

## Success Metrics

| Metric | Target |
|--------|--------|
| Installation success rate | > 95% |
| Transfer completion rate | > 99% on stable networks |
| User-reported bugs | < 5 critical per release |
| Memory leaks | 0 |

## Future Considerations

Features supported by the engine but not yet exposed in the UI:

1. **Runtime port change**: The engine supports dynamic port changes, but the GTK frontend recommends an app restart after port changes

Previously planned features now implemented:
- Directory sending (v2.0.3+)
- Batch accept/reject (v2.0.3+)
- Transfer cancellation (v2.0.3+)
- Interface category filtering (unreleased)
- Peer health checks via test connection button (unreleased)
- Address resolution with live feedback (unreleased)

## Timeline

This document describes requirements for version 2.x. Specific release dates are determined by development progress.

## Appendix: Comparison with Alternatives

| Feature | Gosh Transfer | LocalSend | Warpinator |
|---------|---------------|-----------|------------|
| Auto-discovery | No | Yes | Yes |
| Cloud dependency | No | No | No |
| Explicit addressing | Yes | No | No |
| GTK4 native | Yes | No | Yes |
| Cross-platform | Linux only | Yes | Linux only |
