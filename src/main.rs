mod api;
mod config;
mod geolocate;
mod render;

use std::process;
use std::time::Duration;

use clap::{Parser, Subcommand};
use colored::Colorize;

use api::client::Client;
use api::types::*;
use config::Config;

#[derive(Parser)]
#[command(
    name = "zupo",
    about = "A Rust CLI for Google Places API (New)",
    version,
    after_help = "Environment:\n  GOOGLE_PLACES_API_KEY    API key for Google Places (required)\n\n\
    Location resolution (for commands that use --lat/--lng):\n  \
    1. Explicit --lat/--lng flags (highest priority)\n  \
    2. Default location from config (~/.config/zupo/config.toml)\n  \
    3. IP-based geolocation via --auto-locate flag"
)]
struct Cli {
    /// Google Places API key (or set GOOGLE_PLACES_API_KEY)
    #[arg(long, env = "GOOGLE_PLACES_API_KEY", hide_env_values = true, global = true)]
    api_key: Option<String>,

    /// Output as JSON instead of colored text
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// HTTP timeout in seconds
    #[arg(long, default_value = "10", global = true)]
    timeout: u64,

    /// Auto-detect location via IP geolocation (fallback if no --lat/--lng or config)
    #[arg(long, global = true)]
    auto_locate: bool,

    /// Override Places API base URL
    #[arg(long, global = true)]
    base_url: Option<String>,

    /// Override Routes API base URL
    #[arg(long, global = true)]
    routes_base_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for places by text query
    Search {
        /// Search query (e.g., "coffee shops in Vienna")
        #[arg(short, long)]
        query: String,

        /// Filter by place type (e.g., restaurant, cafe)
        #[arg(long, name = "type")]
        included_type: Option<String>,

        /// Minimum rating (0.0-5.0)
        #[arg(long)]
        min_rating: Option<f64>,

        /// Price level filter (0=Free, 1=$, 2=$$, 3=$$$, 4=$$$$)
        #[arg(long, value_delimiter = ',')]
        price_level: Vec<u8>,

        /// Only return places that are currently open
        #[arg(long)]
        open_now: bool,

        /// Latitude for location bias
        #[arg(long)]
        lat: Option<f64>,

        /// Longitude for location bias
        #[arg(long)]
        lng: Option<f64>,

        /// Radius in meters for location bias
        #[arg(long)]
        radius: Option<f64>,

        /// Maximum number of results (1-20)
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// BCP-47 language code (e.g., en, de, ja)
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code (e.g., US, AT, JP)
        #[arg(long)]
        region: Option<String>,
    },

    /// Get autocomplete suggestions
    Autocomplete {
        /// Input text for autocomplete
        #[arg(short, long)]
        input: String,

        /// Session token for billing optimization
        #[arg(long)]
        session_token: Option<String>,

        /// Latitude for location bias
        #[arg(long)]
        lat: Option<f64>,

        /// Longitude for location bias
        #[arg(long)]
        lng: Option<f64>,

        /// Radius in meters for location bias
        #[arg(long)]
        radius: Option<f64>,

        /// Maximum number of suggestions
        #[arg(short, long, default_value = "5")]
        limit: u32,

        /// BCP-47 language code
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code
        #[arg(long)]
        region: Option<String>,
    },

    /// Search for places near a location
    Nearby {
        /// Latitude (uses config/auto-locate if omitted)
        #[arg(long)]
        lat: Option<f64>,

        /// Longitude (uses config/auto-locate if omitted)
        #[arg(long)]
        lng: Option<f64>,

        /// Search radius in meters
        #[arg(long)]
        radius: Option<f64>,

        /// Include only these place types
        #[arg(long = "include-type", value_delimiter = ',')]
        include_types: Vec<String>,

        /// Exclude these place types
        #[arg(long = "exclude-type", value_delimiter = ',')]
        exclude_types: Vec<String>,

        /// Maximum number of results (1-20)
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// BCP-47 language code
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code
        #[arg(long)]
        region: Option<String>,
    },

    /// Search for places along a route
    Route {
        /// What to search for along the route
        #[arg(short, long)]
        query: String,

        /// Origin address or place name
        #[arg(long)]
        from: String,

        /// Destination address or place name
        #[arg(long)]
        to: String,

        /// Travel mode: DRIVE, WALK, BICYCLE, TWO_WHEELER, TRANSIT
        #[arg(long, default_value = "DRIVE")]
        mode: String,

        /// Search radius around each waypoint in meters
        #[arg(long, default_value = "1000")]
        radius: f64,

        /// Number of waypoints to sample along the route
        #[arg(long, default_value = "5")]
        max_waypoints: u32,

        /// Maximum results per waypoint
        #[arg(short, long, default_value = "5")]
        limit: u32,

        /// BCP-47 language code
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code
        #[arg(long)]
        region: Option<String>,
    },

    /// Get detailed information about a place
    Details {
        /// Place ID (from search results)
        #[arg(long)]
        place_id: String,

        /// Include reviews in the response
        #[arg(long)]
        reviews: bool,

        /// Include photos in the response
        #[arg(long)]
        photos: bool,

        /// Download and display photos inline in the terminal
        #[arg(long)]
        show_photos: bool,

        /// BCP-47 language code
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code
        #[arg(long)]
        region: Option<String>,
    },

    /// Get a photo URL for a place photo
    Photo {
        /// Photo resource name (from details response)
        #[arg(long)]
        name: String,

        /// Maximum width in pixels
        #[arg(long)]
        max_width: Option<u32>,

        /// Maximum height in pixels
        #[arg(long)]
        max_height: Option<u32>,

        /// Display the photo inline in the terminal
        #[arg(long)]
        show: bool,
    },

    /// Resolve an address or location name to place candidates
    Resolve {
        /// Location text to resolve (address, place name, etc.)
        #[arg(short, long)]
        location: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "5")]
        limit: u32,

        /// BCP-47 language code
        #[arg(long)]
        lang: Option<String>,

        /// CLDR region code
        #[arg(long)]
        region: Option<String>,
    },

    /// Manage zupo configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set your default location
    SetLocation {
        /// Latitude
        #[arg(long)]
        lat: f64,

        /// Longitude
        #[arg(long)]
        lng: f64,

        /// Default search radius in meters
        #[arg(long)]
        radius: Option<f64>,

        /// Label for this location (e.g., "SoMa Office")
        #[arg(long)]
        label: Option<String>,
    },

    /// Show current configuration
    Show,

    /// Detect location via IP and save as default
    AutoDetect,

    /// Clear saved location
    ClearLocation,
}

#[tokio::main]
async fn main() {
    // Load .env file if present (ignore errors)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    // Handle config commands first (don't need API key)
    if let Commands::Config { ref action } = cli.command {
        handle_config_command(action).await;
        return;
    }

    let api_key = match cli.api_key {
        Some(ref key) => key.clone(),
        None => {
            eprintln!("Error: missing API key. Set GOOGLE_PLACES_API_KEY or use --api-key");
            process::exit(2);
        }
    };

    let mut client = match Client::new(api_key) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(2);
        }
    };

    client = client.with_timeout(Duration::from_secs(cli.timeout));

    if let Some(url) = cli.base_url {
        client = client.with_places_base_url(url);
    }
    if let Some(url) = cli.routes_base_url {
        client = client.with_routes_base_url(url);
    }

    let cfg = Config::load();
    let result = run_command(&client, &cli.command, cli.json, cli.auto_locate, &cfg).await;
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        match e {
            api::errors::Error::Validation { .. } => process::exit(2),
            _ => process::exit(1),
        }
    }
}

/// Resolve lat/lng from: explicit flags > config > auto-locate
async fn resolve_location(
    explicit_lat: Option<f64>,
    explicit_lng: Option<f64>,
    auto_locate: bool,
    cfg: &Config,
) -> Option<(f64, f64)> {
    // 1. Explicit flags
    if let (Some(lat), Some(lng)) = (explicit_lat, explicit_lng) {
        return Some((lat, lng));
    }

    // 2. Config default
    if let Some((lat, lng)) = cfg.default_location() {
        let label = cfg.location.label.as_deref().unwrap_or("config");
        eprintln!(
            "{}",
            format!("Using saved location ({}) [{:.4}, {:.4}]", label, lat, lng).dimmed()
        );
        return Some((lat, lng));
    }

    // 3. IP-based auto-locate
    if auto_locate {
        eprintln!("{}", "Auto-detecting location via IP...".dimmed());
        match geolocate::geolocate_by_ip().await {
            Ok(geo) => {
                eprintln!(
                    "{}",
                    format!(
                        "Detected: {} [{:.4}, {:.4}]",
                        geo.description, geo.lat, geo.lng
                    )
                    .dimmed()
                );
                return Some((geo.lat, geo.lng));
            }
            Err(e) => {
                eprintln!("{}", format!("Auto-locate failed: {}", e).yellow());
            }
        }
    }

    None
}

/// Resolve radius from: explicit flag > config default > fallback
fn resolve_radius(explicit: Option<f64>, cfg: &Config, fallback: f64) -> f64 {
    explicit.unwrap_or_else(|| {
        cfg.location
            .default_radius
            .unwrap_or(fallback)
    })
}

async fn handle_config_command(action: &ConfigAction) {
    match action {
        ConfigAction::SetLocation {
            lat,
            lng,
            radius,
            label,
        } => {
            let mut cfg = Config::load();
            cfg.set_location(*lat, *lng, *radius, label.clone());
            match cfg.save() {
                Ok(()) => {
                    println!("Location saved to {}", config::config_file_path());
                    println!("  Lat: {}", lat);
                    println!("  Lng: {}", lng);
                    if let Some(r) = radius {
                        println!("  Radius: {}m", r);
                    }
                    if let Some(ref l) = label {
                        println!("  Label: {}", l);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }

        ConfigAction::Show => {
            let cfg = Config::load();
            let path = config::config_file_path();
            println!("{} {}", "Config file:".bold(), path);
            println!();
            if let Some((lat, lng)) = cfg.default_location() {
                println!("  {}", "Default Location".bold());
                if let Some(ref label) = cfg.location.label {
                    println!("    {} {}", "Label:".dimmed(), label);
                }
                println!("    {} {}", "Lat:".dimmed(), lat);
                println!("    {} {}", "Lng:".dimmed(), lng);
                println!(
                    "    {} {}m",
                    "Radius:".dimmed(),
                    cfg.default_radius()
                );
            } else {
                println!(
                    "  {}",
                    "No default location set. Use `zupo config set-location` or `zupo config auto-detect`.".dimmed()
                );
            }
        }

        ConfigAction::AutoDetect => {
            eprintln!("Detecting location via IP...");
            match geolocate::geolocate_by_ip().await {
                Ok(geo) => {
                    let mut cfg = Config::load();
                    cfg.set_location(
                        geo.lat,
                        geo.lng,
                        Some(5000.0),
                        Some(geo.description.clone()),
                    );
                    match cfg.save() {
                        Ok(()) => {
                            println!("Location auto-detected and saved:");
                            println!("  {} {}", "Location:".bold(), geo.description);
                            println!("  {} {}", "Lat:".dimmed(), geo.lat);
                            println!("  {} {}", "Lng:".dimmed(), geo.lng);
                            println!(
                                "  {}",
                                "Note: IP geolocation is approximate. Use `zupo config set-location` for exact coords."
                                    .yellow()
                            );
                        }
                        Err(e) => {
                            eprintln!("Error saving config: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }

        ConfigAction::ClearLocation => {
            let mut cfg = Config::load();
            cfg.clear_location();
            match cfg.save() {
                Ok(()) => println!("Default location cleared."),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}

async fn run_command(
    client: &Client,
    command: &Commands,
    json_output: bool,
    auto_locate: bool,
    cfg: &Config,
) -> Result<(), api::errors::Error> {
    match command {
        Commands::Search {
            query,
            included_type,
            min_rating,
            price_level,
            open_now,
            lat,
            lng,
            radius,
            limit,
            lang,
            region,
        } => {
            let resolved = resolve_location(*lat, *lng, auto_locate, cfg).await;
            let location = resolved.map(|(la, ln)| Circle {
                center: LatLng {
                    latitude: la,
                    longitude: ln,
                },
                radius: resolve_radius(*radius, cfg, 5000.0),
            });

            let price_levels: Vec<String> = price_level
                .iter()
                .filter_map(|&p| price_level_to_api(p).map(String::from))
                .collect();

            let req = SearchRequest {
                query: query.clone(),
                included_type: included_type.clone(),
                min_rating: *min_rating,
                price_levels,
                open_now: *open_now,
                location,
                limit: Some(*limit),
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.search(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                render::render_places(&resp.places, "Search Results");
            }
        }

        Commands::Autocomplete {
            input,
            session_token,
            lat,
            lng,
            radius,
            limit,
            lang,
            region,
        } => {
            let resolved = resolve_location(*lat, *lng, auto_locate, cfg).await;
            let location = resolved.map(|(la, ln)| Circle {
                center: LatLng {
                    latitude: la,
                    longitude: ln,
                },
                radius: resolve_radius(*radius, cfg, 5000.0),
            });

            let req = AutocompleteRequest {
                input: input.clone(),
                session_token: session_token.clone(),
                location,
                limit: Some(*limit),
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.autocomplete(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                render::render_autocomplete(&resp);
            }
        }

        Commands::Nearby {
            lat,
            lng,
            radius,
            include_types,
            exclude_types,
            limit,
            lang,
            region,
        } => {
            let resolved = resolve_location(*lat, *lng, auto_locate, cfg).await;
            let (rlat, rlng) = resolved.ok_or_else(|| api::errors::Error::Validation {
                field: "lat/lng".into(),
                message: "location required: use --lat/--lng, set a default with `zupo config set-location`, or use --auto-locate".into(),
            })?;
            let rradius = resolve_radius(*radius, cfg, 1000.0);

            let req = NearbySearchRequest {
                lat: rlat,
                lng: rlng,
                radius: rradius,
                included_types: include_types.clone(),
                excluded_types: exclude_types.clone(),
                limit: Some(*limit),
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.nearby_search(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                render::render_places(&resp.places, "Nearby Places");
            }
        }

        Commands::Route {
            query,
            from,
            to,
            mode,
            radius,
            max_waypoints,
            limit,
            lang,
            region,
        } => {
            let travel_mode: TravelMode = mode.parse().map_err(|msg: String| {
                api::errors::Error::Validation {
                    field: "mode".into(),
                    message: msg,
                }
            })?;

            let req = RouteRequest {
                query: query.clone(),
                from: from.clone(),
                to: to.clone(),
                travel_mode,
                search_radius: *radius,
                max_waypoints: *max_waypoints,
                results_per_waypoint: *limit,
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.route_search(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                render::render_route(&resp);
            }
        }

        Commands::Details {
            place_id,
            reviews,
            photos,
            show_photos,
            lang,
            region,
        } => {
            let include_photos = *photos || *show_photos;
            let req = DetailsRequest {
                place_id: place_id.clone(),
                include_reviews: *reviews,
                include_photos,
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.details(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                let photo_images = if *show_photos {
                    fetch_place_photo_images(client, &resp).await
                } else {
                    None
                };
                render::render_place_details(&resp, photo_images.as_deref());
            }
        }

        Commands::Photo {
            name,
            max_width,
            max_height,
            show,
        } => {
            let req = PhotoMediaRequest {
                name: name.clone(),
                max_width: *max_width,
                max_height: *max_height,
            };

            let resp = client.photo_media(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                let image_bytes = if *show && !resp.photo_uri.is_empty() {
                    match client.download_bytes(&resp.photo_uri).await {
                        Ok(bytes) => Some(bytes),
                        Err(e) => {
                            eprintln!("Warning: could not download photo: {}", e);
                            None
                        }
                    }
                } else {
                    None
                };
                render::render_photo(&resp, image_bytes.as_deref());
            }
        }

        Commands::Resolve {
            location,
            limit,
            lang,
            region,
        } => {
            let req = ResolveRequest {
                location: location.clone(),
                limit: Some(*limit),
                language: lang.clone(),
                region: region.clone(),
            };

            let resp = client.resolve(&req).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                render::render_places(&resp.places, "Resolved Places");
            }
        }

        Commands::Config { .. } => unreachable!(),
    }

    Ok(())
}

/// Fetch up to 3 place photos as raw image bytes for inline display
async fn fetch_place_photo_images(client: &Client, place: &Place) -> Option<Vec<Vec<u8>>> {
    let photos = place.photos.as_ref()?;
    if photos.is_empty() {
        return None;
    }

    let mut images = Vec::new();
    for photo in photos.iter().take(3) {
        let req = PhotoMediaRequest {
            name: photo.name.clone(),
            max_width: Some(400),
            max_height: None,
        };

        if let Ok(resp) = client.photo_media(&req).await {
            if !resp.photo_uri.is_empty() {
                if let Ok(bytes) = client.download_bytes(&resp.photo_uri).await {
                    images.push(bytes);
                }
            }
        }
    }

    if images.is_empty() {
        None
    } else {
        Some(images)
    }
}
