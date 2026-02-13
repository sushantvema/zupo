# Architecture

## Project structure

```
zupo/
├── Cargo.toml
├── README.md
├── docs/
│   ├── commands.md          # CLI command reference
│   ├── configuration.md     # Config and environment guide
│   └── architecture.md      # This file
└── src/
    ├── main.rs              # Entry point, CLI definition, command routing
    ├── config.rs            # Config file management (~/.config/zupo/config.toml)
    ├── geolocate.rs         # IP-based geolocation via ip-api.com
    ├── render.rs            # Terminal output formatting and photo display
    └── api/
        ├── mod.rs           # Module exports
        ├── client.rs        # HTTP client (reqwest + native TLS)
        ├── errors.rs        # Error types
        ├── types.rs         # Request/response structs
        ├── search.rs        # POST /places:searchText
        ├── autocomplete.rs  # POST /places:autocomplete
        ├── nearby.rs        # POST /places:searchNearby
        ├── details.rs       # GET /places/{id}
        ├── photo.rs         # GET /{name}/media
        ├── resolve.rs       # POST /places:searchText (address resolution)
        └── route.rs         # Routes API + waypoint sampling + per-waypoint search
```

## Design decisions

### API client layer (`api/`)

Each Google Places endpoint maps to a method on `api::Client`. The client handles:

- **Authentication** via `X-Goog-Api-Key` header
- **Field masking** via `X-Goog-FieldMask` header — only requested fields are returned, which controls both response size and billing
- **Response size limit** of 1 MB to prevent memory issues
- **Configurable timeouts** (default 10s)

Request/response types in `types.rs` are shared across all endpoints. The `Place` struct is a unified type that covers search results, nearby results, and detail responses.

### Route search (`route.rs`)

The route command is a multi-step pipeline:

1. **Resolve origin/destination** — text search to get coordinates
2. **Compute route** — call the Routes API (`/directions/v2:computeRoutes`) to get an encoded polyline
3. **Decode polyline** — convert Google's encoded polyline format to lat/lng points
4. **Sample waypoints** — pick N evenly-spaced points along the route using haversine distance and interpolation
5. **Search per waypoint** — run a text search around each waypoint with a circular location bias

### Terminal rendering (`render.rs`)

Output formatting is handled separately from API logic. The renderer supports:

- Star ratings with partial-fill characters (★/⯪/☆)
- Price level display (Free / $ / $$ / $$$ / $$$$)
- Color-coded business status
- Inline photo display via `viuer` (Unicode block art)
- Structured autocomplete suggestions with type tags

### Location resolution

Location is resolved at the `main.rs` level before being passed to API methods. The three-tier chain (flags > config > auto-locate) is handled by helper functions that merge CLI args with saved config. Commands that require location (like `nearby`) error early if none is available.

### TLS

The project uses `native-tls` (OpenSSL on Linux, Secure Transport on macOS, SChannel on Windows) rather than `rustls`. This ensures compatibility with system certificate stores across environments.

### Error handling

Errors are typed in `api::errors::Error` with four variants:

- `MissingApiKey` — no API key provided
- `Validation` — invalid input (e.g., bad coordinates)
- `Api` — HTTP error from Google (includes status code and body)
- `Http` — network/TLS errors from reqwest

The main function maps these to appropriate exit codes (1 for API/network errors, 2 for validation).

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing with derive macros |
| `reqwest` | Async HTTP client |
| `tokio` | Async runtime |
| `serde` / `serde_json` | JSON serialization |
| `colored` | Terminal colors |
| `dotenvy` | `.env` file loading |
| `anyhow` | Error context in main |
| `viuer` | Inline terminal image display |
| `image` | Image decoding |
| `toml` | Config file parsing |
| `dirs` | Platform config directory paths |
