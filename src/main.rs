mod config;
mod controllers;
mod errors;
mod model;

use crate::{config::Config, controllers::index, model::events::Events};
use axum::{routing::get, AddExtensionLayer, Router};
use eyre::Report;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let config = Config::from_file()?;
    let events = Events::load()?;

    let app = Router::new()
        .route("/", get(index))
        .layer(AddExtensionLayer::new(events));

    info!("Listening on {}", config.bind_address);
    axum::Server::bind(&config.bind_address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
