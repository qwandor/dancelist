use crate::{errors::InternalError, model::events::Events};
use askama::Template;
use axum::{extract::Extension, response::Html};

pub async fn bands(Extension(events): Extension<Events>) -> Result<Html<String>, InternalError> {
    let bands = events.bands();
    let template = BandsTemplate { bands };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "bands.html")]
struct BandsTemplate {
    bands: Vec<String>,
}
