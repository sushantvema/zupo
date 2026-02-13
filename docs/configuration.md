# Configuration guide

## API key

zupo requires a Google Places API key. The key is resolved in this order:

1. `--api-key` flag (highest priority)
2. `GOOGLE_PLACES_API_KEY` environment variable
3. `.env` file in the current directory (loaded automatically via dotenvy)

### Getting a key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a project (or select an existing one)
3. Enable the **Places API (New)** and **Routes API**
4. Create an API key under **APIs & Services > Credentials**
5. Restrict the key to the Places and Routes APIs

### Setting the key

```bash
# Option 1: Environment variable
export GOOGLE_PLACES_API_KEY="AIza..."

# Option 2: .env file in your project directory
echo 'GOOGLE_PLACES_API_KEY=AIza...' > .env

# Option 3: Per-command flag
zupo search -q "coffee" --api-key "AIza..."
```

## Location resolution

Commands that accept `--lat`/`--lng` resolve location through a three-tier priority chain:

### 1. Explicit flags (highest priority)

```bash
zupo search -q "pizza" --lat 40.7128 --lng=-74.0060 --radius 2000
```

**Note:** Negative longitude values require the `--lng=-122.4194` syntax (with `=`) because clap interprets `--lng -122` as a separate flag.

### 2. Saved config

```bash
# Save once
zupo config set-location --lat 37.7749 --lng=-122.4194 --label "SF"

# All future commands use this as the default
zupo search -q "coffee"
zupo nearby --include-type cafe
```

### 3. IP-based geolocation

```bash
# One-time: auto-detect and use for this command
zupo search -q "coffee" --auto-locate

# Persistent: detect and save as default
zupo config auto-detect
```

IP geolocation uses the [ip-api.com](http://ip-api.com) free tier (no key required). Accuracy varies by ISP and network.

## Config file

Location: `~/.config/zupo/config.toml`

```toml
[location]
default_lat = 37.7749
default_lng = -122.4194
default_radius = 5000.0
label = "SF Office"
```

### Fields

| Field | Type | Description |
|---|---|---|
| `default_lat` | float | Default latitude (-90 to 90) |
| `default_lng` | float | Default longitude (-180 to 180) |
| `default_radius` | float | Default search radius in meters (default: 1000) |
| `label` | string | Human-readable label for the location |

### Managing config

```bash
zupo config set-location --lat 48.2082 --lng 16.3738 --radius 10000 --label "Vienna"
zupo config show
zupo config auto-detect
zupo config clear-location
```

## Environment variables

| Variable | Description |
|---|---|
| `GOOGLE_PLACES_API_KEY` | API key for Google Places (required) |
| `NO_COLOR` | Set to any value to disable colored output (standard) |

## Base URL overrides

For testing or proxying, you can override API base URLs:

```bash
zupo search -q "test" --base-url "http://localhost:8080/v1"
zupo route -q "gas" --from A --to B --routes-base-url "http://localhost:8081"
```
