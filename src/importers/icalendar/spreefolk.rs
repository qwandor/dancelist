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

pub struct Spreefolk;

impl IcalendarSource for Spreefolk {
    const URL: &'static str = "https://spreefolk.de/?mec-ical-feed=1";
    const DEFAULT_ORGANISATION: &'static str = "Spreefolk eV";

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        description_lower.contains("tanzworkshop")
            || description_lower.contains("tanz-workshop")
            || description_lower.contains("tanzeinführungsworkshop")
            || description_lower.contains("tanzeinführung")
            || summary_lower.contains("workshop")
    }

    fn social(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        !summary_lower.contains("workshop")
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        if parts.summary.contains("Mitgliederversammlung") {
            vec![]
        } else {
            vec![DanceStyle::Balfolk]
        }
    }

    fn location(
        _location_parts: &Option<Vec<String>>,
        _url: &str,
    ) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("Germany".to_string(), None, "Berlin".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        event.name = shorten_name(&event.name);

        event.name = event
            .name
            .replace("Folk & Tanz am", "Folk & Dance at")
            .to_owned();
        if let Some(details) = &mut event.details {
            *details = details
                .split("\n\n\n\n\n\n\nJavaScript ist in deinem Browser deaktiviert.")
                .next()
                .unwrap()
                .split("\n \n\n\n\nJavaScript ist in deinem Browser deaktiviert.")
                .next()
                .unwrap()
                .to_owned();
        }

        Some(event)
    }
}

fn shorten_name(name: &str) -> String {
    name.split(" – ")
        .next()
        .unwrap()
        .split(" mit ")
        .next()
        .unwrap()
        .to_owned()
}
