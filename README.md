# LocalAirtraffic TUI

A terminal UI that shows live ADS-B air traffic within a configurable radius of your location — sorted by distance, color-coded by proximity, auto-refreshing every 15 seconds.

## Features

- Detects your position automatically via IP geolocation (or pass `--lat`/`--lon` manually)
- Queries the [OpenSky Network](https://opensky-network.org) for real-time aircraft positions
- Resolves aircraft type and registration from [hexdb.io](https://hexdb.io)
- Displays a live table: distance · callsign · registration · type · altitude · speed · heading · vertical speed
- Color-coded rows: bright green < 3 mi · yellow 3–6 mi · white 6–10 mi · dim gray = on ground
- No API keys required — all data sources are free and public

## Usage

```bash
# Auto-detect location via IP
cargo run --release

# Specify location and radius manually
cargo run --release -- --lat 51.5074 --lon -0.1278 --radius 15
```

### Key bindings

| Key | Action |
|-----|--------|
| `q` / `Esc` / `Ctrl-C` | Quit |
| `r` / `R` / `F5` | Force refresh |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |

## Data sources

| Source | Purpose |
|--------|---------|
| [ip-api.com](http://ip-api.com) | IP-based geolocation on startup |
| [OpenSky Network](https://opensky-network.org/api) | Live ADS-B state vectors (bounding box query) |
| [hexdb.io](https://hexdb.io/api/v1/aircraft/) | Aircraft type & registration lookup by ICAO24 hex |

## Building

Requires Rust 1.80+ (uses edition 2024). No system libraries needed — TLS is handled by the bundled `rustls` backend.

```bash
cargo build --release
```
