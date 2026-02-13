use serde::Deserialize;

const IP_API_URL: &str = "http://ip-api.com/json/?fields=status,lat,lon,city,regionName,country";

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub lat: f64,
    pub lng: f64,
    pub description: String,
}

/// Geolocate via IP address using ip-api.com (free, no key required)
pub async fn geolocate_by_ip() -> Result<GeoLocation, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp: IpApiResponse = client
        .get(IP_API_URL)
        .send()
        .await
        .map_err(|e| format!("geolocation request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("geolocation parse failed: {}", e))?;

    if resp.status != "success" {
        return Err("IP geolocation failed".to_string());
    }

    let lat = resp.lat.ok_or("no latitude in response")?;
    let lng = resp.lon.ok_or("no longitude in response")?;

    let parts: Vec<&str> = [
        resp.city.as_deref(),
        resp.region_name.as_deref(),
        resp.country.as_deref(),
    ]
    .iter()
    .filter_map(|&s| s)
    .collect();

    let description = if parts.is_empty() {
        format!("{:.4}, {:.4}", lat, lng)
    } else {
        parts.join(", ")
    };

    Ok(GeoLocation {
        lat,
        lng,
        description,
    })
}
