use serde_json::json;

use super::client::Client;
use super::errors::Error;
use super::types::{AutocompleteRequest, AutocompleteResponse};

impl Client {
    pub async fn autocomplete(
        &self,
        req: &AutocompleteRequest,
    ) -> Result<AutocompleteResponse, Error> {
        if req.input.is_empty() {
            return Err(Error::Validation {
                field: "input".into(),
                message: "input is required".into(),
            });
        }

        let mut body = json!({
            "input": req.input,
        });

        if let Some(ref token) = req.session_token {
            body["sessionToken"] = json!(token);
        }
        if let Some(ref loc) = req.location {
            body["locationBias"] = json!({
                "circle": {
                    "center": { "latitude": loc.center.latitude, "longitude": loc.center.longitude },
                    "radius": loc.radius,
                }
            });
        }
        if let Some(ref lang) = req.language {
            body["languageCode"] = json!(lang);
        }
        if let Some(ref region) = req.region {
            body["regionCode"] = json!(region);
        }

        let result = self
            .places_post(
                "/places:autocomplete",
                "suggestions.placePrediction,suggestions.queryPrediction",
                &body,
            )
            .await?;

        let mut response: AutocompleteResponse = serde_json::from_value(result).map_err(|e| {
            Error::Api {
                status: 0,
                message: format!("failed to parse autocomplete response: {}", e),
            }
        })?;

        // Apply client-side limit (autocomplete API doesn't support maxResultCount)
        if let Some(limit) = req.limit {
            response.suggestions.truncate(limit as usize);
        }

        Ok(response)
    }
}
