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
mod diff;
mod errors;
mod extractors;
mod github;
mod icalendar;
mod importers;
mod model;
mod util;

use crate::{
    config::Config,
    controllers::{add, bands, callers, cities, index, organisations, reload},
    diff::diff_markdown,
    errors::internal_error,
    importers::{balfolknl, cdss, folkbalbende, trycontra, webfeet},
    model::events::Events,
};
use axum::{
    extract::FromRef,
    routing::{get, get_service, post},
    Router,
};
use clap::{Parser, Subcommand};
use eyre::Report;
use log::info;
use schemars::schema_for;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Clone, Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Schema,
    Validate {
        events: Option<String>,
    },
    #[command(name = "cat")]
    Concatenate {
        events: Option<String>,
    },
    Sort {
        events: String,
    },
    Diff {
        old: String,
        new: String,
    },
    Balbende,
    Balfolknl,
    Cdss,
    Trycontra,
    Webfeet,
    #[command(name = "dups")]
    Duplicates,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let args = Args::parse();
    match &args.command {
        None => serve().await,
        Some(Command::Schema) => {
            // Output JSON schema for events.
            print!("{}", event_schema()?);
            Ok(())
        }
        Some(Command::Validate { events }) => validate(events.as_deref()).await,
        Some(Command::Concatenate { events }) => concatenate(events.as_deref()).await,
        Some(Command::Sort { events }) => sort(&events).await,
        Some(Command::Duplicates) => find_duplicates().await,
        Some(Command::Diff { old, new }) => diff(&old, &new).await,
        Some(Command::Balbende) => import_balbende().await,
        Some(Command::Balfolknl) => import_balfolknl().await,
        Some(Command::Cdss) => import_cdss().await,
        Some(Command::Trycontra) => import_trycontra().await,
        Some(Command::Webfeet) => import_webfeet().await,
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

/// Load the given file of events, and output them again sorted by start time, country then city.
async fn sort(path: &str) -> Result<(), Report> {
    let mut events = load_events(Some(path)).await?;
    // Sort by date then location.
    events.sort();
    print_events(&events)?;
    Ok(())
}

/// Loads the given two files of events, and outputs a diff between them in Markdown format.
async fn diff(path_a: &str, path_b: &str) -> Result<(), Report> {
    let events_a = load_events(Some(path_a)).await?.events;
    let events_b = load_events(Some(path_b)).await?.events;

    let markdown = diff_markdown(events_a, events_b)?;
    println!("{}", markdown);

    Ok(())
}

async fn import_balbende() -> Result<(), Report> {
    let events = folkbalbende::import_events().await?;
    print_events(&events)
}

async fn import_balfolknl() -> Result<(), Report> {
    let events = balfolknl::import_events().await?;
    print_events(&events)
}

async fn import_cdss() -> Result<(), Report> {
    let events = cdss::import_events().await?;
    print_events(&events)
}

async fn import_trycontra() -> Result<(), Report> {
    let events = trycontra::import_events().await?;
    print_events(&events)
}

async fn import_webfeet() -> Result<(), Report> {
    let events = webfeet::import_events().await?;
    print_events(&events)
}

fn print_events(events: &Events) -> Result<(), Report> {
    print!("{}", events.to_yaml_string()?);
    Ok(())
}

async fn find_duplicates() -> Result<(), Report> {
    let mut events = load_events(None).await?;

    // Sort by date then location, so that possible duplicates are next to each other.
    events.sort();
    for i in 1..events.events.len() {
        let a = &events.events[i - 1];
        let b = &events.events[i];
        if a.merge(b).is_some() {
            println!(
                "Found possible duplicate, {:?} in {}, {}:",
                a.time, a.country, a.city
            );
            println!(
                "  {} (from {})",
                a.name,
                a.source.as_deref().unwrap_or("unknown file")
            );
            println!(
                "  {} (from {})",
                b.name,
                b.source.as_deref().unwrap_or("unknown file")
            );
        }
    }

    Ok(())
}

async fn serve() -> Result<(), Report> {
    let config = Arc::new(Config::from_file()?);
    let events = Events::load_events(&config.events).await?;
    let events = Arc::new(Mutex::new(events));
    let state = AppState {
        config: config.clone(),
        events,
    };

    let app = Router::new()
        .route("/", get(index::index))
        .route("/index.ics", get(index::index_ics))
        .route("/index.json", get(index::index_json))
        .route("/index.toml", get(index::index_toml))
        .route("/index.yaml", get(index::index_yaml))
        .route("/calendar", get(index::calendar))
        .route("/add", get(add::add))
        .route("/add", post(add::submit))
        .route("/bands", get(bands::bands))
        .route("/callers", get(callers::callers))
        .route("/cities", get(cities::cities))
        .route("/organisations", get(organisations::organisations))
        .route("/reload", post(reload::reload))
        .nest_service(
            "/scripts",
            get_service(ServeDir::new(config.public_dir.join("scripts")))
                .handle_error(internal_error),
        )
        .nest_service(
            "/stylesheets",
            get_service(ServeDir::new(config.public_dir.join("stylesheets")))
                .handle_error(internal_error),
        )
        .with_state(state);

    info!("Listening on {}", config.bind_address);
    let listener = TcpListener::bind(&config.bind_address).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[derive(Clone, FromRef)]
struct AppState {
    config: Arc<Config>,
    events: Arc<Mutex<Events>>,
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
