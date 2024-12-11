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

    /// Create a new 404 error response.
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, None, "Not Found".to_string())
    }

    /// Create an error response indicating that a filter is invalid or not supported.
    pub fn invalid_filter() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidFilter),
            "Invalid Filter".to_string(),
        )
    }

    /// Create an error response indicating that an attribute path is invalid or not supported.
    pub fn invalid_path() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidPath),
            "Invalid Path".to_string(),
        )
    }

    /// Create an error response indicating that a different type of value was expected.
    pub fn expected(expected: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidValue),
            format!("Expected {}", expected),
        )
    }

    /// Create an error response indicating that a different type of value was expected.
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
    /// Too many results would be returned.
    TooMany,
    /// Conflict in attribute uniqueness.
    Uniqueness,
    /// The attribute cannot be modified.
    Mutability,
    /// Syntax error when parsing request.
    InvalidSyntax,
    /// The attribute path is invalid or not supported.
    InvalidPath,
    /// Attribute filter did not match any values.
    NoTarget,
    /// The attribute value is invalid.
    InvalidValue,
    /// The SCIM protocol version is not supported.
    InvalidVers,
    /// The attribute is sensitive and cannot be read.
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
