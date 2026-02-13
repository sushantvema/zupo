# Commands reference

## Global flags

These flags are available on all commands:

| Flag | Description |
|---|---|
| `--api-key <KEY>` | Google Places API key (overrides env var) |
| `--json` | Output raw JSON instead of formatted text |
| `--no-color` | Disable colored output |
| `--timeout <SECS>` | HTTP timeout in seconds (default: 10) |
| `--auto-locate` | Fall back to IP-based geolocation if no coordinates provided |
| `--base-url <URL>` | Override the Places API base URL |
| `--routes-base-url <URL>` | Override the Routes API base URL |

---

## search

Search for places by text query.

```bash
zupo search -q "coffee shops in Vienna" -l 5
zupo search -q "pizza" --lat 40.7128 --lng=-74.0060 --radius 1000 --open-now
zupo search -q "sushi" --min-rating 4.5 --price-level 2,3
zupo search -q "museum" --included-type museum --lang de --region AT
```

| Flag | Description |
|---|---|
| `-q, --query <TEXT>` | Search query **(required)** |
| `--included-type <TYPE>` | Filter by place type (e.g. `restaurant`, `cafe`, `museum`) |
| `--min-rating <FLOAT>` | Minimum rating, 0.0–5.0 |
| `--price-level <LEVELS>` | Price level filter: 0=Free, 1=$, 2=$$, 3=$$$, 4=$$$$ |
| `--open-now` | Only return places that are currently open |
| `--lat <FLOAT>` | Latitude for location bias |
| `--lng <FLOAT>` | Longitude for location bias |
| `--radius <METERS>` | Radius in meters for location bias |
| `-l, --limit <N>` | Maximum results, 1–20 (default: 10) |
| `--lang <CODE>` | BCP-47 language code (e.g. `en`, `de`, `ja`) |
| `--region <CODE>` | CLDR region code (e.g. `US`, `AT`, `JP`) |

---

## autocomplete

Get type-ahead suggestions as a user types.

```bash
zupo autocomplete --input "best ramen in"
zupo autocomplete --input "coffee" --lat 48.2082 --lng 16.3738 --radius 5000
zupo autocomplete --input "pizza" --session-token my-session-123
```

| Flag | Description |
|---|---|
| `-i, --input <TEXT>` | Input text for autocomplete **(required)** |
| `--session-token <TOKEN>` | Session token for billing optimization |
| `--lat <FLOAT>` | Latitude for location bias |
| `--lng <FLOAT>` | Longitude for location bias |
| `--radius <METERS>` | Radius for location bias |
| `-l, --limit <N>` | Maximum suggestions (default: 5) |
| `--lang <CODE>` | BCP-47 language code |
| `--region <CODE>` | CLDR region code |

---

## nearby

Search for places near a specific location. Requires coordinates (via flags, config, or `--auto-locate`).

```bash
zupo nearby --lat 37.7749 --lng=-122.4194 --radius 500
zupo nearby --lat 37.7749 --lng=-122.4194 --include-type restaurant --exclude-type fast_food
zupo nearby --auto-locate --include-type cafe -l 5
```

| Flag | Description |
|---|---|
| `--lat <FLOAT>` | Latitude **(required — via flag, config, or auto-locate)** |
| `--lng <FLOAT>` | Longitude **(required — via flag, config, or auto-locate)** |
| `--radius <METERS>` | Search radius |
| `--include-type <TYPES>` | Only include these place types |
| `--exclude-type <TYPES>` | Exclude these place types |
| `-l, --limit <N>` | Maximum results, 1–20 (default: 10) |
| `--lang <CODE>` | BCP-47 language code |
| `--region <CODE>` | CLDR region code |

---

## route

Search for places along a route between two locations. Uses the Routes API to compute a route, samples waypoints along it, then searches near each waypoint.

```bash
zupo route -q "gas station" --from "San Francisco" --to "Los Angeles"
zupo route -q "rest stop" --from "NYC" --to "Boston" --mode DRIVE --max-waypoints 8
zupo route -q "cafe" --from "Portland" --to "Seattle" --mode BICYCLE --radius 500
```

| Flag | Description |
|---|---|
| `-q, --query <TEXT>` | What to search for along the route **(required)** |
| `--from <ADDRESS>` | Origin address or place name **(required)** |
| `--to <ADDRESS>` | Destination address or place name **(required)** |
| `--mode <MODE>` | Travel mode: `DRIVE`, `WALK`, `BICYCLE`, `TWO_WHEELER`, `TRANSIT` (default: `DRIVE`) |
| `--radius <METERS>` | Search radius around each waypoint (default: 1000) |
| `--max-waypoints <N>` | Number of waypoints to sample along route (default: 5) |
| `-l, --limit <N>` | Max results per waypoint (default: 5) |
| `--lang <CODE>` | BCP-47 language code |
| `--region <CODE>` | CLDR region code |

---

## details

Get full details for a specific place by its place ID.

```bash
zupo details --place-id ChIJ84iU6DOBhYARHXonh3NuCNo
zupo details --place-id ChIJ84iU6DOBhYARHXonh3NuCNo --reviews --photos
zupo details --place-id ChIJ84iU6DOBhYARHXonh3NuCNo --show-photos
```

| Flag | Description |
|---|---|
| `--place-id <ID>` | Place ID from search results **(required)** |
| `--reviews` | Include reviews in response |
| `--photos` | Include photo metadata in response |
| `--show-photos` | Download and display photos inline in terminal |
| `--lang <CODE>` | BCP-47 language code |
| `--region <CODE>` | CLDR region code |

---

## photo

Get a photo URL or display a photo inline. Uses the photo resource name from a `details --photos` response.

```bash
zupo photo --name "places/ChIJ.../photos/AUc7..." --max-width 800
zupo photo --name "places/ChIJ.../photos/AUc7..." --show
```

| Flag | Description |
|---|---|
| `--name <NAME>` | Photo resource name **(required)** |
| `--max-width <PX>` | Maximum width in pixels |
| `--max-height <PX>` | Maximum height in pixels |
| `--show` | Display the photo inline in terminal |

---

## resolve

Resolve an address or location name to place candidates.

```bash
zupo resolve -l "1600 Amphitheatre Parkway"
zupo resolve -l "Eiffel Tower" --limit 3
```

| Flag | Description |
|---|---|
| `-l, --location <TEXT>` | Location text to resolve **(required)** |
| `--limit <N>` | Maximum results (default: 5) |
| `--lang <CODE>` | BCP-47 language code |
| `--region <CODE>` | CLDR region code |

---

## config

Manage the zupo configuration file (`~/.config/zupo/config.toml`).

### config set-location

```bash
zupo config set-location --lat 37.7749 --lng=-122.4194
zupo config set-location --lat 48.2082 --lng 16.3738 --radius 10000 --label "Vienna"
```

| Flag | Description |
|---|---|
| `--lat <FLOAT>` | Latitude **(required)** |
| `--lng <FLOAT>` | Longitude **(required)** |
| `--radius <METERS>` | Default search radius |
| `--label <NAME>` | Human-readable label for this location |

### config show

```bash
zupo config show
```

### config auto-detect

Detect your location via IP geolocation and save it as the default.

```bash
zupo config auto-detect
```

### config clear-location

```bash
zupo config clear-location
```
