use crate::{errors::InternalError, model::events::Events};
use askama::Template;
use axum::{extract::Extension, response::Html};

pub async fn organisations(
    Extension(events): Extension<Events>,
) -> Result<Html<String>, InternalError> {
    let organisations = events.organisations();
    let template = OrganisationsTemplate { organisations };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "organisations.html")]
struct OrganisationsTemplate {
    organisations: Vec<String>,
}
