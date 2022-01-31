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

mod config;
mod controllers;
mod errors;
mod model;

use crate::{
    config::Config,
    controllers::{bands, callers, index, organisations},
    errors::internal_error,
    model::events::Events,
};
use axum::{
    routing::{get, get_service},
    AddExtensionLayer, Router,
};
use eyre::Report;
use log::info;
use schemars::schema_for;
use std::env;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "schema" {
        // Output JSON schema for events.
        print!("{}", event_schema()?);
        return Ok(());
    }

    let config = Config::from_file()?;
    let events = Events::load(&config.events_dir)?;

    let app = Router::new()
        .route("/", get(index::index))
        .route("/index.toml", get(index::index_toml))
        .route("/index.yaml", get(index::index_yaml))
        .route("/bands", get(bands::bands))
        .route("/callers", get(callers::callers))
        .route("/organisations", get(organisations::organisations))
        .nest(
            "/stylesheets",
            get_service(ServeDir::new(config.public_dir.join("stylesheets")))
                .handle_error(internal_error),
        )
        .layer(AddExtensionLayer::new(events));

    info!("Listening on {}", config.bind_address);
    axum::Server::bind(&config.bind_address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Returns the JSON schema for events.
fn event_schema() -> Result<String, Report> {
    let schema = schema_for!(Events);
    Ok(serde_json::to_string_pretty(&schema)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn json_schema_matches() {
        assert_eq!(
            event_schema().unwrap(),
            read_to_string("events_schema.json").unwrap()
        );
    }
}
