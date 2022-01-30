use crate::{errors::InternalError, model::events::Events};
use askama::Template;
use axum::{extract::Extension, response::Html};

pub async fn callers(Extension(events): Extension<Events>) -> Result<Html<String>, InternalError> {
    let callers = events.callers();
    let template = CallersTemplate { callers };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "callers.html")]
struct CallersTemplate {
    callers: Vec<String>,
}
