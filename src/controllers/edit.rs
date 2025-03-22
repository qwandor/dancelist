// Copyright 2025 the dancelist authors.
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

use super::event_form::EventForm;
use crate::{
    config::Config,
    errors::InternalError,
    github::edit_event_in_file,
    model::{
        event::Event,
        events::{Band, Caller, Country, Events, Organisation},
        filters::Filters,
    },
};
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use axum_extra::extract::Form;
use eyre::eyre;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

pub async fn edit(
    events: Events,
    Query(query): Query<EditQuery>,
) -> Result<Html<String>, InternalError> {
    let event = events
        .with_hash(&query.hash)
        .ok_or_else(|| InternalError::Internal(eyre!("Event not found")))?;
    let template = EditTemplate::new(&events, EventForm::from_event(event), vec![]);
    Ok(Html(template.render()?))
}

pub async fn submit(
    State(config): State<Arc<Config>>,
    events: Events,
    Query(query): Query<EditQuery>,
    Form(form): Form<EventForm>,
) -> Result<Html<String>, InternalError> {
    let original_event = events
        .with_hash(&query.hash)
        .ok_or_else(|| InternalError::Internal(eyre!("Event not found")))?;
    match Event::try_from(form.clone()) {
        Ok(mut event) => {
            // Set source before checking whether event has been changed.
            event.source = original_event.source.clone();
            if &event == original_event {
                let template = EditTemplate::new(&events, form, vec!["Event not changed"]);
                Ok(Html(template.render()?))
            } else {
                let file = original_event
                    .source
                    .as_deref()
                    .ok_or_else(|| InternalError::Internal(eyre!("Event missing source")))?;
                let pr = if let Some(github) = &config.github {
                    Some(
                        edit_event_in_file(
                            file,
                            original_event,
                            event.clone(),
                            form.email.as_deref(),
                            github,
                        )
                        .await?,
                    )
                } else {
                    None
                };
                let template = SubmitTemplate { pr, event };
                Ok(Html(template.render()?))
            }
        }
        Err(errors) => {
            let template = EditTemplate::new(&events, form, errors);
            Ok(Html(template.render()?))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EditQuery {
    hash: String,
}

#[derive(Template)]
#[template(path = "edit.html")]
struct EditTemplate {
    countries: Vec<Country>,
    bands: Vec<Band>,
    callers: Vec<Caller>,
    organisations: Vec<Organisation>,
    form: EventForm,
    errors: Vec<&'static str>,
}

impl EditTemplate {
    fn new(events: &Events, form: EventForm, errors: Vec<&'static str>) -> Self {
        let countries = events.countries(&Filters::all());
        let bands = events.bands();
        let callers = events.callers();
        let organisations = events.organisations();
        Self {
            countries,
            bands,
            callers,
            organisations,
            form,
            errors,
        }
    }
}

#[derive(Template)]
#[template(path = "edit_submit.html")]
struct SubmitTemplate {
    pr: Option<Url>,
    event: Event,
}

mod filters {
    pub use crate::util::checked_if_true;
}
