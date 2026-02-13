use serde_json::{json, Value};

use super::client::Client;
use super::errors::Error;
use super::types::{SearchRequest, SearchResponse};

const SEARCH_FIELD_MASK: &str = "places.id,places.displayName,places.formattedAddress,\
places.shortFormattedAddress,places.types,places.primaryType,places.primaryTypeDisplayName,\
places.location,places.rating,places.userRatingCount,places.priceLevel,\
places.websiteUri,places.googleMapsUri,places.businessStatus,places.editorialSummary";

impl Client {
    pub async fn search(&self, req: &SearchRequest) -> Result<SearchResponse, Error> {
        if req.query.is_empty() {
            return Err(Error::Validation {
                field: "query".into(),
                message: "query is required".into(),
            });
        }

        let mut body = json!({
            "textQuery": req.query,
        });

        if let Some(ref t) = req.included_type {
            body["includedType"] = json!(t);
        }
        if let Some(min) = req.min_rating {
            body["minRating"] = json!(min);
        }
        if !req.price_levels.is_empty() {
            body["priceLevels"] = json!(req.price_levels);
        }
        if req.open_now {
            body["openNow"] = json!(true);
        }
        if let Some(ref loc) = req.location {
            body["locationBias"] = json!({
                "circle": {
                    "center": { "latitude": loc.center.latitude, "longitude": loc.center.longitude },
                    "radius": loc.radius,
                }
            });
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
            .places_post("/places:searchText", SEARCH_FIELD_MASK, &body)
            .await?;

        parse_search_response(result)
    }
}

fn parse_search_response(value: Value) -> Result<SearchResponse, Error> {
    serde_json::from_value(value).map_err(|e| Error::Api {
        status: 0,
        message: format!("failed to parse search response: {}", e),
    })
}
