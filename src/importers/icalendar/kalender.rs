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

const GERMANY_CITIES: &[(&str, &str)] = &[
    ("Bad Boll", "Bad Boll"),
    ("Balingen", "Balingen"),
    ("Ehningen", "Ehningen"),
    ("Freiburg", "Freiburg i. Br"),
    ("Frickingen", "Frickingen"),
    ("Frommern", "Frommern"),
    ("Gomaringen", "Gomaringen"),
    ("Großschönach", "Großschönach"),
    ("Heiligenberg", "Heiligenberg"),
    ("Karlsruhe", "Karlsruhe"),
    ("Kirchheim", "Kirchheim"),
    ("Marzling", "Marzling"),
    ("Nürtingen", "Nürtingen"),
    ("Rechberghausen", "Göppingen"),
    ("Roggenburg", "Roggenburg"),
    ("Schwäbisch Gmünd", "Schwäbisch Gmünd"),
    ("Stuttgart", "Stuttgart"),
    ("Tübingen", "Tübingen"),
    ("VHS Metzingen", "Metzingen"),
    ("VHS Rottenburg", "Rottenburg am Neckar"),
];

/// Importer for Balfolk-Orga-Kalender.
pub struct Kalender;

impl IcalendarSource for Kalender {
    const URLS: &'static [&'static str] = &[
        "https://export.kalender.digital/ics/0/574d155c91900caea879/balfolk-orga-kalender.ics?past_months=3&future_months=36",
    ];
    const DEFAULT_ORGANISATION: &'static str = "Balfolk-Orga-Kalender";
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/Berlin");

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        description_lower.contains("workshop")
            || description_lower.contains("tanzeinführung")
            || description_lower.contains("kurse")
            || summary_lower.contains("workshop")
    }

    fn social(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        description_lower.contains("bal mit") || !summary_lower.contains("workshop")
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        let mut styles = vec![];
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        if summary_lower.contains("balfolk")
            || summary_lower.contains("bal folk")
            || summary_lower.contains("bretagne")
            || summary_lower.contains("frankreich")
            || summary_lower.contains("minibal")
            || description_lower.contains("balfolk")
        {
            styles.push(DanceStyle::Balfolk);
        }
        if summary_lower.contains("irish set dance")
            || description_lower.contains("irisch set dance")
        {
            styles.push(DanceStyle::IrishSet);
        }
        if summary_lower.contains("skandi-ball") || summary_lower.contains("swedish") {
            styles.push(DanceStyle::Scandinavian);
        }

        if styles.is_empty() {
            vec![DanceStyle::Balfolk]
        } else {
            styles
        }
    }

    fn links(parts: &EventParts) -> Vec<String> {
        let mut links = parts.url.clone().into_iter().collect::<Vec<_>>();
        if let Some(uid) = &parts.uid {
            links.push(format!(
                "https://kalender.digital/574d155c91900caea879/event/{}",
                &uid[4..]
            ));
        }
        links
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        if let Some(location_parts) = &parts.location_parts {
            for (match_str, city) in GERMANY_CITIES {
                if location_parts.iter().any(|part| part.contains(match_str)) {
                    return Ok(Some(("Germany".to_string(), None, city.to_string())));
                }
            }
            return Ok(Some((
                "Germany".to_string(),
                None,
                location_parts[0].to_string(),
            )));
        } else {
            for (match_str, city) in GERMANY_CITIES {
                if parts.summary.contains(match_str) || parts.description.contains(match_str) {
                    return Ok(Some(("Germany".to_string(), None, city.to_string())));
                }
            }
        }
        Ok(None)
    }

    fn fixup(mut event: Event) -> Option<Event> {
        event.name = shorten_name(&event.name);
        if event.name == "KA-BALFOLK" {
            event
                .links
                .insert(0, "https://ka-balfolk.de/termine-elementor/".to_string());
            event.name = "KA-Balfolk".to_string();
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
