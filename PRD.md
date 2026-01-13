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
- [ ] Select multiple files via native file picker
- [ ] Enter destination as IP address or hostname
- [ ] Resolve hostnames to IP addresses
- [ ] Display transfer progress with speed
- [ ] Handle transfer failures gracefully

### FR2: File Receiving
- [ ] Start HTTP server on application launch
- [ ] Display all local network addresses
- [ ] Show pending transfer requests with file details
- [ ] Accept or reject individual transfers
- [ ] Save files to user-configured directory
- [ ] Handle filename conflicts (append number)

### FR3: Favorites Management
- [ ] Save peer with custom name and address
- [ ] List saved favorites in dropdown
- [ ] Delete favorites
- [ ] Auto-fill address when favorite selected

### FR4: Settings
- [ ] Configure device name (shown to peers)
- [ ] Set download directory
- [ ] Manage trusted hosts list
- [ ] Toggle receive-only mode
- [ ] Select theme (system/light/dark)
- [ ] Toggle notifications

### FR5: Transfer History
- [ ] Record all completed/failed transfers
- [ ] Display history with direction, peer, files, status
- [ ] Persist history across sessions (max 100 entries)

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

1. **Directory sending**: Transfer entire folder structures
2. **Batch operations**: Accept/reject multiple transfers at once
3. **Transfer cancellation**: Stop in-progress transfers
4. **Runtime port change**: Modify server port without restart
5. **Peer health checks**: Verify connectivity before sending

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
