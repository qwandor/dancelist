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
use eyre::{eyre, Report};

pub struct CeilidhClub;

impl IcalendarSource for CeilidhClub {
    const URL: &'static str = "https://ceilidhclub.com/tickets/?ical=1";
    const DEFAULT_ORGANISATION: &'static str = "London Ceilidh Club";

    fn workshop(_parts: &EventParts) -> bool {
        false
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::ScottishCeilidh]
    }

    fn location(
        location_parts: &Option<Vec<String>>,
    ) -> Result<Option<(String, Option<String>, String)>, Report> {
        let location_parts = location_parts
            .as_ref()
            .ok_or_else(|| eyre!("Event missing location."))?;
        if location_parts.len() < 3 {
            return Ok(None);
        }
        let mut country = location_parts[location_parts.len() - 1].to_owned();
        if country == "United Kingdom" {
            country = "UK".to_owned();
        }
        let city = location_parts[location_parts.len() - 3].to_owned();

        Ok(Some((country, None, city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        if event.name.starts_with("London Ceilidh Club") {
            event.name = "Ceilidh Club".to_string();
        } else if event.name.starts_with("Scotch Whisky Tasting") {
            return None;
        }
        event.details = None;
        Some(event)
    }
}
