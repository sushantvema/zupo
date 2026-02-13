mod api;
mod render;

use std::process;
use std::time::Duration;

use clap::{Parser, Subcommand};

use api::client::Client;
use api::types::*;

#[derive(Parser)]
#[command(
    name = "zupo",
    about = "A Rust CLI for Google Places API (New)",
    version,
    after_help = "Environment:\n  GOOGLE_PLACES_API_KEY    API key for Google Places (required)"
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
        #[arg(long, requires = "lng")]
        lat: Option<f64>,

        /// Longitude for location bias
        #[arg(long, requires = "lat")]
        lng: Option<f64>,

        /// Radius in meters for location bias
        #[arg(long, default_value = "5000")]
        radius: f64,

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
        #[arg(long, requires = "lng")]
        lat: Option<f64>,

        /// Longitude for location bias
        #[arg(long, requires = "lat")]
        lng: Option<f64>,

        /// Radius in meters for location bias
        #[arg(long, default_value = "5000")]
        radius: f64,

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
        /// Latitude (required)
        #[arg(long)]
        lat: f64,

        /// Longitude (required)
        #[arg(long)]
        lng: f64,

        /// Search radius in meters (required)
        #[arg(long)]
        radius: f64,

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
}

#[tokio::main]
async fn main() {
    // Load .env file if present (ignore errors)
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
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

    let result = run_command(&client, &cli.command, cli.json).await;
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        match e {
            api::errors::Error::Validation { .. } => process::exit(2),
            _ => process::exit(1),
        }
    }
}

async fn run_command(
    client: &Client,
    command: &Commands,
    json_output: bool,
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
            let location = lat.zip(*lng).map(|(la, ln)| Circle {
                center: LatLng {
                    latitude: la,
                    longitude: ln,
                },
                radius: *radius,
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
            let location = lat.zip(*lng).map(|(la, ln)| Circle {
                center: LatLng {
                    latitude: la,
                    longitude: ln,
                },
                radius: *radius,
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
            let req = NearbySearchRequest {
                lat: *lat,
                lng: *lng,
                radius: *radius,
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
