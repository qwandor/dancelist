// Copyright 2022 the dancelist authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    Unauthorised,
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
            Self::Unauthorised => StatusCode::UNAUTHORIZED.into_response(),
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
