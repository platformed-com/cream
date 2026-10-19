use std::fmt::Display;

use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use cream_core::declare_schema;
use serde::Serialize;

use crate::json::Json;

declare_schema!(ErrorSchema = "urn:ietf:params:scim:api:messages:2.0:Error");

/// SCIM error response.
#[derive(Debug, Serialize)]
pub struct Error {
    /// ["urn:ietf:params:scim:api:messages:2.0:Error"]
    pub(crate) schemas: [ErrorSchema; 1],
    /// HTTP status code to be returned.
    #[serde(serialize_with = "Error::serialize_status")]
    pub(crate) status: StatusCode,
    /// SCIM error type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scim_type: Option<ErrorType>,
    /// Human-readable error message.
    pub(crate) detail: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scim_type) = &self.scim_type {
            write!(f, "{} [{}]: {}", self.status, scim_type, self.detail)
        } else {
            write!(f, "{}: {}", self.status, self.detail)
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    fn serialize_status<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(status.as_str())
    }

    /// Create a new error response.
    pub fn new(status: StatusCode, scim_type: Option<ErrorType>, detail: String) -> Self {
        Self {
            schemas: [ErrorSchema],
            status,
            scim_type,
            detail,
        }
    }

    /// Create a 404 error response.
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, None, "Not Found".to_string())
    }

    /// Create a 400 error response indicating that a filter is invalid or not supported.
    pub fn invalid_filter() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidFilter),
            "Invalid Filter".to_string(),
        )
    }

    /// Create a 400 error response indicating that an attribute path is invalid or not supported.
    pub fn invalid_path() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidPath),
            "Invalid Path".to_string(),
        )
    }

    /// Create a 400 error response indicating that a different type of value was expected.
    pub fn expected(expected: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidValue),
            format!("Expected {}", expected),
        )
    }

    /// Create a 409 error response indicating that the value of `attribute` is already in use.
    pub fn uniqueness(attribute: &str) -> Self {
        Self::new(
            StatusCode::CONFLICT,
            Some(ErrorType::Uniqueness),
            format!("Attribute `{}` must be unique", attribute),
        )
    }
}

/// SCIM error type.
#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ErrorType {
    /// The filter is invalid or not supported.
    InvalidFilter,
    /// The filter yields more results than the server is willing to process.
    TooMany,
    /// One or more attribute values are already in use or reserved.
    Uniqueness,
    /// The modification is not compatible with the attribute's mutability.
    Mutability,
    /// The request body is malformed or does not conform to the request schema.
    InvalidSyntax,
    /// The attribute path is invalid or not supported.
    InvalidPath,
    /// The path did not yield an attribute or value that could be operated on.
    NoTarget,
    /// A required value is missing, or a value is not compatible with the operation or attribute type.
    InvalidValue,
    /// The SCIM protocol version is not supported.
    InvalidVers,
    /// The request passed sensitive information, such as personal data, in the request URI.
    Sensitive,
}

serde_plain::derive_display_from_serialize!(ErrorType);

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self)).into_response()
    }
}

impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidSyntax),
            rejection.to_string(),
        )
    }
}
