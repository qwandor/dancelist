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
    controllers::{add, bands, callers, cities, edit, index, organisations, reload},
    diff::diff_markdown,
    errors::internal_error,
    importers::{
        folkbalbende,
        icalendar::{
            balfolkcambridge::BalfolkCambridge, balfolknl::BalfolkNl, boulder::Boulder,
            bristolcontra::BristolContra, burghausen::Burghausen, cdss::Cdss,
            ceilidhclub::CeilidhClub, contrabridge::Contrabridge, dresden, freiburg::Freiburg,
            import_events, kalender::Kalender, lancastercontra::LancasterContra, marburg::Marburg,
            skandia::Skandia, spreefolk::Spreefolk,
        },
        plugevents, trycontra, webfeet,
    },
    model::events::Events,
};
use axum::{
    Router,
    extract::FromRef,
    routing::{get, get_service, post},
};
use clap::{Parser, Subcommand, ValueEnum};
use eyre::Report;
use importers::write_by_country;
use log::info;
use schemars::schema_for;
use std::{
    fs::write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone, Debug, Parser)]
struct Args {
    /// If no command is specified, loads events according to the config file and serves the
    /// website.
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Prints out a JSON schema for events.
    Schema,
    /// Validates events from the given file, directory or URL.
    ///
    /// If no path or URL is specified, uses the one configured in the config file.
    Validate { events: Option<String> },
    /// Loads all events from the given file, directory or URL, and prints them as a single file.
    ///
    /// If no path or URL is specified, uses the one configured in the config file.
    #[command(name = "cat")]
    Concatenate { events: Option<String> },
    /// Loads all events from the given file, directory or URL, and prints them sorted by start
    /// time, country then city.
    Sort { events: String },
    /// Loads the given two files (or directories or URLs) of events, and outputs a diff between
    /// them in Markdown format.
    Diff { old: String, new: String },
    /// Imports events from another site.
    Import {
        /// The source from which to import events.
        source: ImportSource,
        /// The file to which to write the imported events.
        filename: PathBuf,
    },
    /// Imports events from plug.events.
    ImportPlugEvents {
        /// The API token to use.
        token: String,
        /// The file to which to write the imported events.
        filename: PathBuf,
    },
    /// Loads events as configured in the config file and tries to find duplicates.
    #[command(name = "dups")]
    Duplicates,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ImportSource {
    /// Imports events from folkbalbende.be.
    Balbende,
    /// Imports events from balfolkcambridge.org.
    BalfolkCambridge,
    /// Imports events from balfolk.nl.
    Balfolknl,
    /// Imports events from boulderdance.org.
    Boulder,
    /// Imports events from bristolcontra.wordpress.com.
    BristolContra,
    /// Imports events from balfolk-burghausen.de.
    Burghausen,
    /// Imports events from ceilidhclub.com.
    CeilidhClub,
    /// Imports events from cdss.org.
    Cdss,
    /// Imports events from contrabridge.org.
    Contrabridge,
    /// Imports events from gugelhupf-dresden.de.
    Dresden,
    /// Imports events from 7schritt.de.
    Freiburg,
    /// Imports events from Balfolk-Orga-Kalender.
    Kalender,
    /// Imports events from lancastercontra.org.uk.
    LancasterContra,
    /// Imports events from folkclub-marburg.de.
    Marburg,
    /// Imports events from skandia-folkdance.org\
    Skandia,
    /// Imports events from spreefolk.de.
    Spreefolk,
    /// Imports longer events from trycontra.com.
    Trycontra,
    /// Imports events from webfeet.org.
    Webfeet,
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
        Some(Command::Sort { events }) => sort(events).await,
        Some(Command::Duplicates) => find_duplicates().await,
        Some(Command::Diff { old, new }) => diff(old, new).await,
        Some(Command::Import { source, filename }) => import(*source, filename).await,
        Some(Command::ImportPlugEvents { token, filename }) => {
            import_plug_events(token, filename).await
        }
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
    let mut events = Events::load_events(path).await?;
    // Sort by date then location.
    events.sort();
    print_events(&events)?;
    Ok(())
}

/// Loads the given two files of events, and outputs a diff between them in Markdown format.
async fn diff(path_a: &str, path_b: &str) -> Result<(), Report> {
    let events_a = Events::load_events(path_a).await?.events;
    let events_b = Events::load_events(path_b).await?.events;

    let markdown = diff_markdown(events_a, events_b)?;
    println!("{}", markdown);

    Ok(())
}

async fn import(source: ImportSource, filename: &Path) -> Result<(), Report> {
    let old_events = if filename.exists() {
        // Load without validating, as imports may be invalid.
        Events::load_file_without_validation(filename)?
    } else {
        Events::default()
    };
    let events = match source {
        ImportSource::Balbende => folkbalbende::import_events().await?,
        ImportSource::BalfolkCambridge => import_events::<BalfolkCambridge>(old_events).await?,
        ImportSource::Balfolknl => import_events::<BalfolkNl>(old_events).await?,
        ImportSource::Boulder => import_events::<Boulder>(old_events).await?,
        ImportSource::BristolContra => import_events::<BristolContra>(old_events).await?,
        ImportSource::Burghausen => import_events::<Burghausen>(old_events).await?,
        ImportSource::CeilidhClub => import_events::<CeilidhClub>(old_events).await?,
        ImportSource::Cdss => import_events::<Cdss>(old_events).await?,
        ImportSource::Contrabridge => import_events::<Contrabridge>(old_events).await?,
        ImportSource::Dresden => dresden::import_events(old_events).await?,
        ImportSource::Freiburg => import_events::<Freiburg>(old_events).await?,
        ImportSource::Kalender => import_events::<Kalender>(old_events).await?,
        ImportSource::Marburg => import_events::<Marburg>(old_events).await?,
        ImportSource::Skandia => import_events::<Skandia>(old_events).await?,
        ImportSource::Spreefolk => import_events::<Spreefolk>(old_events).await?,
        ImportSource::LancasterContra => import_events::<LancasterContra>(old_events).await?,
        ImportSource::Trycontra => trycontra::import_events().await?,
        ImportSource::Webfeet => webfeet::import_events().await?,
    };
    write(filename, events.to_yaml_string()?)?;
    Ok(())
}

async fn import_plug_events(token: &str, filename: &Path) -> Result<(), Report> {
    let events = plugevents::import_events(token).await?;
    write_by_country(events, filename)?;
    Ok(())
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
        .route("/index_edit", get(index::index_edit))
        .route("/flyer", get(index::flyer))
        .route("/add", get(add::add))
        .route("/add", post(add::submit))
        .route("/edit", get(edit::edit))
        .route("/edit", post(edit::submit))
        .route("/bands", get(bands::bands))
        .route("/callers", get(callers::callers))
        .route("/cities", get(cities::cities))
        .route("/organisations", get(organisations::organisations))
        .route("/reload", post(reload::reload))
        .route_service(
            "/robots.txt",
            get_service(ServeFile::new(config.public_dir.join("robots.txt"))),
        )
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
