use serde_json::json;

use super::client::Client;
use super::errors::Error;
use super::types::{NearbySearchRequest, NearbySearchResponse};

const NEARBY_FIELD_MASK: &str = "places.id,places.displayName,places.formattedAddress,\
places.shortFormattedAddress,places.types,places.primaryType,places.primaryTypeDisplayName,\
places.location,places.rating,places.userRatingCount,places.priceLevel,\
places.websiteUri,places.googleMapsUri,places.businessStatus,places.editorialSummary";

impl Client {
    pub async fn nearby_search(
        &self,
        req: &NearbySearchRequest,
    ) -> Result<NearbySearchResponse, Error> {
        validate_coords(req.lat, req.lng)?;
        if req.radius <= 0.0 {
            return Err(Error::Validation {
                field: "radius".into(),
                message: "radius must be positive".into(),
            });
        }

        let mut body = json!({
            "locationRestriction": {
                "circle": {
                    "center": { "latitude": req.lat, "longitude": req.lng },
                    "radius": req.radius,
                }
            },
        });

        if !req.included_types.is_empty() {
            body["includedTypes"] = json!(req.included_types);
        }
        if !req.excluded_types.is_empty() {
            body["excludedTypes"] = json!(req.excluded_types);
        }
        if let Some(limit) = req.limit {
            body["maxResultCount"] = json!(limit.min(20));
        }
        if let Some(ref lang) = req.language {
            body["languageCode"] = json!(lang);
        }
        if let Some(ref region) = req.region {
            body["regionCode"] = json!(region);
        }

        let result = self
            .places_post("/places:searchNearby", NEARBY_FIELD_MASK, &body)
            .await?;

        serde_json::from_value(result).map_err(|e| Error::Api {
            status: 0,
            message: format!("failed to parse nearby response: {}", e),
        })
    }
}

fn validate_coords(lat: f64, lng: f64) -> Result<(), Error> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(Error::Validation {
            field: "lat".into(),
            message: "latitude must be between -90 and 90".into(),
        });
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(Error::Validation {
            field: "lng".into(),
            message: "longitude must be between -180 and 180".into(),
        });
    }
    Ok(())
}
