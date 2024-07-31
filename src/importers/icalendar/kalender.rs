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

const GERMANY_CITIES: [&str; 10] = [
    "Ehningen",
    "Freiburg",
    "Frickingen",
    "Frommern",
    "Gomaringen",
    "Heiligenberg",
    "Karlsruhe",
    "Kirchheim",
    "Nürtingen",
    "Tübingen",
];

/// Importer for Balfolk-Orga-Kalender.
pub struct Kalender;

impl IcalendarSource for Kalender {
    const URL: &'static str = "https://export.kalender.digital/ics/0/574d155c91900caea879/balfolk-orga-kalender.ics?past_months=3&future_months=36";
    const DEFAULT_ORGANISATION: &'static str = "Balfolk-Orga-Kalender";
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/Berlin");

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
        let mut styles = vec![];
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        if summary_lower.contains("balfolk")
            || summary_lower.contains("bal folk")
            || description_lower.contains("balfolk")
        {
            styles.push(DanceStyle::Balfolk);
        }
        styles
    }

    fn location(
        location_parts: &Option<Vec<String>>,
    ) -> Result<Option<(String, Option<String>, String)>, Report> {
        if let Some(location_parts) = location_parts {
            for city in &GERMANY_CITIES {
                if location_parts.iter().any(|part| part.contains(city)) {
                    return Ok(Some((city.to_string(), None, "Germany".to_string())));
                }
            }
        }
        Ok(None)
    }

    fn fixup(mut event: Event) -> Option<Event> {
        event.name = shorten_name(&event.name);

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
