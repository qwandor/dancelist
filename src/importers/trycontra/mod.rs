// Copyright 2023 the dancelist authors.
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

mod types;

use self::types::Event;
use crate::model::{dancestyle::DanceStyle, event, events::Events};
use chrono::NaiveDate;
use eyre::Report;
use log::warn;
use regex::Regex;

const STATES: [&str; 50] = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
    "WI", "WY",
];
const DATE_FORMAT: &str = "%m/%d/%Y";

async fn events() -> Result<Vec<Event>, Report> {
    let json =
        reqwest::get("https://raw.githubusercontent.com/jeffkaufman/trycontra/master/events.json")
            .await?
            .text()
            .await?;
    let events: Vec<Event> = serde_json::from_str(&json)?;
    Ok(events)
}

pub async fn import_events() -> Result<Events, Report> {
    let events = events().await?;

    Ok(Events {
        events: events
            .iter()
            .filter_map(|event| convert(event).transpose())
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event) -> Result<Option<event::Event>, Report> {
    let name = Regex::new(r"(( at|:) [a-zA-Z ]+| \([a-zA-Z ]+\))$")
        .unwrap()
        .replace(&event.name, "")
        .into_owned();

    let city;
    let state;
    let country;
    if event.location == "St Croix" {
        city = event.location.to_owned();
        state = None;
        country = "US Virgin Islands".to_owned();
    } else {
        let Some((city_s, state_or_country)) = event.location.rsplit_once(' ') else {
            warn!(
                "Invalid location '{}' for event {}",
                event.location, event.name
            );
            return Ok(None);
        };
        city = city_s.to_owned();
        if STATES.contains(&state_or_country) {
            state = Some(state_or_country.to_owned());
            country = "USA".to_owned();
        } else {
            state = None;
            country = state_or_country.to_owned();
        }
    }

    let links;
    let cancelled;
    if event.url.contains("CANCELLED") {
        cancelled = true;
        links = vec![];
    } else {
        cancelled = false;
        links = vec![event.url.to_owned()];
    }

    let Some(date_end) = &event.date_end else {
        return Ok(None);
    };
    let time = event::EventTime::DateOnly {
        start_date: NaiveDate::parse_from_str(&event.date, DATE_FORMAT)?,
        end_date: NaiveDate::parse_from_str(date_end, DATE_FORMAT)?,
    };

    Ok(Some(event::Event {
        name,
        details: None,
        links,
        time,
        country,
        state,
        city,
        styles: vec![DanceStyle::Contra],
        workshop: true,
        social: true,
        bands: event.bands.to_owned(),
        callers: event.callers.to_owned(),
        price: None,
        organisation: Some("TryContra".to_string()),
        cancelled,
        source: None,
    }))
}
