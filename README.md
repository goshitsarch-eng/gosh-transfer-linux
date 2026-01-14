# Gosh Transfer

A GTK4/Libadwaita desktop application for explicit file transfers over LAN, Tailscale, and VPNs.

Powered by [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer).

## Philosophy

Gosh apps are built with a Linux-first mindset: simplicity, transparency, and user control.

## What It Does

Gosh Transfer sends files between computers using explicit IP addresses or hostnames. No auto-discovery, no cloud services, no magic—you specify where files go.

## Screenshots

> **Note**: Screenshots below are from an earlier version and will be updated to show the current GTK4/Libadwaita interface.

![Screenshot 1](screenshots/img1.png)
![Screenshot 2](screenshots/img2.png)
![Screenshot 3](screenshots/img3.png)
![Screenshot 4](screenshots/img4.png)
![Screenshot 5](screenshots/img5.png)


## Features

- **Send files and directories** to a specific IP/hostname with file picker
- **Receive files** with manual accept/reject for each transfer request
- **Batch operations** to accept or reject all pending transfers at once
- **Cancel transfers** mid-progress
- **Real-time progress** with transfer speed display
- **Favorites** for saving frequently-used peer addresses
- **Trusted hosts** for auto-accepting transfers from specific IPs
- **Interface filtering** to show/hide network interface types (WiFi, Ethernet, VPN, Docker)
- **Receive-only mode** to disable sending
- **Theme support** (dark/light/system)
- **Transfer history** with persistent storage (up to 100 entries)
- **Automatic retry** on network interruptions

## Technical Details

### Architecture

| Component | Technology |
|-----------|------------|
| Frontend | GTK4 0.9 + Libadwaita 0.7 |
| Transfer Engine | [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer) |
| Async Runtime | Tokio |
| Language | Rust 2021 edition |

### Network Protocol

The application runs an HTTP server on port **53317** (default).

The transfer protocol is implemented in the [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer) crate, which provides:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Server status verification |
| `/transfer` | POST | Initiate transfer request with metadata |
| `/transfer/status` | GET | Poll approval status |
| `/chunk` | POST | Stream file data |

**Security model**: UUID tokens authorize uploads per transfer, filenames undergo path traversal sanitization. The system assumes trusted networks (LAN/VPN/Tailscale).

### Data Storage

Settings, favorites, and history are stored in `~/.config/gosh/transfer/`:

| File | Purpose |
|------|---------|
| `settings.json` | Application settings |
| `favorites.json` | Saved peer addresses |
| `history.json` | Transfer history (max 100 entries) |

### Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `port` | u16 | 53317 | Server port |
| `deviceName` | string | hostname | Shown to peers |
| `downloadDir` | path | OS Downloads folder | Where received files are saved |
| `trustedHosts` | string[] | [] | IPs for auto-accept |
| `receiveOnly` | bool | false | Hides Send functionality |
| `notificationsEnabled` | bool | true | System notifications toggle |
| `theme` | string | "system" | "dark", "light", or "system" |
| `maxRetries` | u32 | 3 | Retry attempts for failed transfers |
| `retryDelayMs` | u64 | 1000 | Delay between retry attempts |
| `interfaceFilters` | object | see below | Interface category visibility |

**Interface Filters** (all boolean, defaults in parentheses):
- `showWifi` (true), `showEthernet` (true), `showVpn` (true), `showDocker` (false), `showOther` (true)

## Building

### Prerequisites

- Rust 1.70+
- GTK4 development libraries
- Libadwaita development libraries

### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libgtk-4-dev libadwaita-1-dev libssl-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel openssl-devel
```

### Development

```bash
cargo run -p gosh-transfer-gtk
```

### Production Build

```bash
cargo build --release -p gosh-transfer-gtk
```

The binary will be at `target/release/gosh-transfer-gtk`.

## Disclaimer

This application is an independent project and is not sponsored by, endorsed by, or affiliated with Localsend or GitHub, Inc.

This software is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). It is provided "as is", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability or fitness for a particular purpose. Use at your own risk.

## License

AGPL-3.0 - See [LICENSE](LICENSE)
