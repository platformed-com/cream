use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use cream_core::declare_schema;
use serde::Serialize;

use crate::json::Json;

declare_schema!(ErrorSchema = "urn:ietf:params:scim:api:messages:2.0:Error");

#[derive(Serialize)]
pub struct Error {
    pub schemas: [ErrorSchema; 1],
    #[serde(serialize_with = "Error::serialize_status")]
    pub status: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scim_type: Option<ErrorType>,
    pub detail: String,
}

impl Error {
    fn serialize_status<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(status.as_str())
    }

    pub fn new(status: StatusCode, scim_type: Option<ErrorType>, detail: String) -> Self {
        Self {
            schemas: [ErrorSchema],
            status,
            scim_type,
            detail,
        }
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, None, "Not Found".to_string())
    }
    pub fn invalid_filter() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidFilter),
            "Invalid Filter".to_string(),
        )
    }
    pub fn invalid_path() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidPath),
            "Invalid Path".to_string(),
        )
    }
    pub fn expected(expected: &str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            Some(ErrorType::InvalidValue),
            format!("Expected {}", expected),
        )
    }
}

#[allow(unused)]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ErrorType {
    InvalidFilter,
    TooMany,
    Uniqueness,
    Mutability,
    InvalidSyntax,
    InvalidPath,
    NoTarget,
    InvalidValue,
    InvalidVers,
    Sensitive,
}

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
