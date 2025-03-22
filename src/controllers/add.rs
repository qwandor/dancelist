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

use super::event_form::EventForm;
use crate::{
    config::Config,
    errors::InternalError,
    github::{add_event_to_file, choose_file_for_event},
    model::{
        event::Event,
        events::{Band, Caller, Country, Events, Organisation},
        filters::Filters,
    },
};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::Form;
use std::sync::Arc;
use url::Url;

pub async fn add(events: Events) -> Result<Html<String>, InternalError> {
    let template = AddTemplate::new(
        &events,
        EventForm {
            with_time: true,
            ..Default::default()
        },
        vec![],
    );
    Ok(Html(template.render()?))
}

pub async fn submit(
    State(config): State<Arc<Config>>,
    events: Events,
    Form(form): Form<EventForm>,
) -> Result<Html<String>, InternalError> {
    match Event::try_from(form.clone()) {
        Ok(event) => match choose_file_for_event(&events, &event) {
            Ok(chosen_file) => {
                let pr = if let Some(github) = &config.github {
                    Some(
                        add_event_to_file(
                            event.clone(),
                            &chosen_file,
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
            Err(duplicate) => {
                let template = SubmitFailedTemplate {
                    event: &event,
                    existing_event: &duplicate.existing,
                    merged: &duplicate.merged,
                };
                Ok(Html(template.render()?))
            }
        },
        Err(errors) => {
            let template = AddTemplate::new(&events, form, errors);
            Ok(Html(template.render()?))
        }
    }
}

#[derive(Template)]
#[template(path = "add.html")]
struct AddTemplate {
    countries: Vec<Country>,
    bands: Vec<Band>,
    callers: Vec<Caller>,
    organisations: Vec<Organisation>,
    form: EventForm,
    errors: Vec<&'static str>,
}

impl AddTemplate {
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
#[template(path = "submit.html")]
struct SubmitTemplate {
    pr: Option<Url>,
    event: Event,
}

#[derive(Template)]
#[template(path = "submit_failed.html")]
struct SubmitFailedTemplate<'a> {
    event: &'a Event,
    existing_event: &'a Event,
    merged: &'a Event,
}

mod filters {
    pub use crate::util::checked_if_true;
}
