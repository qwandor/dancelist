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

use crate::{config::Config, errors::InternalError, model::events::Events};
use axum::extract::{Form, State};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub async fn reload(
    State(events): State<Arc<Mutex<Events>>>,
    Form(request): Form<ReloadRequest>,
) -> Result<String, InternalError> {
    let config = Config::from_file().map_err(InternalError::Internal)?;

    if request.reload_token != config.reload_token {
        return Err(InternalError::Unauthorised);
    }

    let new_events = Events::load_events(&config.events)
        .await
        .map_err(InternalError::Internal)?;

    let mut events = events.lock().unwrap();
    *events = new_events;

    info!(
        "Reloaded {} events from {}.",
        events.events.len(),
        config.events,
    );

    Ok(format!("Reloaded {} events.\n", events.events.len()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReloadRequest {
    reload_token: String,
}
