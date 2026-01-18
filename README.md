# Gosh Transfer

A GTK4/Libadwaita desktop application for explicit file transfers over LAN, Tailscale, and VPNs. Powered by [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer).

## Philosophy

Gosh apps are built with a Linux-first mindset: simplicity, transparency, and user control.

## What It Does

Gosh Transfer sends files between computers using explicit IP addresses or hostnames. No auto-discovery, no cloud services, no magic. You specify exactly where files go.

Send files or entire directories to any machine on your network by entering its address and picking what to transfer. On the receiving end, incoming requests appear for you to accept or reject individually, or handle all at once with batch operations. Transfers show real-time progress with speed indicators, and you can cancel them mid-flight if needed.

Save frequently-used peers as favorites for quick access, or add trusted hosts that auto-accept transfers without prompting. The interface filtering lets you choose which network types to display (WiFi, Ethernet, VPN, Docker), and receive-only mode disables sending entirely when you just want to accept files. All transfers are logged in a persistent history (up to 100 entries), and failed transfers retry automatically.

## Screenshots

![Screenshot 1](screenshots/img1.png)
![Screenshot 2](screenshots/img2.png)
![Screenshot 3](screenshots/img3.png)
![Screenshot 4](screenshots/img4.png)

## Technical Details

The frontend uses GTK4 0.9 with Libadwaita 0.7, written in Rust 2021 edition. The transfer engine comes from the [gosh-lan-transfer](https://github.com/goshitsarch-eng/gosh-lan-transfer) crate, which runs on Tokio for async operations.

### Network Protocol

The application runs an HTTP server on port 53317 by default. The protocol exposes four endpoints: `/health` for status checks, `/transfer` (POST) to initiate requests with metadata, `/transfer/status` (GET) to poll approval status, and `/chunk` (POST) to stream file data.

Security relies on UUID tokens authorizing uploads per transfer, with filename sanitization preventing path traversal attacks. The system assumes you're on a trusted network like a LAN, VPN, or Tailscale.

### Data Storage

Configuration lives in `~/.config/gosh/transfer/` on Linux (determined by the `directories` crate). Three JSON files handle persistence: `settings.json` for application settings, `favorites.json` for saved peer addresses, and `history.json` for transfer records (capped at 100 entries).

### Settings

The server runs on port 53317 by default and identifies itself to peers using your system hostname. Downloaded files land in your OS Downloads folder unless you change it. You can configure trusted hosts for auto-accept, toggle receive-only mode, enable or disable notifications, and pick your theme (system, light, or dark).

Transfer reliability settings include max retries (default 3) and retry delay (default 1000ms). Interface filters control which network types appear in the address list, with WiFi, Ethernet, VPN, and Other shown by default while Docker interfaces are hidden.

## Building

You'll need Rust 1.70 or newer along with GTK4 and Libadwaita development libraries.

**Ubuntu/Debian:**
```bash
sudo apt-get install libgtk-4-dev libadwaita-1-dev libssl-dev pkg-config
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel openssl-devel
```

For development, run `cargo run -p gosh-transfer-gtk`. Production builds use `cargo build --release -p gosh-transfer-gtk`, producing a binary at `target/release/gosh-transfer-gtk`.

## Disclaimer

This application is an independent project and is not sponsored by, endorsed by, or affiliated with Localsend or GitHub, Inc.

This software is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). It is provided "as is", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability or fitness for a particular purpose. Use at your own risk.

## License

AGPL-3.0. See [LICENSE](LICENSE).
