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
mod extractors;
mod github;
mod icalendar;
mod importers;
mod model;
mod util;

use crate::{
    config::Config,
    controllers::{add, bands, callers, cities, index, organisations, reload},
    errors::internal_error,
    importers::{balfolknl, cdss, folkbalbende, trycontra, webfeet},
    model::events::Events,
};
use axum::{
    extract::FromRef,
    routing::{get, get_service, post},
    Router,
};
use eyre::Report;
use log::info;
use schemars::schema_for;
use std::{
    env,
    process::exit,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
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
    } else if args.len() == 3 && args[1] == "sort" {
        sort(&args[2]).await
    } else if args.len() == 2 && args[1] == "balbende" {
        import_balbende().await
    } else if args.len() == 2 && args[1] == "balfolknl" {
        import_balfolknl().await
    } else if args.len() == 2 && args[1] == "cdss" {
        import_cdss().await
    } else if args.len() == 2 && args[1] == "trycontra" {
        import_trycontra().await
    } else if args.len() == 2 && args[1] == "webfeet" {
        import_webfeet().await
    } else if args.len() == 2 && args[1] == "dups" {
        find_duplicates().await
    } else if args.len() == 2 && args[1] == "timezones" {
        dump_timezones().await
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

/// Load the given file of events, and output them again sorted by start time, country then city.
async fn sort(path: &str) -> Result<(), Report> {
    let mut events = load_events(Some(path)).await?;
    // Sort by date then location.
    events.events.sort_by_key(|event| {
        (
            event.time.start_time_sort_key(),
            event.country.clone(),
            event.city.clone(),
        )
    });
    print_events(&events)?;
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
    let yaml = serde_yaml::to_string(events)?;
    let yaml = yaml.replacen(
        "---",
        "# yaml-language-server: $schema=../../events_schema.json",
        1,
    );
    print!("{}", yaml);
    Ok(())
}

async fn find_duplicates() -> Result<(), Report> {
    let mut events = load_events(None).await?.events;

    // Sort by date then location, so that possible duplicates are next to each other.
    events.sort_by_key(|event| {
        (
            event.time.start_time_sort_key(),
            event.country.clone(),
            event.city.clone(),
        )
    });
    for i in 1..events.len() {
        let a = &events[i - 1];
        let b = &events[i];
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

async fn dump_timezones() -> Result<(), Report> {
    let events = load_events(None).await?;

    for ((country, state, city), timezone) in events.city_timezones() {
        println!(
            "{}, {}, {} => {}",
            country,
            state.unwrap_or_default(),
            city,
            timezone
        );
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
