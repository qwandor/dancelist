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
mod icalendar;
mod importers;
mod model;

use crate::{
    config::Config,
    controllers::{bands, callers, cities, index, organisations},
    errors::internal_error,
    importers::{balfolknl, folkbalbende, webfeet},
    model::events::Events,
};
use axum::{
    routing::{get, get_service},
    AddExtensionLayer, Router,
};
use eyre::Report;
use log::info;
use schemars::schema_for;
use std::{env, process::exit};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        serve().await
    } else if args.len() == 2 && args[1] == "schema" {
        // Output JSON schema for events.
        print!("{}", event_schema()?);
        Ok(())
    } else if args.len() >= 2 && args.len() <= 3 && args[1] == "validate" {
        validate(args.get(2).map(String::as_str)).await
    } else if args.len() >= 2 && args.len() <= 3 && args[1] == "cat" {
        concatenate(args.get(2).map(String::as_str)).await
    } else if args.len() == 2 && args[1] == "balbende" {
        import_balbende().await
    } else if args.len() == 2 && args[1] == "webfeet" {
        import_webfeet().await
    } else if args.len() == 2 && args[1] == "balfolknl" {
        import_balfolknl().await
    } else {
        eprintln!("Invalid command.");
        exit(1);
    }
}

/// Load events from the given file, directory or URL, or from the one in the config file if no path
/// is provided.
async fn load_events(path: Option<&str>) -> Result<Events, Report> {
    if let Some(path) = path {
        Events::load_events(path).await
    } else {
        let config = Config::from_file()?;
        Events::load_events(&config.events).await
    }
}

async fn validate(path: Option<&str>) -> Result<(), Report> {
    let events = load_events(path).await?;
    println!("Successfully validated {} events.", events.events.len());

    Ok(())
}

async fn concatenate(path: Option<&str>) -> Result<(), Report> {
    let events = load_events(path).await?;
    print!("{}", serde_yaml::to_string(&events)?);
    Ok(())
}

async fn import_balbende() -> Result<(), Report> {
    let events = folkbalbende::import_events().await?;
    print_events(&events)
}

async fn import_webfeet() -> Result<(), Report> {
    let events = webfeet::import_events().await?;
    print_events(&events)
}

async fn import_balfolknl() -> Result<(), Report> {
    let events = balfolknl::import_events().await?;
    print_events(&events)
}

fn print_events(events: &Events) -> Result<(), Report> {
    let yaml = serde_yaml::to_string(events)?;
    let yaml = yaml.replacen(
        "---",
        "# yaml-language-server: $schema=../../events_schema.json",
        1,
    );
    print!("{}", yaml);
    Ok(())
}

async fn serve() -> Result<(), Report> {
    let config = Config::from_file()?;
    let events = Events::load_events(&config.events).await?;

    let app = Router::new()
        .route("/", get(index::index))
        .route("/index.ics", get(index::index_ics))
        .route("/index.json", get(index::index_json))
        .route("/index.toml", get(index::index_toml))
        .route("/index.yaml", get(index::index_yaml))
        .route("/bands", get(bands::bands))
        .route("/callers", get(callers::callers))
        .route("/cities", get(cities::cities))
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
