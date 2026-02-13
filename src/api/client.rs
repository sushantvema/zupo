use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use super::errors::Error;

const PLACES_BASE_URL: &str = "https://places.googleapis.com/v1";
const ROUTES_BASE_URL: &str = "https://routes.googleapis.com";
const MAX_RESPONSE_BYTES: usize = 1_048_576; // 1 MB

pub struct Client {
    api_key: String,
    http: reqwest::Client,
    places_base_url: String,
    routes_base_url: String,
}

impl Client {
    pub fn new(api_key: String) -> Result<Self, Error> {
        if api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(Error::Http)?;

        Ok(Client {
            api_key,
            http,
            places_base_url: PLACES_BASE_URL.to_string(),
            routes_base_url: ROUTES_BASE_URL.to_string(),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or(self.http);
        self
    }

    pub fn with_places_base_url(mut self, url: String) -> Self {
        self.places_base_url = url;
        self
    }

    pub fn with_routes_base_url(mut self, url: String) -> Self {
        self.routes_base_url = url;
        self
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Goog-Api-Key",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }

    /// POST to a Places API endpoint with field mask
    pub(crate) async fn places_post(
        &self,
        path: &str,
        field_mask: &str,
        body: &Value,
    ) -> Result<Value, Error> {
        let url = format!("{}{}", self.places_base_url, path);
        let mut headers = self.auth_headers();
        headers.insert(
            "X-Goog-FieldMask",
            HeaderValue::from_str(field_mask).unwrap(),
        );

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// GET from a Places API endpoint with field mask
    pub(crate) async fn places_get(
        &self,
        path: &str,
        field_mask: &str,
        query_params: &[(&str, &str)],
    ) -> Result<Value, Error> {
        let url = format!("{}{}", self.places_base_url, path);
        let mut headers = self.auth_headers();
        if !field_mask.is_empty() {
            headers.insert(
                "X-Goog-FieldMask",
                HeaderValue::from_str(field_mask).unwrap(),
            );
        }

        let resp = self
            .http
            .get(&url)
            .headers(headers)
            .query(query_params)
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// POST to Routes API
    pub(crate) async fn routes_post(
        &self,
        path: &str,
        field_mask: &str,
        body: &Value,
    ) -> Result<Value, Error> {
        let url = format!("{}{}", self.routes_base_url, path);
        let mut headers = self.auth_headers();
        headers.insert(
            "X-Goog-FieldMask",
            HeaderValue::from_str(field_mask).unwrap(),
        );

        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await?;

        self.handle_response(resp).await
    }

    /// Download raw bytes from a URL (used for fetching photos)
    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, Error> {
        let resp = self.http.get(url).send().await?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await?;
        if status < 200 || status >= 300 {
            return Err(Error::Api {
                status,
                message: format!("failed to download: HTTP {}", status),
            });
        }
        Ok(bytes.to_vec())
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value, Error> {
        let status = resp.status().as_u16();

        // Read body with size limit
        let bytes = resp.bytes().await?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(Error::Api {
                status,
                message: format!("response too large: {} bytes", bytes.len()),
            });
        }

        if status < 200 || status >= 300 {
            let body_str = String::from_utf8_lossy(&bytes).to_string();
            return Err(Error::Api {
                status,
                message: body_str,
            });
        }

        // Empty body is valid for some endpoints (photo redirect)
        if bytes.is_empty() {
            return Ok(Value::Null);
        }

        serde_json::from_slice(&bytes).map_err(|e| Error::Api {
            status,
            message: format!("failed to parse JSON response: {}", e),
        })
    }
}
