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

use super::{EventParts, IcalendarSource};
use crate::model::{dancestyle::DanceStyle, event::Event};
use eyre::{Report, bail};

pub struct Burghausen;

impl IcalendarSource for Burghausen {
    const URLS: &'static [&'static str] = &[
        "https://calendar.google.com/calendar/ical/6111cddd254168febe2a853852ea97fcaaa6accfce277ef842d66db6f585bd5b%40group.calendar.google.com/public/basic.ics",
    ];
    const DEFAULT_ORGANISATION: &'static str = "Balfolk Burghausen";

    fn workshop(_parts: &EventParts) -> bool {
        false
    }

    fn social(parts: &EventParts) -> bool {
        parts.summary.to_lowercase().contains("tanzabend")
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        let location_parts = parts.location_parts.as_ref().unwrap();
        let postcode_and_city = match location_parts.len() {
            2 | 3 => &location_parts[1],
            4 => &location_parts[2],
            _ => bail!("Unexpected location format {:?}", location_parts),
        };
        let city = postcode_and_city.split_once(' ').unwrap().1.to_string();
        Ok(Some(("Germany".to_string(), None, city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        if event.name == "Balfolk-Session" {
            return None;
        }

        event
            .links
            .insert(0, "https://www.balfolk-burghausen.de".to_string());
        if event.price.is_none()
            && event
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("Unkostenbeitrag erbeten")
        {
            event.price = Some("donation".to_string());
        }
        Some(event)
    }
}
