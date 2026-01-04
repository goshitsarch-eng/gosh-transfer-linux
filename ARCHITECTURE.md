# Architecture

This document describes the internal architecture of Gosh Transfer based on code analysis.

## Module Structure

### Backend (`src-tauri/src/`)

```
src/
├── main.rs         # Entry point, calls lib::run()
├── lib.rs          # App initialization, server startup, event forwarding
├── types.rs        # All shared data structures
├── server.rs       # Axum HTTP server for receiving files
├── client.rs       # HTTP client for sending files
├── commands.rs     # Tauri IPC command handlers
├── settings.rs     # Settings persistence
└── favorites.rs    # Favorites persistence
```

### Frontend (`src/`)

```
src/
├── main.js                     # Svelte app mount
├── App.svelte                  # Main layout, navigation, event listeners
├── lib/
│   ├── theme.js                # Theme switching logic
│   └── components/
│       ├── SendView.svelte     # File sending UI
│       ├── ReceiveView.svelte  # Incoming transfer UI
│       ├── TransfersView.svelte # Transfer history
│       ├── SettingsView.svelte  # Settings UI
│       └── AboutView.svelte     # About page
└── styles/
    └── global.css              # Global styles
```

## State Management

### AppState (`commands.rs:19-26`)

Central state managed by Tauri:

```rust
pub struct AppState {
    pub favorites: FavoritesStore,        // Persisted to disk
    pub client: TransferClient,           // HTTP client singleton
    pub server_state: Arc<ServerState>,   // Shared with server task
    pub settings_store: SettingsStore,    // Persisted to disk
    pub settings: RwLock<AppSettings>,    // In-memory copy
    pub transfer_history: RwLock<Vec<TransferRecord>>, // Volatile
}
```

### ServerState (`server.rs:40-55`)

State for the HTTP server:

```rust
pub struct ServerState {
    pub settings: RwLock<AppSettings>,
    pub pending_transfers: RwLock<HashMap<String, PendingTransfer>>,
    pub approved_tokens: RwLock<HashMap<String, String>>,
    pub rejected_transfers: RwLock<HashMap<String, String>>,
    pub received_files: RwLock<HashMap<String, HashSet<String>>>,
    pub event_tx: broadcast::Sender<ServerEvent>,
    pub download_dir: RwLock<PathBuf>,
}
```

## Event Flow

### Server to Frontend

```
Server (Axum)
    → ServerEvent (broadcast channel)
    → lib.rs event forwarder
    → Tauri emit()
    → Frontend listen()
```

Events:
- `transfer-request` - New incoming transfer
- `transfer-progress` - Bytes received update
- `transfer-complete` - All files received
- `transfer-failed` - Error during receive

### Client to Frontend

```
TransferClient
    → TransferProgress (broadcast channel)
    → lib.rs event forwarder
    → Tauri emit("send-progress")
    → Frontend listen()
```

### Frontend to Backend

```
Frontend invoke()
    → Tauri command handler
    → AppState access
    → Result returned
```

## Transfer Protocol Detail

### Sending Files

1. **Resolve hostname** (`client.rs:61-102`)
   - If already IP: return immediately
   - Otherwise: DNS lookup via `ToSocketAddrs`

2. **Request transfer** (`client.rs:155-197`)
   - POST to `/transfer` with `TransferRequest`
   - Contains: transfer_id, sender_name, files[], total_size

3. **Handle response**
   - If `accepted: true`: proceed with token
   - If `accepted: false`: poll `/transfer/status` every 500ms for 120s

4. **Send files** (`client.rs:252-344`)
   - For each file: POST to `/chunk` with query params
   - Stream file content as body
   - Track progress with atomic counter

### Receiving Files

1. **Accept request** (`server.rs:196-278`)
   - Create `PendingTransfer` record
   - If trusted host: auto-accept with token
   - Otherwise: emit `TransferRequest` event, return pending

2. **Status polling** (`server.rs:281-319`)
   - Check approved_tokens map
   - Check rejected_transfers map
   - Check pending_transfers map
   - Return appropriate status

3. **Receive chunks** (`server.rs:322-513`)
   - Verify token matches
   - Create file with conflict resolution
   - Stream body to file
   - Emit progress events
   - Track received files per transfer
   - Emit complete when all files received

## File Conflict Resolution (`server.rs:141-173`)

When writing received files:

```
file.txt → file.txt
file.txt (exists) → file (1).txt
file (1).txt (exists) → file (2).txt
...up to (999)
```

Uses `OpenOptions::create_new(true)` for atomic uniqueness check.

## Configuration Paths

Determined by `directories::ProjectDirs::from("com", "gosh", "transfer")`:

| Platform | Path |
|----------|------|
| Linux | `~/.config/gosh-transfer/` |
| macOS | `~/Library/Application Support/com.gosh.transfer/` |
| Windows | `%APPDATA%\gosh\transfer\config\` |

## Tauri Commands Reference

### Favorites
- `list_favorites() → Vec<Favorite>`
- `add_favorite(name, address) → Favorite`
- `update_favorite(id, name?, address?) → Favorite`
- `delete_favorite(id)`

### Network
- `resolve_hostname(address) → ResolveResult`
- `get_interfaces() → Vec<NetworkInterface>`
- `check_peer(address, port) → bool`
- `get_peer_info(address, port) → JSON`

### Transfers
- `send_files(address, port, file_paths)`
- `accept_transfer(transfer_id) → token`
- `reject_transfer(transfer_id)`
- `get_pending_transfers() → Vec<PendingTransfer>`
- `get_transfer_history() → Vec<TransferRecord>`
- `clear_transfer_history()`

### Settings
- `get_settings() → AppSettings`
- `update_settings(new_settings)`
- `add_trusted_host(host)`
- `remove_trusted_host(host)`

### Server
- `get_server_status() → JSON`

## Timeouts and Constants

| Constant | Value | Location |
|----------|-------|----------|
| Server port | 53317 | `lib.rs:61` |
| Connect timeout | 10s | `client.rs:45` |
| Read timeout | 60s | `client.rs:44` |
| Approval timeout | 120s | `client.rs:209` |
| Approval poll interval | 500ms | `client.rs:210` |
| Progress throttle | 32KB | `client.rs:300` |
| Filename conflict limit | 1000 | `server.rs:147` |
| Event channel capacity | 100 | `server.rs:73` |

## Unimplemented Features

Based on code analysis, these are declared but not functional:

1. **Port configuration** - Setting stored but server uses hardcoded value
2. **System notifications** - `notifications_enabled` setting exists but no notification code
3. **Transfer speed** - `speed_bps` always 0 (has TODO comment)
4. **IPv6** - Code comment claims dual-stack but only binds IPv4
5. **Transfer history persistence** - In-memory only
