use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use eyre::Report;
use std::{error::Error, fmt::Debug};

/// Newtype wrapper around `Report` which implements `IntoResponse`.
pub enum InternalError {
    Internal(Report),
    NotFound,
}

impl<E: Error + Send + Sync + 'static> From<E> for InternalError {
    fn from(error: E) -> Self {
        Self::Internal(error.into())
    }
}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        match self {
            Self::Internal(report) => internal_error_response(report),
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

/// Converts an error into an 'internal server error' response.
pub async fn internal_error<E: Debug>(e: E) -> Response {
    internal_error_response(e)
}

fn internal_error_response<E: Debug>(e: E) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {:?}", e),
    )
        .into_response()
}
