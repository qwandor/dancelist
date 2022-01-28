use crate::{
    errors::InternalError,
    model::{event::Event, events::Events},
};
use askama::Template;
use axum::{extract::Extension, response::Html};

pub async fn index(Extension(events): Extension<Events>) -> Result<Html<String>, InternalError> {
    let template = IndexTemplate {
        events: events.events,
    };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    events: Vec<Event>,
}
