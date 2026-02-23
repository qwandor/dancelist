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
use eyre::Report;

/// Importer for 7schritt.de in Freiburg im Breisgau.
pub struct Freiburg;

impl IcalendarSource for Freiburg {
    const URLS: &'static [&'static str] = &["https://7schritt.de/kalender/liste/?ical=1"];
    const DEFAULT_ORGANISATION: &'static str = "Siebenschritt e.V.";

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        summary_lower.contains("montagstanzen")
            || summary_lower.contains("monstagstanzen")
            || description_lower.contains("tanzkurs")
            || description_lower.contains("workshop")
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        let mut styles = Vec::new();

        if summary_lower.contains("skandinavische") || description_lower.contains("skandinavische")
        {
            styles.push(DanceStyle::Scandinavian)
        }
        if description_lower.contains("bal folk") || styles.is_empty() {
            styles.push(DanceStyle::Balfolk);
        }

        styles.sort();
        styles.dedup();
        styles
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some((
            "Germany".to_string(),
            None,
            "Freiburg i. Br".to_string(),
        )))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        match event.name.as_str() {
            "Montagstanzen: Fortgeschrittene" | "Monstagstanzen: Fortgeschrittene" => {
                event.name = "Monday Dancing: Advanced".to_string();
                event.price = Some("donation".to_string());
            }
            "Montagstanzen:  Anfänger"
            | "Montagstanzen: Anfänger"
            | "Montagstanz: Anfänger"
            | "Monstagstanzen: Anfänger" => {
                event.name = "Monday Dancing: Beginners".to_string();
                event.price = Some("donation".to_string());
            }
            _ => {}
        }
        Some(event)
    }
}
