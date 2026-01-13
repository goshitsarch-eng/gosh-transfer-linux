# Architecture

This document describes the internal architecture of Gosh Transfer Linux, a GTK4/Libadwaita frontend for the gosh-lan-transfer engine.

## Overview

The application is a Rust workspace with two crates that provide a native Linux desktop experience for peer-to-peer file transfers.

```
┌─────────────────────────────────────────────────────────┐
│                    GTK4 Main Loop                        │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              gosh-transfer-gtk                      │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │ │
│  │  │ SendView │ │ReceiveView│ │SettingsView│ ...      │ │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘            │ │
│  │       │            │            │                   │ │
│  │       └────────────┼────────────┘                   │ │
│  │                    ▼                                │ │
│  │           ┌─────────────────┐                       │ │
│  │           │  EngineBridge   │                       │ │
│  │           │ (async_channel) │                       │ │
│  │           └────────┬────────┘                       │ │
│  └────────────────────┼────────────────────────────────┘ │
└───────────────────────┼─────────────────────────────────┘
                        │
┌───────────────────────┼─────────────────────────────────┐
│                       ▼            Tokio Runtime         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              GoshTransferEngine                     │ │
│  │         (from gosh-lan-transfer crate)              │ │
│  │                                                     │ │
│  │  • HTTP Server (Axum) on port 53317                 │ │
│  │  • HTTP Client (Reqwest) for sending                │ │
│  │  • Transfer protocol implementation                 │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                        │
┌───────────────────────┼─────────────────────────────────┐
│                       ▼                                  │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              gosh-transfer-core                     │ │
│  │                                                     │ │
│  │  • SettingsStore     (~/.config/gosh-transfer/)     │ │
│  │  • FileFavoritesStore                               │ │
│  │  • TransferHistory                                  │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## Crate Structure

### `gosh-transfer-core`

Shared business logic, UI-agnostic. Location: `crates/gosh-transfer-core/`

```
src/
├── lib.rs          # Re-exports and engine type re-exports
├── types.rs        # AppSettings, AppError definitions
├── settings.rs     # SettingsStore (persistent settings)
├── favorites.rs    # FileFavoritesStore (implements FavoritesPersistence)
└── history.rs      # TransferHistory (persistent, max 100 entries)
```

Key types:
- `AppSettings` - Serializable settings struct with `to_engine_config()` conversion
- `SettingsStore` - Thread-safe RwLock-wrapped settings with JSON persistence
- `FileFavoritesStore` - Implements `gosh_lan_transfer::FavoritesPersistence` trait
- `TransferHistory` - FIFO queue with disk persistence

### `gosh-transfer-gtk`

GTK4/Libadwaita frontend. Location: `crates/gosh-transfer-gtk/`

```
src/
├── main.rs              # Entry point, logging setup
├── application.rs       # GoshTransferApplication (Adw::Application subclass)
├── window/
│   ├── mod.rs           # GoshTransferWindow wrapper
│   └── imp.rs           # Window implementation, navigation, engine events
├── views/
│   ├── mod.rs           # View exports
│   ├── send.rs          # SendView - file sending UI
│   ├── receive.rs       # ReceiveView - incoming transfers
│   ├── transfers.rs     # TransfersView - history display
│   ├── settings.rs      # SettingsView - configuration UI
│   └── about.rs         # AboutView - app info
├── services/
│   ├── mod.rs           # Service exports
│   └── engine_bridge.rs # EngineBridge - async/sync bridge
└── widgets/
    └── mod.rs           # Custom widget exports (empty currently)
```

## EngineBridge: The Critical Component

The `EngineBridge` (`services/engine_bridge.rs`) solves the fundamental mismatch between GTK's synchronous main loop and the async `GoshTransferEngine`.

### How it works

1. **Initialization**: Creates a Tokio runtime with 2 worker threads
2. **Command channel**: UI sends `EngineCommand` variants via `async_channel::Sender`
3. **Event channel**: Engine events forwarded to UI via `async_channel::Receiver`
4. **GTK integration**: Uses `glib::spawn_future_local()` to bridge async operations

### Command Types

```rust
pub enum EngineCommand {
    StartServer,
    StopServer,
    ResolveAddress { address: String, reply: Sender<ResolveResult> },
    SendFiles { address: String, port: u16, paths: Vec<PathBuf> },
    AcceptTransfer { id: String },
    RejectTransfer { id: String },
    GetPendingTransfers { reply: Sender<Vec<PendingTransfer>> },
    GetInterfaces { reply: Sender<Vec<NetworkInterface>> },
    UpdateConfig { config: EngineConfig },
}
```

### Data Flow

**UI → Engine:**
```
User action → View method → EngineBridge.send_files()
    → glib::spawn_future_local → async_channel::send
    → Tokio runtime → GoshTransferEngine.send_files()
```

**Engine → UI:**
```
GoshTransferEngine event → mpsc::Receiver
    → EngineBridge task → async_channel::send
    → Window event handler (glib::spawn_future_local)
    → View update
```

## Application Lifecycle

1. `main.rs`: Initialize tracing, create `GoshTransferApplication`
2. `application.rs`: On `activate`:
   - Initialize `SettingsStore`, `FileFavoritesStore`, `TransferHistory`
   - Create `EngineBridge` with settings converted to `EngineConfig`
   - Create and present `GoshTransferWindow`
3. `window/imp.rs`: On `constructed`:
   - Setup navigation sidebar
   - Create all views
   - `setup_engine_events()` starts server and subscribes to events

## Event Handling

The window subscribes to engine events in `setup_engine_events()`:

| Event | Handler |
|-------|---------|
| `TransferRequest` | Add to pending, show badge |
| `TransferProgress` | Update progress bar, move to active |
| `TransferComplete` | Mark complete, remove after 3s |
| `TransferFailed` | Show error, remove after 5s |
| `ServerStarted` | Log port |
| `ServerStopped` | Log |

## Configuration

Configuration path determined by `directories::ProjectDirs::from("com", "gosh", "transfer")`:
- Linux: `~/.config/gosh-transfer/`

Files:
| File | Format | Max Size |
|------|--------|----------|
| `settings.json` | JSON | ~500 bytes |
| `favorites.json` | JSON | Unbounded |
| `history.json` | JSON | 100 entries |

## Constants

| Constant | Value | Location |
|----------|-------|----------|
| App ID | `com.gosh.Transfer` | `main.rs:12` |
| Default port | 53317 | `types.rs:39` |
| Window size | 1024×768 | `window/imp.rs:19-20` |
| Tokio workers | 2 | `engine_bridge.rs:62` |
| Command channel | 32 | `engine_bridge.rs:56` |
| Event channel | 64 | `engine_bridge.rs:57` |
| History max | 100 | `history.rs:13` |

## External Dependencies

The transfer protocol implementation lives entirely in [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer) (git dependency). This crate provides:
- `GoshTransferEngine` - Main engine struct
- `EngineConfig` - Configuration builder
- `EngineEvent` - Event enum for transfer lifecycle
- `FavoritesPersistence` - Trait for custom favorites storage
- Type definitions for transfers, peers, progress, etc.

This application does not implement any HTTP server/client logic itself.

## Engine Capabilities vs GTK Implementation

The `gosh-lan-transfer` engine supports more features than currently exposed in the GTK frontend:

| Capability | Engine | GTK Frontend |
|------------|--------|--------------|
| Send files | ✓ | ✓ |
| Send directories (preserving structure) | ✓ | ✗ |
| Accept/reject transfers | ✓ | ✓ |
| Batch accept/reject | ✓ | ✗ |
| Cancel mid-transfer | ✓ | ✗ |
| Runtime port change | ✓ | ✗ |
| Automatic retry with backoff | ✓ | ✓ (via engine) |
| Progress tracking | ✓ | ✓ |
| TransferRetry event | ✓ | ✗ (not handled) |
| Peer health checks | ✓ | ✗ |

These represent potential future enhancements for the GTK frontend.
