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

use super::icalendar_utils::{get_time, unescape};
use crate::model::{dancestyle::DanceStyle, event, events::Events};
use eyre::{eyre, Report, WrapErr};
use icalendar::{Calendar, CalendarComponent, Component, Event, EventLike};
use regex::Regex;
use std::cmp::{max, min};

const BANDS: [&str; 14] = [
    "Bare Necessities",
    "Ben Bolker and Susanne Maziarz",
    "Bunny Bread Bandits",
    "Cojiro",
    "Elixir",
    "Eloise & Co.",
    "The Free Raisins",
    "Lone Star Pirates",
    "Playing with Fyre",
    "SpringTide",
    "Starling",
    "Stomp Rocket",
    "Supertrad",
    "Take a Dance",
];
const CALLERS: [&str; 19] = [
    "Alan Rosenthal",
    "Alice Raybourn",
    "Cathy Campbell",
    "Dave Berman",
    "Don Heinold",
    "Gaye Fifer",
    "George Marshall",
    "Janine Smith",
    "Joanna Reiner Wilkinson",
    "Lindsey Dono",
    "Lisa Greenleaf",
    "Liz Nelson",
    "Michael Karchar",
    "Nils Fredland",
    "Steph West",
    "Steve Zakon-Anderson",
    "Tara Bolker",
    "Walter Zagorski",
    "Will Mentor",
];

pub async fn import_events() -> Result<Events, Report> {
    let calendar = reqwest::get("https://cdss.org/events/list/?ical=1")
        .await?
        .text()
        .await?
        .parse::<Calendar>()
        .map_err(|e| eyre!("Error parsing iCalendar file: {}", e))?;
    Ok(Events {
        events: calendar
            .iter()
            .filter_map(|component| {
                if let CalendarComponent::Event(event) = component {
                    convert(event).transpose()
                } else {
                    None
                }
            })
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event) -> Result<Option<event::Event>, Report> {
    let url = event
        .get_url()
        .ok_or_else(|| eyre!("Event {:?} missing url.", event))?
        .to_owned();
    let summary = event
        .get_summary()
        .ok_or_else(|| eyre!("Event {:?} missing summary.", event))?;
    let description = unescape(
        event
            .get_description()
            .ok_or_else(|| eyre!("Event {:?} missing description.", event))?,
    );
    let time = get_time(event)?;

    let categories = event
        .property_value("CATEGORIES")
        .ok_or_else(|| eyre!("Event {:?} missing categories.", event))?
        .split(",")
        .collect::<Vec<_>>();

    let mut styles = Vec::new();
    let summary_lowercase = summary.to_lowercase();
    if categories.contains(&"Online Event") || summary_lowercase.contains("online") {
        return Ok(None);
    }
    if categories.contains(&"Contra Dance") {
        styles.push(DanceStyle::Contra);
    }
    if categories.contains(&"English Country Dance") {
        styles.push(DanceStyle::EnglishCountryDance);
    }
    if summary_lowercase.contains("bal folk") || summary_lowercase.contains("balfolk") {
        styles.push(DanceStyle::Balfolk);
    }
    if styles.is_empty() {
        return Ok(None);
    }

    let location = event
        .get_location()
        .ok_or_else(|| eyre!("Event {:?} missing location.", event))?;
    let location_parts = location.split("\\, ").collect::<Vec<_>>();
    let mut country = location_parts[location_parts.len() - 1].to_owned();
    if country == "United States" {
        country = "USA".to_owned();
    } else if country == "United Kingdom" {
        country = "UK".to_owned();
    }
    let (state, city) = if ["Canada", "USA"].contains(&country.as_str()) {
        (
            Some(location_parts[location_parts.len() - 3].to_owned()),
            location_parts[location_parts.len() - 4].to_owned(),
        )
    } else {
        (None, location_parts[location_parts.len() - 3].to_owned())
    };

    let organisation = Some(
        if let Some(organiser) = event.properties().get("ORGANIZER") {
            let organiser_name = organiser
                .params()
                .get("CN")
                .ok_or_else(|| eyre!("Event {:?} missing organiser name", event))?
                .value();
            organiser_name[1..organiser_name.len() - 1].to_owned()
        } else {
            "CDSS".to_owned()
        },
    );

    // Figure out price from description.
    let price_regex = Regex::new(r"\$([0-9]+)").unwrap();
    let mut min_price = u32::MAX;
    let mut max_price = u32::MIN;
    for capture in price_regex.captures_iter(&description) {
        let price: u32 = capture
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .wrap_err("Invalid price")?;
        min_price = min(price, min_price);
        max_price = max(price, max_price);
    }
    let price = if min_price == u32::MAX {
        None
    } else if min_price == max_price {
        Some(format!("${}", min_price))
    } else {
        Some(format!("${}-${}", min_price, max_price))
    };

    let bands = BANDS
        .iter()
        .filter_map(|band| {
            if description.contains(band) || summary.contains(band) {
                Some(band.to_string())
            } else {
                None
            }
        })
        .collect();
    let callers = CALLERS
        .iter()
        .filter_map(|caller| {
            if description.contains(caller) || summary.contains(caller) {
                Some(caller.to_string())
            } else {
                None
            }
        })
        .collect();

    let description_lower = description.to_lowercase();
    let workshop = description_lower.contains("lesson")
        || description_lower.contains("skills session")
        || description_lower.contains("workshops")
        || description_lower.contains("beginners workshop");

    let details = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    Ok(Some(event::Event {
        name: summary.to_owned(),
        details,
        links: vec![url],
        time,
        country,
        state,
        city,
        styles,
        workshop,
        social: true,
        bands,
        callers,
        price,
        organisation,
        cancelled: false,
        source: None,
    }))
}
