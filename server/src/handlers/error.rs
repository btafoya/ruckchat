//! HTTP error response mapping.
//!
//! All handlers return a uniform JSON body for failures:
//!
//! ```json
//! { "error": "human readable message", "code": "validation" }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ruckchat_common::Error as DomainError;
use serde::Serialize;

/// Error response body returned to API clients.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    /// Short machine-readable error category.
    pub code: &'static str,
    /// Human-readable message.
    pub error: String,
}

impl ErrorBody {
    /// Creates an error body from a domain error.
    #[must_use]
    pub fn from_domain(err: &DomainError) -> Self {
        let (code, error) = match err {
            DomainError::Validation { message } => ("validation", message.clone()),
            DomainError::NotFound(message) => ("not_found", message.clone()),
            DomainError::Forbidden(message) => ("forbidden", message.clone()),
            DomainError::Unauthorized(message) => ("unauthorized", message.clone()),
            DomainError::Conflict(message) => ("conflict", message.clone()),
            DomainError::Internal(message) => ("internal", message.clone()),
            DomainError::TooManyRequests(message) => ("too_many_requests", message.clone()),
        };
        Self { code, error }
    }
}

/// Converts a server error into an HTTP response.
impl IntoResponse for crate::Error {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Domain(ref err) => {
                let status = domain_status(err);
                (status, Json(ErrorBody::from_domain(err)))
            }
            Self::PasswordHash | Self::TokenGeneration => {
                let body = ErrorBody {
                    code: "internal",
                    error: self.to_string(),
                };
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body))
            }
        };
        (status, body).into_response()
    }
}

fn domain_status(err: &DomainError) -> StatusCode {
    match err {
        DomainError::Validation { .. } => StatusCode::BAD_REQUEST,
        DomainError::NotFound(_) => StatusCode::NOT_FOUND,
        DomainError::Forbidden(_) => StatusCode::FORBIDDEN,
        DomainError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        DomainError::Conflict(_) => StatusCode::CONFLICT,
        DomainError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DomainError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
    }
}

/// Converts a JSON extraction failure into a 422 response.
pub fn json_rejection(err: axum::extract::rejection::JsonRejection) -> Response {
    let body = ErrorBody {
        code: "validation",
        error: err.to_string(),
    };
    (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
}
