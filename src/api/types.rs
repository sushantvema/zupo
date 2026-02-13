use serde::{Deserialize, Serialize};

// ─── Common types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Circle {
    pub center: LatLng,
    pub radius: f64,
}

// ─── Place (unified response type) ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub display_name: Option<DisplayName>,
    #[serde(default)]
    pub formatted_address: Option<String>,
    #[serde(default)]
    pub short_formatted_address: Option<String>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
    #[serde(default)]
    pub primary_type: Option<String>,
    #[serde(default)]
    pub primary_type_display_name: Option<DisplayName>,
    #[serde(default)]
    pub location: Option<LatLng>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub user_rating_count: Option<u32>,
    #[serde(default)]
    pub price_level: Option<String>,
    #[serde(default)]
    pub website_uri: Option<String>,
    #[serde(default)]
    pub google_maps_uri: Option<String>,
    #[serde(default)]
    pub national_phone_number: Option<String>,
    #[serde(default)]
    pub international_phone_number: Option<String>,
    #[serde(default)]
    pub current_opening_hours: Option<OpeningHours>,
    #[serde(default)]
    pub regular_opening_hours: Option<OpeningHours>,
    #[serde(default)]
    pub business_status: Option<String>,
    #[serde(default)]
    pub editorial_summary: Option<EditorialSummary>,
    #[serde(default)]
    pub reviews: Option<Vec<Review>>,
    #[serde(default)]
    pub photos: Option<Vec<Photo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayName {
    pub text: String,
    #[serde(default)]
    pub language_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorialSummary {
    pub text: Option<String>,
    #[serde(default)]
    pub language_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHours {
    #[serde(default)]
    pub open_now: Option<bool>,
    #[serde(default)]
    pub weekday_descriptions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    #[serde(default)]
    pub author_attribution: Option<AuthorAttribution>,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub relative_publish_time_description: Option<String>,
    #[serde(default)]
    pub text: Option<LocalizedText>,
    #[serde(default)]
    pub original_text: Option<LocalizedText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAttribution {
    pub display_name: String,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub photo_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalizedText {
    pub text: String,
    #[serde(default)]
    pub language_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Photo {
    pub name: String,
    #[serde(default)]
    pub width_px: Option<u32>,
    #[serde(default)]
    pub height_px: Option<u32>,
    #[serde(default)]
    pub author_attributions: Option<Vec<AuthorAttribution>>,
}

// ─── Search ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub included_type: Option<String>,
    pub min_rating: Option<f64>,
    pub price_levels: Vec<String>,
    pub open_now: bool,
    pub location: Option<Circle>,
    pub limit: Option<u32>,
    pub language: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    #[serde(default)]
    pub places: Vec<Place>,
}

// ─── Autocomplete ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutocompleteRequest {
    pub input: String,
    pub session_token: Option<String>,
    pub location: Option<Circle>,
    pub limit: Option<u32>,
    pub language: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteResponse {
    #[serde(default)]
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    #[serde(default)]
    pub place_prediction: Option<PlacePrediction>,
    #[serde(default)]
    pub query_prediction: Option<QueryPrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacePrediction {
    pub place: Option<String>,
    pub place_id: Option<String>,
    pub text: Option<FormattedText>,
    pub structured_format: Option<StructuredFormat>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryPrediction {
    pub text: Option<FormattedText>,
    pub structured_format: Option<StructuredFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattedText {
    pub text: String,
    #[serde(default)]
    pub matches: Option<Vec<TextMatch>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextMatch {
    #[serde(default)]
    pub start_offset: Option<u32>,
    pub end_offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredFormat {
    pub main_text: Option<FormattedText>,
    pub secondary_text: Option<FormattedText>,
}

// ─── Nearby Search ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NearbySearchRequest {
    pub lat: f64,
    pub lng: f64,
    pub radius: f64,
    pub included_types: Vec<String>,
    pub excluded_types: Vec<String>,
    pub limit: Option<u32>,
    pub language: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearbySearchResponse {
    #[serde(default)]
    pub places: Vec<Place>,
}

// ─── Place Details ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DetailsRequest {
    pub place_id: String,
    pub include_reviews: bool,
    pub include_photos: bool,
    pub language: Option<String>,
    pub region: Option<String>,
}

// Details response is just a Place

// ─── Photo Media ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PhotoMediaRequest {
    pub name: String,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMediaResponse {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub photo_uri: String,
}

// ─── Location Resolve ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ResolveRequest {
    pub location: String,
    pub limit: Option<u32>,
    pub language: Option<String>,
    pub region: Option<String>,
}

// Resolve response reuses SearchResponse

// ─── Route Search ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub query: String,
    pub from: String,
    pub to: String,
    pub travel_mode: TravelMode,
    pub search_radius: f64,
    pub max_waypoints: u32,
    pub results_per_waypoint: u32,
    pub language: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TravelMode {
    Drive,
    Walk,
    Bicycle,
    TwoWheeler,
    Transit,
}

impl TravelMode {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            TravelMode::Drive => "DRIVE",
            TravelMode::Walk => "WALK",
            TravelMode::Bicycle => "BICYCLE",
            TravelMode::TwoWheeler => "TWO_WHEELER",
            TravelMode::Transit => "TRANSIT",
        }
    }
}

impl std::str::FromStr for TravelMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRIVE" => Ok(TravelMode::Drive),
            "WALK" => Ok(TravelMode::Walk),
            "BICYCLE" => Ok(TravelMode::Bicycle),
            "TWO_WHEELER" => Ok(TravelMode::TwoWheeler),
            "TRANSIT" => Ok(TravelMode::Transit),
            _ => Err(format!(
                "invalid travel mode '{}': use DRIVE, WALK, BICYCLE, TWO_WHEELER, or TRANSIT",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteWaypointResult {
    pub waypoint: LatLng,
    pub waypoint_index: usize,
    pub places: Vec<Place>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSearchResponse {
    pub from: String,
    pub to: String,
    pub travel_mode: String,
    pub waypoints: Vec<RouteWaypointResult>,
}

// ─── Price level helpers ────────────────────────────────────────────────────

pub fn price_level_to_api(level: u8) -> Option<&'static str> {
    match level {
        0 => Some("PRICE_LEVEL_FREE"),
        1 => Some("PRICE_LEVEL_INEXPENSIVE"),
        2 => Some("PRICE_LEVEL_MODERATE"),
        3 => Some("PRICE_LEVEL_EXPENSIVE"),
        4 => Some("PRICE_LEVEL_VERY_EXPENSIVE"),
        _ => None,
    }
}

pub fn price_level_display(level: &str) -> &str {
    match level {
        "PRICE_LEVEL_FREE" => "Free",
        "PRICE_LEVEL_INEXPENSIVE" => "$",
        "PRICE_LEVEL_MODERATE" => "$$",
        "PRICE_LEVEL_EXPENSIVE" => "$$$",
        "PRICE_LEVEL_VERY_EXPENSIVE" => "$$$$",
        _ => level,
    }
}
