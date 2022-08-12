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

use crate::{
    errors::InternalError,
    model::{
        dancestyle::DanceStyle,
        event::{Event, EventTime},
        events::{Band, Caller, Country, Events, Organisation},
        filters::Filters,
    },
};
use askama::Template;
use axum::response::Html;
use axum_extra::extract::Form;
use chrono::NaiveDate;
use serde::{de::IntoDeserializer, Deserialize, Deserializer};

pub async fn add(events: Events) -> Result<Html<String>, InternalError> {
    let template = AddTemplate::new(&events, AddForm::default(), vec![]);
    Ok(Html(template.render()?))
}

pub async fn submit(
    events: Events,
    Form(form): Form<AddForm>,
) -> Result<Html<String>, InternalError> {
    match Event::try_from(form.clone()) {
        Ok(event) => Ok(Html(format!("<pre>{:#?}\n{:#?}</pre>", form, event))),
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
    form: AddForm,
    errors: Vec<&'static str>,
}

impl AddTemplate {
    fn new(events: &Events, form: AddForm, errors: Vec<&'static str>) -> Self {
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct AddForm {
    #[serde(deserialize_with = "trim")]
    name: String,
    #[serde(deserialize_with = "trim_non_empty")]
    details: Option<String>,
    links: String,
    #[serde(deserialize_with = "date_or_none")]
    start_date: Option<NaiveDate>,
    #[serde(deserialize_with = "date_or_none")]
    end_date: Option<NaiveDate>,
    #[serde(deserialize_with = "trim")]
    country: String,
    #[serde(deserialize_with = "trim")]
    city: String,
    #[serde(default)]
    styles: Vec<DanceStyle>,
    #[serde(default)]
    workshop: bool,
    #[serde(default)]
    social: bool,
    bands: Vec<String>,
    callers: Vec<String>,
    #[serde(deserialize_with = "trim_non_empty")]
    price: Option<String>,
    #[serde(deserialize_with = "trim_non_empty")]
    organisation: Option<String>,
}

impl AddForm {
    pub fn workshop(&self) -> bool {
        self.workshop
    }

    pub fn social(&self) -> bool {
        self.social
    }
}

impl TryFrom<AddForm> for Event {
    type Error = Vec<&'static str>;

    fn try_from(form: AddForm) -> Result<Self, Self::Error> {
        let time = EventTime::DateOnly {
            start_date: form.start_date.ok_or(vec!["Missing start date"])?,
            end_date: form.end_date.ok_or(vec!["Missing end date"])?,
        };
        let event = Self {
            name: form.name,
            details: form.details,
            links: vec![], // TODO
            time,
            country: form.country,
            city: form.city,
            styles: form.styles,
            workshop: form.workshop,
            social: form.social,
            bands: form
                .bands
                .into_iter()
                .filter_map(trimmed_non_empty)
                .collect(),
            callers: form
                .callers
                .into_iter()
                .filter_map(trimmed_non_empty)
                .collect(),
            price: form.price,
            organisation: form.organisation,
            cancelled: false,
            source: None,
        };
        let problems = event.validate();
        if problems.is_empty() {
            Ok(event)
        } else {
            Err(problems)
        }
    }
}

fn trim<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(String::deserialize(deserializer)?.trim().to_string())
}

fn trimmed_non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn trim_non_empty<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.and_then(trimmed_non_empty))
}

fn date_or_none<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<NaiveDate>, D::Error> {
    if let Some(str) = Option::<String>::deserialize(deserializer)? {
        if str.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NaiveDate::deserialize(str.into_deserializer())?))
        }
    } else {
        Ok(None)
    }
}

mod filters {
    pub fn checked_if_true(value: bool) -> askama::Result<&'static str> {
        Ok(if value { "checked=\"checked\"" } else { "" })
    }
}
