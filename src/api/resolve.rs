use serde_json::json;

use super::client::Client;
use super::errors::Error;
use super::types::{ResolveRequest, SearchResponse};

const RESOLVE_FIELD_MASK: &str = "places.id,places.displayName,places.formattedAddress,\
places.shortFormattedAddress,places.types,places.primaryType,places.primaryTypeDisplayName,\
places.location,places.rating,places.userRatingCount,places.priceLevel,\
places.websiteUri,places.googleMapsUri,places.businessStatus,places.editorialSummary";

impl Client {
    pub async fn resolve(&self, req: &ResolveRequest) -> Result<SearchResponse, Error> {
        if req.location.is_empty() {
            return Err(Error::Validation {
                field: "location".into(),
                message: "location is required".into(),
            });
        }

        let mut body = json!({
            "textQuery": req.location,
        });

        if let Some(limit) = req.limit {
            body["maxResultCount"] = json!(limit.min(10));
        }
        if let Some(ref lang) = req.language {
            body["languageCode"] = json!(lang);
        }
        if let Some(ref region) = req.region {
            body["regionCode"] = json!(region);
        }

        let result = self
            .places_post("/places:searchText", RESOLVE_FIELD_MASK, &body)
            .await?;

        serde_json::from_value(result).map_err(|e| Error::Api {
            status: 0,
            message: format!("failed to parse resolve response: {}", e),
        })
    }
}
