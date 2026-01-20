# Qt6 Migration Plan (Completed)

Goal: migrate the GTK frontend to Qt6 Widgets while keeping the Rust backend and ensuring full engine parity (no feature loss).

## 1) Feature Parity Matrix

Use this list as the single source of truth. Every item must be implemented in the Qt UI or explicitly documented as intentionally not exposed.

### Engine Capabilities (from gosh-lan-transfer docs)

- Send files to IP/hostname
- Send directories (preserve structure)
- Receive files with approval workflow
- Auto-accept from trusted hosts
- Cancel in-progress transfers
- Batch accept/reject pending transfers
- Progress updates with speed
- Retry with exponential backoff (request retry only; not chunk uploads)
- Transfer history persistence trait
- Favorites persistence trait
- Hostname resolution
- Network interface enumeration
- Peer health check
- Peer info retrieval (`/info`)
- Runtime port change with rollback on failure
- Optional no-rollback port change
- Server start/stop; running status
- IPv6 dual-stack binding
- File name conflict resolution `(n)` suffix up to 1000 attempts
- Approval timeout (~2 minutes)
- Path traversal protection and relative path sanitization
- Size validation (delete oversized files)
- Progress throttling (~32KB increments)
- SSE `/events` stream (engine feature; UI may not consume)
- Bandwidth limit config field (engine config; throttling not implemented)

### App Requirements (from local docs)

- Send/Receive flows
- Favorites management
- Trusted hosts list
- Receive-only mode
- Notifications toggle
- Theme selection (system/light/dark)
- Retry settings (max retries, delay)
- Interface category filters (WiFi/Ethernet/VPN/Docker/Other)
- History list (cap 100 entries) + clear
- About page + links
- “No magic” explicit addressing UX

## 2) Architecture Decisions

- Rust UI integration: choose `cxx-qt` (or alternative) and lock decision.
- Keep `crates/gosh-transfer-core` unchanged.
- Create `crates/gosh-transfer-qt` as the new UI crate.
- Replace GTK main loop with Qt Widgets main thread.
- Bridge Tokio runtime with Qt signal/slot emitter (thread-safe).

## 3) Engine Bridge (Qt)

Required commands (UI → engine):
- Start/Stop server
- Resolve hostname
- Send files / Send directory
- Accept/Reject transfer
- Accept/Reject all
- Cancel transfer
- Check peer
- Get interfaces
- Update config
- Change port (+ optional no-rollback)
- Get pending transfers (if needed for UI refresh)

Required events (engine → UI):
- TransferRequest
- TransferProgress
- TransferComplete
- TransferFailed
- TransferRetry
- ServerStarted
- ServerStopped
- PortChanged

Rules:
- Marshal all engine events onto the Qt main thread.
- Ensure clean shutdown order (stop server → join runtime tasks).
- Keep event ordering stable for UI state transitions (pending → active → done).

## 4) UI Port Checklist (Qt Widgets)

Main Window + Navigation:
- Sidebar nav (Send / Receive / Transfers / Settings / About)
- Status indicator + pending badge

Send View:
- Address entry + live hostname resolution feedback
- Favorites dropdown + manage dialog
- Test connection button
- File picker (multi-select)
- Folder picker
- Send button
- Receive-only mode disables send

Receive View:
- Pending list with Accept/Reject
- Batch Accept/Reject
- Active transfers with progress + speed
- Cancel in-progress transfer

Transfers View:
- History list with status + relative time
- Clear history

Settings View:
- Device name
- Download directory
- Trusted hosts list (add/remove)
- Receive-only toggle
- Notifications toggle
- Theme (system/light/dark)
- Retry settings (max retries, delay)
- Interface filters
- Port change at runtime
- Bandwidth limit (if exposed; note engine does not throttle yet)

About View:
- App name, version, license, links, icon

## 5) Migration Gaps to Close During Port

- Runtime port change UI and logic using engine’s change_port (and optional no-rollback).
- Peer info fetch (show device name on test connection results).
- IPv6 address entry validation (do not reject IPv6).
- Correct messaging for:
  - Retry behavior (request only)
  - Approval timeout (2 minutes)
  - Filename conflict handling `(n)` suffix

## 6) Bug Hunt / Risk Areas

- Pending → active transfer state transitions (avoid duplicate rows).
- Transfer cancel timing vs. progress events.
- Hostname resolution debouncing (avoid stale results).
- History persistence and max 100 entries.
- Trusted hosts auto-accept behavior.
- Progress throttling (do not assume events for every chunk).
- Port change rollback handling and UI status.

## 7) Packaging + Docs

- Add `gosh-transfer-qt` to workspace.
- Update desktop/metainfo entries and Flatpak manifest.
- Replace GTK dependencies with Qt6 dev dependencies in README.
- Keep binary naming consistent; update CI artifacts.

## 8) Verification Checklist

LAN/VPN/Tailscale test matrix:
- Send single + multiple files
- Send directory (nested paths)
- Accept/Reject single
- Batch Accept/Reject
- Cancel transfer mid-way
- Retry scenario (transient failure)
- Port change success + rollback
- Trusted hosts auto-accept
- IPv4 + IPv6
- History persistence + clear
- Interface filtering

## Status Tracker

Legend: [ ] not started, [~] in progress, [x] done

Architecture/Setup
- [x] Qt integration choice documented
- [x] `gosh-transfer-qt` crate created
- [x] Engine bridge ported

UI Port
- [x] Main window/nav
- [x] Send view
- [x] Receive view
- [x] Transfers view
- [x] Settings view
- [x] About view
- [x] Dialogs (favorites/trusted hosts)

Feature Parity
- [x] Runtime port change
- [x] Peer info display
- [x] IPv6 support
- [x] Retry/timeout messaging
- [x] Filename conflict behavior
- [x] Bandwidth limit decision documented

Packaging
- [x] Desktop/metainfo updates
- [x] Flatpak manifest update
- [x] README build deps update

Verification
- [ ] Manual QA pass across LAN/VPN/Tailscale
- [ ] Known issues documented
- [x] Qt build (`cargo build -p gosh-transfer-qt`)
