// Copyright 2026 the dancelist authors.
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

pub struct FridayFolk;

impl IcalendarSource for FridayFolk {
    const URLS: &'static [&'static str] = &["https://fridayfolk.org.uk/ffprog.ics"];
    const DEFAULT_ORGANISATION: &'static str = "Friday Folk";

    fn workshop(_parts: &EventParts) -> bool {
        false
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::EnglishCountryDance]
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("UK".to_string(), None, "St Albans".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        if event.name.contains("Closed") {
            return None;
        } else if event.name.starts_with("Friday Folk: ") {
            let (_, rest) = event.name.split_once(": ").unwrap();
            event.details = Some(if let Some(details) = &event.details {
                format!("{details}: {rest}")
            } else {
                rest.to_owned()
            });
            if event
                .name
                .starts_with("Friday Folk: Saturday Special Dance")
            {
                event.name = "Friday Folk Saturday Special".to_string();
            } else {
                event.name = "Friday Folk".to_string();
            }
        }

        for link in &mut event.links {
            *link = link.replace("http:", "https:");
        }

        Some(event)
    }
}
