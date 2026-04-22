# LocalAirtraffic TUI

A terminal UI that shows live ADS-B air traffic within a configurable radius of your location — sorted by distance, color-coded by proximity, auto-refreshing every 30 minutes.

## Install

### macOS / Linux

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Brasilius/LocalAirtrafficTUI/releases/latest/download/LocalAirtrafficTUI-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Brasilius/LocalAirtrafficTUI/releases/latest/download/LocalAirtrafficTUI-installer.ps1 | iex"
```

### Or download a pre-built binary directly

Grab the archive for your platform from the [Releases page](https://github.com/Brasilius/LocalAirtrafficTUI/releases) and put the binary on your `PATH`.

| Platform | Archive |
|----------|---------|
| macOS (Apple Silicon) | `LocalAirtrafficTUI-aarch64-apple-darwin.tar.xz` |
| macOS (Intel) | `LocalAirtrafficTUI-x86_64-apple-darwin.tar.xz` |
| Linux x86_64 | `LocalAirtrafficTUI-x86_64-unknown-linux-gnu.tar.xz` |
| Linux ARM64 | `LocalAirtrafficTUI-aarch64-unknown-linux-gnu.tar.xz` |
| Windows x86_64 | `LocalAirtrafficTUI-x86_64-pc-windows-msvc.zip` |

## Features

- Detects your position automatically via IP geolocation (or pass `--lat`/`--lon` manually)
- Queries the [OpenSky Network](https://opensky-network.org) for real-time aircraft positions
- Resolves aircraft type and registration from [hexdb.io](https://hexdb.io)
- Displays a live table: distance · callsign · registration · type · altitude · speed · heading · vertical speed
- Color-coded rows: bright green < 3 mi · yellow 3–6 mi · white 6–10 mi · dim gray = on ground
- No API keys required — all data sources are free and public

## Usage

```
LocalAirtrafficTUI [--lat <deg>] [--lon <deg>] [--radius <miles>]
```

```bash
# Auto-detect location via IP
LocalAirtrafficTUI

# Specify location and radius manually
LocalAirtrafficTUI --lat 51.5074 --lon -0.1278 --radius 15
```

### Key bindings

| Key | Action |
|-----|--------|
| `q` / `Esc` / `Ctrl-C` | Quit |
| `r` / `R` / `F5` | Force refresh |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |

### Optional: OpenSky credentials

The OpenSky Network API is rate-limited for anonymous requests. If you have a free account, set these environment variables to get higher request limits:

```bash
export OPENSKY_USER=your_username
export OPENSKY_PASS=your_password
```

On Windows:

```powershell
$env:OPENSKY_USER="your_username"
$env:OPENSKY_PASS="your_password"
```

## Data sources

| Source | Purpose |
|--------|---------|
| [ip-api.com](http://ip-api.com) | IP-based geolocation on startup |
| [OpenSky Network](https://opensky-network.org/api) | Live ADS-B state vectors (bounding box query) |
| [hexdb.io](https://hexdb.io/api/v1/aircraft/) | Aircraft type & registration lookup by ICAO24 hex |

## Building from source

Requires Rust 1.85+ (uses edition 2024). No system libraries needed — TLS is handled by the bundled `rustls` backend.

```bash
cargo build --release
```
