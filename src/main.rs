mod controllers;
mod errors;
mod model;

use crate::controllers::index;
use crate::model::events::Events;
use axum::{routing::get, AddExtensionLayer, Router};
use eyre::Report;
use log::info;

const BIND_ADDRESS: &str = "127.0.0.1:1234";

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let events = Events::load()?;

    let app = Router::new()
        .route("/", get(index))
        .layer(AddExtensionLayer::new(events));

    let bind_address = BIND_ADDRESS.parse()?;
    info!("Listening on {}", bind_address);
    axum::Server::bind(&bind_address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
