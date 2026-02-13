use super::client::Client;
use super::errors::Error;
use super::types::{PhotoMediaRequest, PhotoMediaResponse};

impl Client {
    pub async fn photo_media(&self, req: &PhotoMediaRequest) -> Result<PhotoMediaResponse, Error> {
        if req.name.is_empty() {
            return Err(Error::Validation {
                field: "name".into(),
                message: "photo resource name is required".into(),
            });
        }

        let path = format!("/{}/media", req.name);
        let mut query_params: Vec<(&str, String)> = Vec::new();

        if let Some(w) = req.max_width {
            query_params.push(("maxWidthPx", w.to_string()));
        }
        if let Some(h) = req.max_height {
            query_params.push(("maxHeightPx", h.to_string()));
        }

        // If neither dimension specified, default to max width 400
        if req.max_width.is_none() && req.max_height.is_none() {
            query_params.push(("maxWidthPx", "400".to_string()));
        }

        query_params.push(("skipHttpRedirect", "true".to_string()));

        let params: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let result = self.places_get(&path, "", &params).await?;

        serde_json::from_value(result).map_err(|e| Error::Api {
            status: 0,
            message: format!("failed to parse photo response: {}", e),
        })
    }
}
