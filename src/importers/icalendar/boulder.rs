// Copyright 2024 the dancelist authors.
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

use super::{EventParts, IcalendarSource};
use crate::model::{
    dancestyle::DanceStyle,
    event::{Event, EventTime},
};
use chrono::TimeDelta;
use eyre::{eyre, Report};

pub struct Boulder;

impl IcalendarSource for Boulder {
    const URLS: &'static [&'static str] = &[
        "https://boulderdance.org/events/?ical=1",
        "https://boulderdance.org/events/list/?ical=1",
        "https://boulderdance.org/events/list/page/2/?ical=1",
    ];

    const DEFAULT_ORGANISATION: &'static str = "Boulder Dance Coalition";

    fn workshop(parts: &EventParts) -> bool {
        let description_lower = parts.description.to_lowercase();
        parts.summary == "Scottish Country Dance"
            || parts.summary == "Scandinavian Weekly Dance"
            || parts.summary == "Boulder Scandinavian Weekend"
            || parts.summary.contains("Class")
            || (description_lower.contains("lesson") && !description_lower.contains("no lesson"))
    }

    fn social(parts: &EventParts) -> bool {
        parts.summary == "Boulder Scandinavian Weekend"
            || parts.summary == "Scandinavian Christmas Dance"
            || parts.summary == "Scandinavian Monthly Dance"
            || parts.summary == "Scandinavian Weekly Dance"
            || parts.summary.contains("Contra")
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        let mut styles = vec![];
        if parts.summary.contains("Scottish Country Dance") {
            styles.push(DanceStyle::ScottishCountryDance);
        }
        if parts.summary.contains("Scandinavian") {
            styles.push(DanceStyle::Scandinavian);
        }
        if parts.summary.contains("Contra") {
            styles.push(DanceStyle::Contra);
        }
        styles
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        let location_parts = parts
            .location_parts
            .as_ref()
            .ok_or_else(|| eyre!("Event missing location."))?;
        if location_parts.len() < 3 {
            return Ok(None);
        }
        let mut country = location_parts[location_parts.len() - 1].to_owned();
        if country == "United States" {
            country = "USA".to_owned();
        }
        let (state, city) = if location_parts[location_parts.len() - 2].len() == 2 {
            (
                Some(location_parts[location_parts.len() - 2].to_owned()),
                location_parts[location_parts.len() - 3].to_owned(),
            )
        } else {
            (
                Some(location_parts[location_parts.len() - 3].to_owned()),
                location_parts[location_parts.len() - 4].to_owned(),
            )
        };
        Ok(Some((country, state, city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        if let Some(organisation) = &mut event.organisation {
            *organisation = organisation.replace("%20", " ");
            if organisation == "Scandinavian - Monday Dance"
                || organisation == "Scandinavian - Boulder Scandinavian Dancers"
            {
                *organisation = "Boulder Scandinavian Dancers".to_string();
            }
        }
        match event.name.as_str() {
            "Boulder Scottish Country Dance" | "Scottish Country Dance" => {
                event.name = "Scottish Country Dance".to_string();
                if let EventTime::DateTime { start, end } = &mut event.time {
                    if end == start {
                        *end = *start + TimeDelta::try_hours(2).unwrap();
                    }
                }
                event
                    .links
                    .insert(0, "https://scdcolorado.org/Weekly_Classes.html".to_string());
                if event.price.is_none() {
                    event.price = Some("$5".to_string());
                }
            }
            "Scandinavian Weekly Dance" | "Scandinavian Basics Class" => {
                if event.price.is_none() {
                    event.price = Some("$7".to_string());
                }
            }
            _ => {}
        }
        Some(event)
    }
}
