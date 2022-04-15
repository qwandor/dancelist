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
mod icalendar;
mod importers;
mod model;

use crate::{
    config::Config,
    controllers::{bands, callers, cities, index, organisations, reload},
    errors::internal_error,
    importers::{balfolknl, folkbalbende, webfeet},
    model::events::Events,
};
use axum::{
    routing::{get, get_service, post},
    Extension, Router,
};
use eyre::Report;
use log::info;
use schemars::schema_for;
use shuttle_service::{IntoService, Service};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;
use tower_http::services::ServeDir;

/// Load events from the given file, directory or URL, or from the one in the config file if no path
/// is provided.
pub async fn load_events(path: Option<&str>) -> Result<Events, Report> {
    if let Some(path) = path {
        Events::load_events(path).await
    } else {
        let config = Config::from_file()?;
        Events::load_events(&config.events).await
    }
}

pub async fn validate(path: Option<&str>) -> Result<(), Report> {
    let events = load_events(path).await?;
    println!("Successfully validated {} events.", events.events.len());

    Ok(())
}

pub async fn concatenate(path: Option<&str>) -> Result<(), Report> {
    let events = load_events(path).await?;
    print!("{}", serde_yaml::to_string(&events)?);
    Ok(())
}

pub async fn import_balbende() -> Result<(), Report> {
    let events = folkbalbende::import_events().await?;
    print_events(&events)
}

pub async fn import_webfeet() -> Result<(), Report> {
    let events = webfeet::import_events().await?;
    print_events(&events)
}

pub async fn import_balfolknl() -> Result<(), Report> {
    let events = balfolknl::import_events().await?;
    print_events(&events)
}

pub fn print_events(events: &Events) -> Result<(), Report> {
    let yaml = serde_yaml::to_string(events)?;
    let yaml = yaml.replacen(
        "---",
        "# yaml-language-server: $schema=../../events_schema.json",
        1,
    );
    print!("{}", yaml);
    Ok(())
}

pub async fn setup_app(config: &Config) -> Result<Router, Report> {
    let events = Events::load_events(&config.events).await?;
    let events = Arc::new(Mutex::new(events));

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
        .route("/reload", post(reload::reload))
        .nest(
            "/stylesheets",
            get_service(ServeDir::new(config.public_dir.join("stylesheets")))
                .handle_error(internal_error),
        )
        .layer(Extension(events));

    Ok(app)
}

pub async fn serve() -> Result<(), Report> {
    let config = Config::from_file()?;
    let app = setup_app(&config).await?;

    info!("Listening on {}", config.bind_address);
    axum::Server::bind(&config.bind_address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub async fn serve_shuttle(addr: SocketAddr) -> Result<(), Report> {
    println!("in shuttle_service()");
    log::warn!("in shuttle_service()");
    let config = Config::from_file()?;
    let app = setup_app(&config).await?;

    println!("Listening on {}", config.bind_address);
    log::warn!("Listening on {}", config.bind_address);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
// We can't use the web-axum feature because there is no released 0.5 version on crates.io yet.
// We can't use the shuttle_service::main macro because that needs a SimpleService, and orphan rules
// do not allow us to impl anything useful for SimpleService outside of the shuttle_service crate.
struct MyService;
impl MyService {
    fn new() -> Self {
        println!("in MyService::new()");
        log::warn!("in MyService::new()");
        Self
    }
}

impl IntoService for MyService {
    type Service = Self;

    fn into_service(self) -> Self::Service {
        println!("in into_service()");
        log::warn!("in into_service()");
        self
    }
}

fn eyre_to_anyhow(e: Report) -> anyhow::Error {
    let e: Box<dyn std::error::Error + Send + Sync + 'static> = e.into();
    anyhow::anyhow!(dbg!(e))
}

impl Service for MyService {
    fn bind(&mut self, addr: SocketAddr) -> Result<(), shuttle_service::error::Error> {
        println!("in bind()");
        log::warn!("in bind()");
        let rt = Runtime::new().unwrap();
        rt.block_on(serve_shuttle(addr)).map_err(eyre_to_anyhow)?;
        println!("out bind()");
        log::warn!("out bind()");
        Ok(())
    }
}
shuttle_service::declare_service!(MyService, MyService::new);

/// Returns the JSON schema for events.
pub fn event_schema() -> Result<String, Report> {
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
