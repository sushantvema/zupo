use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// API key is missing
    MissingApiKey,
    /// Validation error on a specific field
    Validation { field: String, message: String },
    /// HTTP/API error with status code and body
    Api { status: u16, message: String },
    /// Network or other reqwest error
    Http(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingApiKey => write!(
                f,
                "missing API key: set GOOGLE_PLACES_API_KEY or use --api-key"
            ),
            Error::Validation { field, message } => {
                write!(f, "validation error on '{}': {}", field, message)
            }
            Error::Api { status, message } => {
                write!(f, "API error (HTTP {}): {}", status, message)
            }
            Error::Http(e) => write!(f, "HTTP error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
    }
}
