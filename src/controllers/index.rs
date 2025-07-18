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
    icalendar::{Ics, events_to_calendar},
    model::{
        dancestyle::DanceStyle,
        event::Event,
        events::{Country, Events},
        filters::Filters,
    },
};
use askama::Template;
use axum::{extract::Query, response::Html};
use axum_extra::{TypedHeader, headers::Host};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{Datelike, Months, NaiveDate};
use fast_qr::{
    QRBuilder,
    convert::{Builder, image::ImageBuilder},
};

pub async fn index(
    events: Events,
    Query(filters): Query<Filters>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Html<String>, InternalError> {
    index_html(events, filters, host, false, false).await
}

pub async fn index_edit(
    events: Events,
    Query(filters): Query<Filters>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Html<String>, InternalError> {
    index_html(events, filters, host, false, true).await
}

pub async fn calendar(
    events: Events,
    Query(filters): Query<Filters>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Html<String>, InternalError> {
    index_html(events, filters, host, true, false).await
}

pub async fn index_html(
    events: Events,
    mut filters: Filters,
    host: Host,
    calendar: bool,
    show_edit_link: bool,
) -> Result<Html<String>, InternalError> {
    let has_filters = filters.has_some();

    if host.hostname().contains("balfolk.org") && filters.styles.is_empty() {
        // Default to only showing Balfolk events.
        filters.styles = [DanceStyle::Balfolk].into_iter().collect();
    }

    let countries = events.countries(&filters.with_country(None));
    let states = if !filters.country.is_empty() {
        events.states(&filters.with_state(None))
    } else {
        vec![]
    };
    let styles = events.styles(&filters.with_style(None));
    let cities = if !filters.country.is_empty() {
        events.cities(&filters.with_city(None))
    } else {
        vec![]
    };
    let events = events.sorted_matching(&filters);
    let months = group_by_month(events.events);
    let template = IndexTemplate {
        filters,
        months,
        has_filters,
        countries,
        states,
        cities,
        styles,
        calendar,
        show_edit_link,
    };
    Ok(Html(template.render()?))
}

pub async fn index_json(
    events: Events,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let events = events.sorted_matching(&filters);
    Ok(serde_json::to_string(&events)?)
}

pub async fn index_toml(
    events: Events,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let events = events.sorted_matching(&filters);
    Ok(toml::to_string(&events)?)
}

pub async fn index_yaml(
    events: Events,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let events = events.sorted_matching(&filters);
    Ok(serde_yaml::to_string(&events)?)
}

pub async fn index_ics(
    events: Events,
    Query(mut filters): Query<Filters>,
) -> Result<Ics, InternalError> {
    // Default to hiding cancelled events unless the filter explicitly asks for them.
    if filters.cancelled.is_none() {
        filters.cancelled = Some(false);
    }

    let events = events.sorted_matching(&filters);
    let calendar = events_to_calendar(&events.events, &filters.make_title());
    Ok(Ics(calendar))
}

pub async fn flyer(
    events: Events,
    Query(mut filters): Query<Filters>,
) -> Result<Html<String>, InternalError> {
    let events = events.sorted_matching(&filters);
    let months = group_by_month(events.events);

    filters.limit = None;
    let qr_code_link = format!(
        "https://folkdance.page/?{}",
        filters.to_query_string().map_err(InternalError::Internal)?
    );
    let qr_code = QRBuilder::new(qr_code_link.clone()).build()?;
    let qr_code_image = ImageBuilder::default().margin(0).to_bytes(&qr_code)?;

    let template = FlyerTemplate {
        filters,
        months,
        qr_code_uri: format!("data:image/png;base64,{}", STANDARD.encode(qr_code_image)),
        qr_code_size: qr_code.size,
        qr_code_link,
    };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    filters: Filters,
    months: Vec<Month>,
    has_filters: bool,
    countries: Vec<Country>,
    states: Vec<String>,
    cities: Vec<String>,
    styles: Vec<DanceStyle>,
    calendar: bool,
    show_edit_link: bool,
}

#[derive(Template)]
#[template(path = "flyer.html")]
struct FlyerTemplate {
    filters: Filters,
    months: Vec<Month>,
    qr_code_uri: String,
    qr_code_size: usize,
    qr_code_link: String,
}

struct Month {
    /// The first day of the month.
    start: NaiveDate,
    events: Vec<Event>,
}

impl Month {
    pub fn name(&self) -> String {
        self.start.format("%B %Y").to_string()
    }

    pub fn calendar(&self) -> Vec<Vec<Day>> {
        // Start with some empty days before the month to start on the right weekday.
        let mut days = vec![Day::default(); self.start.weekday().num_days_from_monday() as usize];
        let end = self.start + Months::new(1);
        for day in self.start.iter_days().take_while(|day| day < &end) {
            days.push(Day {
                day_of_month: Some(day.day()),
                events: self
                    .events
                    .iter()
                    .filter(|event| event.time.start_date() == day)
                    .cloned()
                    .collect(),
            });
        }
        days.chunks(7).map(ToOwned::to_owned).collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Day {
    day_of_month: Option<u32>,
    events: Vec<Event>,
}

/// Given a list of events sorted in ascending order of start date, groups them by starting month.
fn group_by_month(events: Vec<Event>) -> Vec<Month> {
    let mut months = vec![];
    let mut month = Month {
        start: NaiveDate::MIN,
        events: vec![],
    };
    for event in events {
        if event.start_year() == month.start.year() && event.start_month() == month.start.month() {
            month.events.push(event);
        } else {
            if !month.events.is_empty() {
                months.push(month);
            }
            month = Month {
                start: NaiveDate::from_ymd_opt(event.start_year(), event.start_month(), 1).unwrap(),
                events: vec![event],
            };
        }
    }
    if !month.events.is_empty() {
        months.push(month);
    }

    months
}
