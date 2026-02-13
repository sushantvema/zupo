# zupo

A fast, feature-rich CLI for the [Google Places API (New)](https://developers.google.com/maps/documentation/places/web-service/op-overview) written in Rust.

Search for places, get autocomplete suggestions, explore nearby spots, find stops along a route, view place details with inline photos — all from your terminal.

## Quick start

```bash
# Set your API key
export GOOGLE_PLACES_API_KEY="your-key-here"

# Search for places
zupo search -q "ramen near SoMa San Francisco" -l 5

# Get autocomplete suggestions
zupo autocomplete --input "best coffee in"

# Explore nearby
zupo nearby --lat 37.7749 --lng=-122.4194 --include-type restaurant --radius 500

# Place details with photos
zupo details --place-id ChIJ84iU6DOBhYARHXonh3NuCNo --show-photos

# Find places along a route
zupo route -q "gas station" --from "San Francisco" --to "Los Angeles" --mode DRIVE
```

## Installation

### From source

```bash
git clone https://github.com/sushantvema/zupo.git
cd zupo
cargo install --path .
```

### Prerequisites

- Rust 1.70+
- A [Google Places API key](https://developers.google.com/maps/documentation/places/web-service/get-api-key) with the Places API (New) enabled
- OpenSSL development libraries (for native TLS)
  - Ubuntu/Debian: `apt install libssl-dev pkg-config`
  - macOS: included with Xcode Command Line Tools
  - Fedora: `dnf install openssl-devel`

## Configuration

zupo resolves the API key and location through a priority chain:

**API key:** `--api-key` flag > `GOOGLE_PLACES_API_KEY` env var > `.env` file

**Location** (for commands that accept `--lat`/`--lng`):
1. Explicit `--lat`/`--lng` flags (highest priority)
2. Saved default from config file
3. IP-based geolocation via `--auto-locate`

### Config file

Stored at `~/.config/zupo/config.toml`:

```bash
# Save a default location
zupo config set-location --lat 37.7749 --lng=-122.4194 --label "SF Office"

# Auto-detect location from IP and save it
zupo config auto-detect

# View current config
zupo config show

# Clear saved location
zupo config clear-location
```

## Commands

| Command | Description |
|---|---|
| `search` | Text search for places |
| `autocomplete` | Get type-ahead suggestions |
| `nearby` | Search near a location |
| `route` | Find places along a driving/walking route |
| `details` | Full details for a place (hours, reviews, photos) |
| `photo` | Get a photo URL or display it inline |
| `resolve` | Resolve an address to place candidates |
| `config` | Manage saved configuration |

Run `zupo <command> --help` for full flag reference.

## Output modes

```bash
# Colored terminal output (default)
zupo search -q "tacos"

# JSON output for piping/scripting
zupo search -q "tacos" --json

# No color (for logging or piped output)
zupo search -q "tacos" --no-color
```

## Documentation

See the [docs/](docs/) directory:

- [Commands reference](docs/commands.md) — detailed flags and examples for every command
- [Configuration guide](docs/configuration.md) — config file, environment variables, location resolution
- [Architecture](docs/architecture.md) — project structure and design decisions

## License

MIT
