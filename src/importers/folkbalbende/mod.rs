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

pub mod types;

use self::types::{Event, EventType};
use crate::{
    model::{
        dancestyle::DanceStyle,
        event::{self, EventTime},
        events::Events,
    },
    util::local_datetime_to_fixed_offset,
};
use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Europe::Brussels;
use eyre::Report;

pub async fn events() -> Result<Vec<Event>, Report> {
    let json = reqwest::get("https://folkbalbende.be/interface/events.php?start=2022-02-01&end=3000-01-01&type=ball,course,festal&image=0").await?.text().await?;
    let mut events: Vec<Event> = serde_json::from_str(&json)?;
    // Sort by ID to give a stable order.
    events.sort_by_key(|event| event.id);
    Ok(events)
}

pub async fn import_events() -> Result<Events, Report> {
    let events = events().await?;

    // Print warnings about cancelled, deleted and unchecked events.
    for event in &events {
        let dates = event
            .dates
            .iter()
            .map(|date| date.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if event.cancelled {
            eprintln!("Cancelled: {} {}", dates, event.name);
        }
        if event.deleted {
            eprintln!("Deleted: {} {}", dates, event.name);
        }
        if !event.checked {
            eprintln!("Not checked: {} {}", dates, event.name);
        }
    }

    Ok(Events {
        events: events
            .iter()
            .flat_map(|event| {
                if event.checked && !event.deleted {
                    convert(event)
                } else {
                    vec![]
                }
            })
            .collect(),
    })
}

fn convert(event: &Event) -> Vec<event::Event> {
    // Filter out "mailto:" URLs and duplicates in non-English languages.
    let mut links: Vec<String> = event
        .websites
        .iter()
        .filter_map(|website| {
            if website.url.starts_with("http")
                && !website
                    .url
                    .starts_with("https://frissefolk.be/fr/civicrm/event/info")
                && !website
                    .url
                    .starts_with("https://frissefolk.be/nl/civicrm/event/info")
            {
                Some(website.url.trim().to_owned())
            } else {
                None
            }
        })
        .collect();
    links.push(format!("https://folkbalbende.be/event/{}", event.id));
    if !event.facebook_event.is_empty() {
        links.push(event.facebook_event.trim().to_owned());
    }

    let details = format!("{:?}", event.event_type);

    let mut workshop = event.event_type == EventType::Course || !event.courses.is_empty();
    if let Some(ball) = &event.ball {
        if ball.initiation_start.is_some() || !ball.initiators.is_empty() {
            workshop = true;
        }
    }

    let social = match event.event_type {
        EventType::Course => false,
        EventType::Ball | EventType::Festival => true,
    };

    let price = if event.prices.is_empty() {
        None
    } else {
        let prices: Vec<_> = event
            .prices
            .iter()
            .filter_map(|price| {
                if price.price == 0 {
                    None
                } else {
                    Some(price.price)
                }
            })
            .collect();
        let min_price = prices.iter().min();
        let max_price = prices.iter().max();
        if let (Some(min_price), Some(max_price)) = (min_price, max_price) {
            Some(if *min_price == -1 {
                "donation".to_string()
            } else if min_price == max_price {
                format!("€{}", min_price)
            } else {
                format!("€{}-€{}", min_price, max_price)
            })
        } else {
            None
        }
    };

    let bands = if let Some(ball) = &event.ball {
        ball.performances
            .iter()
            .filter_map(|performance| {
                if performance.band.placeholder || performance.band.name.contains("Practica") {
                    None
                } else {
                    Some(performance.band.name.trim().to_owned())
                }
            })
            .collect()
    } else {
        vec![]
    };

    let organisation = if let Some(organisation) = &event.organisation {
        Some(organisation.name.to_owned())
    } else if links.iter().any(|link| link.contains("eledanse.be")) {
        Some("EléDanse ASBL".to_owned())
    } else if links
        .iter()
        .any(|link| link.contains("folknammusiquetrad.be"))
    {
        Some("Folknam Musique Trad".to_owned())
    } else if links.iter().any(|link| link.contains("tey.be")) {
        Some("Muziekclub 't Ey".to_owned())
    } else if links.iter().any(|link| link.contains("frissefolk.be")) {
        Some("Frisse Folk Vzw/asbl".to_owned())
    } else if links.iter().any(|link| link.contains("tsmiske")) {
        Some("'t Smiske".to_owned())
    } else if links.iter().any(|link| link.contains("balhalla.be")) {
        Some("Balhalla".to_owned())
    } else if links.iter().any(|link| link.contains("rzf.be")) {
        Some("RZF".to_owned())
    } else {
        None
    };

    // Find the earliest start time and latest finish time, if any.
    let mut start_times: Vec<NaiveTime> = event.courses.iter().map(|course| course.start).collect();
    let mut end_times: Vec<NaiveTime> = event.courses.iter().map(|course| course.end).collect();
    if let Some(ball) = &event.ball {
        start_times.extend(ball.initiation_start);
        end_times.extend(ball.initiation_end);
        start_times.extend(
            ball.performances
                .iter()
                .flat_map(|performance| performance.start),
        );
        end_times.extend(
            ball.performances
                .iter()
                .flat_map(|performance| performance.end),
        );
    }
    let mut start_time = start_times.into_iter().min();
    if start_time == Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()) {
        start_time = None;
    }
    let end_time = end_times.into_iter().max();

    let city = match event.location.address.city.as_str() {
        "Brugge" => "Bruges",
        "Assebroek" => "Bruges",
        "Bruxelles" => "Brussels",
        "Elsene" => "Brussels",
        "Etterbeek" => "Brussels",
        "Hombeek" => "Mechelen",
        "Ixelles" => "Brussels",
        "Jette" => "Brussels",
        "Saint-Gilles" => "Brussels",
        "Schoten" => "Antwerp",
        "Wijgmaal" => "Leuven",
        "Wilsele" => "Leuven",
        other => other,
    };

    // Remove duplicate dates.
    let mut dates = event.dates.clone();
    dates.dedup();

    dates
        .iter()
        .map(|&date| event::Event {
            name: event.name.clone(),
            details: Some(details.clone()),
            links: links.clone(),
            time: make_time(date, start_time, end_time),
            country: "Belgium".to_string(),
            state: None,
            city: city.to_owned(),
            styles: vec![DanceStyle::Balfolk],
            workshop,
            social,
            bands: bands.clone(),
            callers: vec![],
            price: price.clone(),
            organisation: organisation.clone(),
            cancelled: event.cancelled,
            source: None,
        })
        .collect()
}

fn make_time(
    date: NaiveDate,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
) -> EventTime {
    if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
        if let (Some(start), Some(end)) = (
            local_datetime_to_fixed_offset(&date.and_time(start_time), Brussels),
            local_datetime_to_fixed_offset(&date.and_time(end_time), Brussels),
        ) {
            return EventTime::DateTime { start, end };
        }
    }

    EventTime::DateOnly {
        start_date: date,
        end_date: date,
    }
}
