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
    errors::InternalError,
    model::{
        events::{Band, Caller, Country, Events, Organisation},
        filters::Filters,
    },
};
use askama::Template;
use axum::{extract::Query, response::Html};
use eyre::eyre;
use serde::{Deserialize, Serialize};

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

mod filters {
    pub fn checked_if_true(value: bool) -> askama::Result<&'static str> {
        Ok(if value { "checked=\"checked\"" } else { "" })
    }
}
