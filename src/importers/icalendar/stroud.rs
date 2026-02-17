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
use regex::Regex;

pub struct Stroud;

impl IcalendarSource for Stroud {
    const URLS: &'static [&'static str] = &[
        "https://stroud.dance/ceilidh/events/all.ics",
        "https://balfolkstroud.uk/events/all.ics",
    ];
    const DEFAULT_ORGANISATION: &'static str = "New Stroud Ceilidhs";
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/London");

    fn workshop(_parts: &EventParts) -> bool {
        false
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        let summary_lower = parts.summary.to_lowercase();
        let mut styles = Vec::new();
        if summary_lower.contains("balfolk") | summary_lower.contains("french dance") {
            styles.push(DanceStyle::Balfolk);
        }
        if summary_lower.contains("ceilidh") || styles.is_empty() {
            styles.push(DanceStyle::EnglishCeilidh);
        }
        styles
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("UK".to_string(), None, "Stroud".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        let organisation_regex = Regex::new(r"presented by (.+) Â£").unwrap();
        if let Some(capture) = organisation_regex.captures(&event.name) {
            event.organisation = Some(capture.get(1).unwrap().as_str().to_owned());
        }
        if event.details.is_none() {
            event.details = Some(event.name.clone());
            event.name = match event.organisation.as_deref().unwrap() {
                "New Stroud Ceilidhs" => "Stroud Ceilidh",
                "Balfolk Stroud/New Stroud Ceilidhs" | "New Stroud Ceilidhs/Balfolk Stroud" => {
                    "Balfolk Stroud"
                }
                organisation => organisation,
            }
            .to_owned();
        }
        Some(event)
    }
}
