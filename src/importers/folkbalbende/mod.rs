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

mod bool_as_int;
mod uint_as_string;

use crate::model::{
    dancestyle::DanceStyle,
    event::{self, EventTime},
    events::Events,
};
use chrono::NaiveDate;
use eyre::Report;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: u32,
    pub name: String,
    pub recurrence: u32,
    #[serde(rename = "type")]
    pub event_type: EventType,
    #[serde(with = "bool_as_int")]
    pub cancelled: bool,
    #[serde(with = "bool_as_int")]
    pub deleted: bool,
    #[serde(with = "bool_as_int")]
    pub checked: bool,
    pub dates: Vec<NaiveDate>,
    pub location: Location,
    pub prices: Vec<Price>,
    pub thumbnail: String,
    pub reservation_type: u32,
    pub reservation_url: String,
    pub websites: Vec<Website>,
    #[serde(default)]
    pub courses: Vec<Course>,
    pub ball: Option<Ball>,
    pub facebook_event: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub tags: Vec<String>,
    pub image: String,
    pub organisation: Option<Organisation>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Ball,
    Course,
    Festival,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub address: Address,
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub id: u32,
    pub street: Option<String>,
    pub number: Option<String>,
    #[serde(rename = "zip-city")]
    pub zip_city: String,
    pub city: String,
    pub zip: String,
    pub lat: f32,
    pub lng: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub name: String,
    #[serde(with = "uint_as_string")]
    pub price: u32,
    pub free_contribution: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Website {
    pub id: u32,
    #[serde(rename = "type")]
    pub website_type: WebsiteType,
    pub url: String,
    pub icon: String,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebsiteType {
    Facebook,
    SoundCloud,
    Website,
    #[serde(rename = "Vi.be")]
    ViBe,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub id: u32,
    pub title: String,
    pub start: String,
    pub end: String,
    pub teachers: Vec<Teacher>,
    pub nl: String,
    pub fr: String,
    pub en: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Teacher {
    pub id: u32,
    pub name: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub thumbnail: Option<String>,
    pub image: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ball {
    pub initiation_start: Option<String>,
    pub initiation_end: Option<String>,
    pub initiators: Vec<String>,
    pub performances: Vec<Performance>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Performance {
    pub start: Option<String>,
    pub end: Option<String>,
    pub band: Band,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    pub id: u32,
    pub name: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub country: Country,
    #[serde(with = "bool_as_int")]
    pub placeholder: bool,
    pub websites: Vec<Website>,
    pub tags: Vec<String>,
    pub musicians: Vec<Musician>,
    pub image: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Country {
    pub code: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Musician {
    pub id: u32,
    pub name: String,
    pub instruments: String,
    pub country: Country,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Organisation {
    pub id: u32,
    pub name: String,
    pub websites: Vec<Website>,
    pub thumbnail: String,
    pub image: String,
    pub address: Option<Address>,
}

pub async fn events() -> Result<Vec<Event>, Report> {
    let json = reqwest::get("https://folkbalbende.be/interface/events.php?start=2022-02-01&end=3000-01-01&type=ball,course,festal").await?.text().await?;
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
                if event.checked && !event.cancelled && !event.deleted {
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
                Some(website.url.to_owned())
            } else {
                None
            }
        })
        .collect();
    links.push(format!("https://folkbalbende.be/event/{}", event.id));

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
            Some(if min_price == max_price {
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
                if performance.band.placeholder {
                    None
                } else {
                    Some(performance.band.name.to_owned())
                }
            })
            .collect()
    } else {
        vec![]
    };

    let organisation = if let Some(organisation) = &event.organisation {
        Some(organisation.name.to_owned())
    } else {
        None
    };

    event
        .dates
        .iter()
        .map(|&date| event::Event {
            name: event.name.clone(),
            details: Some(details.clone()),
            links: links.clone(),
            time: EventTime::DateOnly {
                start_date: date,
                end_date: date,
            },
            country: "Belgium".to_string(),
            city: event.location.address.city.clone(),
            styles: vec![DanceStyle::Balfolk],
            workshop,
            social,
            bands: bands.clone(),
            callers: vec![],
            price: price.clone(),
            organisation: organisation.clone(),
        })
        .collect()
}
