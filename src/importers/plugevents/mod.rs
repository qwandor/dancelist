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

use self::types::{Event, EventFormat, EventList};
use crate::model::{
    dancestyle::DanceStyle,
    event::{self, EventTime},
    events::Events,
};
use eyre::{eyre, Report};
use std::fs::read_to_string;

pub async fn events() -> Result<Vec<Event>, Report> {
    let json = read_to_string("plugevents.json")?;
    let events: EventList = serde_json::from_str(&json)?;
    Ok(events.events)
}

pub async fn import_events() -> Result<Events, Report> {
    let events = events().await?;
    let style = DanceStyle::Balfolk;

    Ok(Events {
        events: events
            .iter()
            .filter_map(|event| convert(event, style).transpose())
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event, style: DanceStyle) -> Result<Option<event::Event>, Report> {
    let Some(venue_locale) = &event.venue_locale else {
        eprintln!("Event \"{}\" has no venueLocale, skipping.", event.name);
        return Ok(None);
    };
    let locale_parts: Vec<_> = venue_locale.split(", ").collect();
    let country = locale_parts
        .last()
        .ok_or_else(|| eyre!("venueLocale only has one part: \"{}\"", venue_locale))?
        .to_string();

    let city = if locale_parts.len() > 3 {
        locale_parts[1]
    } else {
        locale_parts[0]
    }
    .to_string();

    let (workshop, social) = match event.event_format {
        EventFormat::Class => (true, false),
        EventFormat::Fest => (true, true),
        EventFormat::Party => (false, true),
    };

    Ok(Some(event::Event {
        name: event.name.clone(),
        details: Some(event.description.clone()),
        links: vec![event.plug_url.clone()],
        time: EventTime::DateTime {
            start: event
                .start_date_time_iso
                .with_timezone(&event.timezone)
                .fixed_offset(),
            end: event
                .end_date_time_iso
                .with_timezone(&event.timezone)
                .fixed_offset(),
        },
        country,
        state: None,
        city,
        styles: vec![style],
        workshop,
        social,
        bands: vec![],
        callers: vec![],
        price: format_price(event),
        organisation: event.published_by_name.clone(),
        cancelled: false,
        source: None,
    }))
}

fn format_price(event: &Event) -> Option<String> {
    event.price_display.as_ref().map(|price| {
        let mut price = price.replace(" ", "");
        let currency = price.chars().next().unwrap();
        if "$£€".contains(currency) {
            price = price.replace("-", &format!("-{}", currency));
        }
        price
    })
}
