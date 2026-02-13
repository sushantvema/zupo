use serde_json::json;

use super::client::Client;
use super::errors::Error;
use super::types::{
    Circle, LatLng, RouteRequest, RouteSearchResponse, RouteWaypointResult, SearchRequest,
};

impl Client {
    pub async fn route_search(&self, req: &RouteRequest) -> Result<RouteSearchResponse, Error> {
        if req.query.is_empty() {
            return Err(Error::Validation {
                field: "query".into(),
                message: "query is required".into(),
            });
        }
        if req.from.is_empty() {
            return Err(Error::Validation {
                field: "from".into(),
                message: "origin is required".into(),
            });
        }
        if req.to.is_empty() {
            return Err(Error::Validation {
                field: "to".into(),
                message: "destination is required".into(),
            });
        }

        // Step 1: Compute route via Routes API
        let polyline = self.compute_route_polyline(req).await?;

        // Step 2: Decode polyline into points
        let points = decode_polyline(&polyline);
        if points.is_empty() {
            return Err(Error::Api {
                status: 0,
                message: "route returned no path points".into(),
            });
        }

        // Step 3: Sample waypoints along the route
        let waypoints = sample_waypoints(&points, req.max_waypoints as usize);

        // Step 4: Search near each waypoint
        let mut results = Vec::new();
        for (idx, wp) in waypoints.iter().enumerate() {
            let search_req = SearchRequest {
                query: req.query.clone(),
                included_type: None,
                min_rating: None,
                price_levels: vec![],
                open_now: false,
                location: Some(Circle {
                    center: wp.clone(),
                    radius: req.search_radius,
                }),
                limit: Some(req.results_per_waypoint),
                language: req.language.clone(),
                region: req.region.clone(),
            };

            match self.search(&search_req).await {
                Ok(resp) => {
                    results.push(RouteWaypointResult {
                        waypoint: wp.clone(),
                        waypoint_index: idx,
                        places: resp.places,
                    });
                }
                Err(_) => {
                    // Skip waypoints that fail (e.g., no results in that area)
                    results.push(RouteWaypointResult {
                        waypoint: wp.clone(),
                        waypoint_index: idx,
                        places: vec![],
                    });
                }
            }
        }

        Ok(RouteSearchResponse {
            from: req.from.clone(),
            to: req.to.clone(),
            travel_mode: req.travel_mode.as_api_str().to_string(),
            waypoints: results,
        })
    }

    async fn compute_route_polyline(&self, req: &RouteRequest) -> Result<String, Error> {
        let body = json!({
            "origin": {
                "address": req.from,
            },
            "destination": {
                "address": req.to,
            },
            "travelMode": req.travel_mode.as_api_str(),
            "polylineEncoding": "ENCODED_POLYLINE",
        });

        let result = self
            .routes_post(
                "/directions/v2:computeRoutes",
                "routes.polyline.encodedPolyline",
                &body,
            )
            .await?;

        // Extract encoded polyline from response
        result["routes"]
            .as_array()
            .and_then(|routes| routes.first())
            .and_then(|route| route["polyline"]["encodedPolyline"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Api {
                status: 0,
                message: "no route found between origin and destination".into(),
            })
    }
}

/// Decode a Google encoded polyline string into a list of LatLng points
fn decode_polyline(encoded: &str) -> Vec<LatLng> {
    let mut points = Vec::new();
    let mut lat: i64 = 0;
    let mut lng: i64 = 0;
    let bytes = encoded.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Decode latitude
        let mut shift = 0u32;
        let mut result: i64 = 0;
        loop {
            if i >= bytes.len() {
                break;
            }
            let b = (bytes[i] as i64) - 63;
            i += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        lat += if result & 1 != 0 {
            !(result >> 1)
        } else {
            result >> 1
        };

        // Decode longitude
        shift = 0;
        result = 0;
        loop {
            if i >= bytes.len() {
                break;
            }
            let b = (bytes[i] as i64) - 63;
            i += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        lng += if result & 1 != 0 {
            !(result >> 1)
        } else {
            result >> 1
        };

        points.push(LatLng {
            latitude: lat as f64 / 1e5,
            longitude: lng as f64 / 1e5,
        });
    }

    points
}

/// Haversine distance in meters between two points
fn haversine_distance(a: &LatLng, b: &LatLng) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in meters
    let d_lat = (b.latitude - a.latitude).to_radians();
    let d_lng = (b.longitude - a.longitude).to_radians();
    let lat1 = a.latitude.to_radians();
    let lat2 = b.latitude.to_radians();

    let a_val = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lng / 2.0).sin().powi(2);
    let c = 2.0 * a_val.sqrt().asin();
    R * c
}

/// Sample N evenly-spaced waypoints along a polyline path
fn sample_waypoints(points: &[LatLng], n: usize) -> Vec<LatLng> {
    if points.len() <= 1 || n == 0 {
        return points.to_vec();
    }

    // Compute cumulative distances
    let mut cumulative = vec![0.0f64];
    for i in 1..points.len() {
        let d = haversine_distance(&points[i - 1], &points[i]);
        cumulative.push(cumulative[i - 1] + d);
    }

    let total_distance = *cumulative.last().unwrap();
    if total_distance == 0.0 {
        return vec![points[0].clone()];
    }

    let mut waypoints = Vec::with_capacity(n);
    for i in 0..n {
        let target = total_distance * (i as f64 + 0.5) / n as f64;

        // Find segment containing this target distance
        let seg = cumulative
            .partition_point(|&d| d < target)
            .saturating_sub(1)
            .min(points.len() - 2);

        let seg_start = cumulative[seg];
        let seg_len = cumulative[seg + 1] - seg_start;

        if seg_len == 0.0 {
            waypoints.push(points[seg].clone());
        } else {
            let t = (target - seg_start) / seg_len;
            waypoints.push(LatLng {
                latitude: points[seg].latitude + t * (points[seg + 1].latitude - points[seg].latitude),
                longitude: points[seg].longitude
                    + t * (points[seg + 1].longitude - points[seg].longitude),
            });
        }
    }

    waypoints
}
