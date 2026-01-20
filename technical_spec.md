# Technical Specification

## System Architecture

Gosh Transfer Linux is a native desktop application that provides a graphical interface for the `gosh-lan-transfer` file transfer engine. It ships a Qt6 Widgets frontend.

### High-Level Architecture

```
┌────────────────────────────────────────────────────┐
│                 Qt6 Main Thread                     │
│  ┌──────────────────────────────────────────────┐  │
│  │           gosh-transfer-qt                   │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐           │  │
│  │  │  Views │ │ Window │ │  App   │           │  │
│  │  └───┬────┘ └───┬────┘ └───┬────┘           │  │
│  │      └──────────┼──────────┘                │  │
│  │                 ▼                           │  │
│  │         ┌─────────────┐                     │  │
│  │         │EngineBridge │                     │  │
│  │         └──────┬──────┘                     │  │
│  └────────────────┼─────────────────────────────┘  │
└───────────────────┼────────────────────────────────┘
                    │ async_channel
┌───────────────────┼────────────────────────────────┐
│                   ▼           Tokio Runtime        │
│  ┌──────────────────────────────────────────────┐  │
│  │         GoshTransferEngine                   │  │
│  │    (from gosh-lan-transfer crate)            │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
                    │
┌───────────────────┼────────────────────────────────┐
│                   ▼                                │
│  ┌──────────────────────────────────────────────┐  │
│  │           gosh-transfer-core                 │  │
│  │  • SettingsStore                             │  │
│  │  • FileFavoritesStore                        │  │
│  │  • TransferHistory                           │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

## Technology Stack

| Layer | Technology | Version |
|-------|------------|---------|
| UI Framework | Qt6 Widgets | 6.x |
| Language | Rust | 2021 edition |
| Async Runtime | Tokio | Multi-threaded |
| Transfer Engine | gosh-lan-transfer | Git dependency |
| Serialization | Serde + JSON | Latest |
| Logging | tracing + tracing-subscriber | Latest |

## Crate Structure

### Workspace Layout

```
gosh-transfer-linux/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── gosh-transfer-core/ # Shared business logic
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs
│   │       ├── settings.rs
│   │       ├── favorites.rs
│   │       └── history.rs
│   └── gosh-transfer-qt/   # Qt6 Widgets frontend
│       └── src/
│           ├── main.rs
│           ├── engine_bridge.rs
│           └── qt/
└── .github/workflows/      # CI/CD
```

### Dependencies

**gosh-transfer-core:**
- `gosh-lan-transfer` - Transfer engine (implements protocol)
- `serde`, `serde_json` - Configuration serialization
- `directories` - Platform config paths
- `uuid` - Transfer ID generation
- `chrono` - Timestamps

**gosh-transfer-qt:**
- `cxx-qt` - Qt bridge
- `cxx-qt-lib` - Qt bindings
- `async-channel` - Async/sync bridge
- `tokio` - Async runtime
- `tracing` - Logging

## Data Models

### AppSettings

```rust
pub struct AppSettings {
    pub port: u16,                      // Default: 53317
    pub device_name: String,            // Default: hostname
    pub download_dir: PathBuf,          // Default: ~/Downloads
    pub trusted_hosts: Vec<String>,     // Auto-accept IPs
    pub receive_only: bool,             // Disable sending
    pub notifications_enabled: bool,    // System notifications
    pub theme: String,                  // "system", "light", "dark"
    pub max_retries: u32,               // Default: 3
    pub retry_delay_ms: u64,            // Default: 1000
    pub interface_filters: InterfaceFilters, // Category visibility
}

pub struct InterfaceFilters {
    pub show_wifi: bool,      // Default: true
    pub show_ethernet: bool,  // Default: true
    pub show_vpn: bool,       // Default: true
    pub show_docker: bool,    // Default: false
    pub show_other: bool,     // Default: true
}
```

### Favorite

```rust
pub struct Favorite {
    pub id: String,                   // UUID
    pub name: String,                 // Display name
    pub address: String,              // IP or hostname
    pub last_resolved_ip: Option<String>, // Cached resolved IP
    pub last_used: Option<DateTime<Utc>>, // Last usage timestamp
}
```

Note: `Favorite` is defined in the `gosh-lan-transfer` crate. The `last_resolved_ip` field is updated by the Qt frontend when hostname resolution succeeds.

### TransferRecord

```rust
pub struct TransferRecord {
    pub id: String,
    pub direction: TransferDirection, // Send or Receive
    pub peer_address: String,
    pub peer_name: Option<String>,
    pub files: Vec<FileInfo>,
    pub total_bytes: u64,
    pub status: TransferStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

## Data Storage

### Configuration Directory

Location: `~/.config/gosh/transfer/` (via `directories` crate with `ProjectDirs::from("com", "gosh", "transfer")`)

| File | Purpose | Format | Limits |
|------|---------|--------|--------|
| `settings.json` | Application settings | JSON | ~500 bytes |
| `favorites.json` | Saved peer addresses | JSON | Unbounded |
| `history.json` | Transfer history | JSON | 100 entries max |

### Persistence Strategy

- **Settings**: Read on startup, write on save button click
- **Favorites**: Read on startup, write immediately on add/remove
- **History**: Read on startup, write on each transfer completion, FIFO eviction at 100 entries

## Network Protocol

Implemented by `gosh-lan-transfer` crate. This application does not implement protocol logic.

### Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Server availability check |
| `/transfer` | POST | Initiate transfer request |
| `/transfer/status` | GET | Poll approval status |
| `/chunk` | POST | Stream file data |

### Security Model

- UUID tokens authorize uploads per transfer
- Filename sanitization prevents path traversal
- Designed for trusted networks (LAN/VPN/Tailscale)
- No built-in encryption (rely on network-level security)

## EngineBridge Pattern

The `EngineBridge` solves the async/sync impedance mismatch between the Qt UI thread and the async transfer engine.

### Command Flow (UI → Engine)

```
User action
  → View method call
  → EngineBridge.method()
  → Qt signal dispatch (queued)
  → async_channel::send(EngineCommand)
  → Tokio runtime receives
  → GoshTransferEngine.method()
```

### Event Flow (Engine → UI)

```
GoshTransferEngine event
  → mpsc::Receiver in bridge task
  → async_channel::send(EngineEvent)
  → Window event handler (Qt signal/slot)
  → View update methods
```

### Channel Configuration

| Channel | Capacity | Purpose |
|---------|----------|---------|
| Command | 32 | UI commands to engine |
| Event | 64 | Engine events to UI |

## Performance Requirements

| Metric | Target |
|--------|--------|
| Startup time | < 500ms |
| Memory usage | < 100MB idle |
| Transfer overhead | < 5% of raw throughput |
| UI responsiveness | 60 FPS during transfers |

## Infrastructure

### Build Targets

| Target | Workflow | Output |
|--------|----------|--------|
| Linux x86_64 | `release.yml` | `gosh-transfer-linux-qt` binary |
| Linux ARM64 | `release.yml` | `gosh-transfer-linux-qt` binary |
| Flatpak | `release.yml` | `.flatpak` bundle |

### System Dependencies

**Ubuntu/Debian:**
```bash
libgtk-4-dev libadwaita-1-dev libssl-dev pkg-config
```

**Fedora:**
```bash
gtk4-devel libadwaita-devel openssl-devel
```

**Qt6 (Ubuntu/Debian):**
```bash
qt6-base-dev libssl-dev pkg-config
```

**Qt6 (Fedora):**
```bash
qt6-qtbase-devel openssl-devel
```

## Security Considerations

1. **Network trust**: Assumes trusted network environment
2. **File writes**: Downloads only to user-specified directory
3. **Path sanitization**: Handled by engine, prevents directory traversal
4. **No privilege escalation**: Runs as normal user
5. **Config permissions**: Standard user-only file permissions
