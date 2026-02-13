use super::client::Client;
use super::errors::Error;
use super::types::{DetailsRequest, Place};

impl Client {
    pub async fn details(&self, req: &DetailsRequest) -> Result<Place, Error> {
        if req.place_id.is_empty() {
            return Err(Error::Validation {
                field: "place_id".into(),
                message: "place_id is required".into(),
            });
        }

        let mut fields = vec![
            "id",
            "displayName",
            "formattedAddress",
            "shortFormattedAddress",
            "types",
            "primaryType",
            "primaryTypeDisplayName",
            "location",
            "rating",
            "userRatingCount",
            "priceLevel",
            "websiteUri",
            "googleMapsUri",
            "nationalPhoneNumber",
            "internationalPhoneNumber",
            "currentOpeningHours",
            "regularOpeningHours",
            "businessStatus",
            "editorialSummary",
        ];

        if req.include_reviews {
            fields.push("reviews");
        }
        if req.include_photos {
            fields.push("photos");
        }

        let field_mask = fields.join(",");
        let path = format!("/places/{}", req.place_id);

        let mut query_params: Vec<(&str, &str)> = Vec::new();
        let lang_val;
        let region_val;
        if let Some(ref lang) = req.language {
            lang_val = lang.clone();
            query_params.push(("languageCode", &lang_val));
        }
        if let Some(ref region) = req.region {
            region_val = region.clone();
            query_params.push(("regionCode", &region_val));
        }

        let result = self.places_get(&path, &field_mask, &query_params).await?;

        serde_json::from_value(result).map_err(|e| Error::Api {
            status: 0,
            message: format!("failed to parse details response: {}", e),
        })
    }
}
