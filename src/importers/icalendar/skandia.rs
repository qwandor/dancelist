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
use crate::model::{dancestyle::DanceStyle, event::Event};
use eyre::Report;
use log::warn;

pub struct Skandia;

impl IcalendarSource for Skandia {
    const URLS: &'static [&'static str] = &[
        "https://calendar.google.com/calendar/ical/6pl4osll43g516ccl1vd79qmm0%40group.calendar.google.com/public/basic.ics",
    ];
    const DEFAULT_ORGANISATION: &'static str = "Skandia Folkdance Society";

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        summary_lower.contains("beginner class")
            || summary_lower.contains("free class")
            || summary_lower.contains("dance class")
            || summary_lower.contains("dance review")
            || description_lower.contains("class")
            || description_lower.contains("dance workshop")
    }

    fn social(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        summary_lower.contains("free dance")
            || summary_lower.contains("gala")
            || summary_lower.contains("jullekstuga")
            || summary_lower.contains("live")
            || summary_lower.contains("midsommarfest")
            || summary_lower.contains("third friday dance")
            || summary_lower.contains("valdres week")
            || summary_lower.contains("vinterdans")
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Scandinavian]
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        let Some(location_parts) = parts.location_parts.as_ref() else {
            warn!("Event missing location on {:?}.", parts.time);
            return Ok(Some((
                "USA".to_string(),
                Some("WA".to_string()),
                "Seattle".to_string(),
            )));
        };
        if location_parts.len() == 1 && location_parts[0].to_lowercase().contains("zoom") {
            return Ok(None);
        } else if location_parts.len() < 3 {
            return Ok(Some((
                "USA".to_owned(),
                location_parts.get(1).cloned(),
                location_parts.get(0).cloned().unwrap_or_default(),
            )));
        }
        let country = location_parts[location_parts.len() - 1].to_owned();
        let mut state = location_parts[location_parts.len() - 2].to_owned();
        let city = location_parts[location_parts.len() - 3].to_owned();
        if state.starts_with("WA") {
            state = "WA".to_string();
        }
        Ok(Some((country, Some(state), city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        let name_lower = event.name.to_lowercase();
        if name_lower.contains("book kona")
            || name_lower.contains("brunch")
            || name_lower.contains("deadline")
            || name_lower.contains("house concert")
            || name_lower.contains("on zoom")
            || name_lower.contains("photo class")
            || name_lower.contains("visas")
            || name_lower.contains("work party")
            || event.start_year() < 2024
        {
            return None;
        }
        if name_lower.contains("no class")
            || event
                .details
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains("no class")
        {
            event.cancelled = true;
        }
        if name_lower.contains("free") && event.price.is_none() {
            event.price = Some("free".to_string());
        }
        match event.name.as_str() {
            "Beginner Scandinavia dance classes near UW" => {
                event.name = "Beginner Scandinavian dance classes".to_string();
            }
            "Nordic dance classes" => {
                event.name = "Nordic dance class".to_string();
            }
            _ => {}
        }

        event
            .links
            .insert(0, "https://skandia-folkdance.org/".to_string());
        Some(event)
    }
}
