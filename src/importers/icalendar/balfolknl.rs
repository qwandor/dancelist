// Copyright 2022 the dancelist authors.
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
use log::{info, warn};

pub struct BalfolkNl;

impl IcalendarSource for BalfolkNl {
    const URL: &'static str = "https://www.balfolk.nl/events.ics";
    const DEFAULT_ORGANISATION: &'static str = "balfolk.nl";

    fn workshop(parts: &EventParts) -> bool {
        parts.summary.contains("Fundamentals")
            || parts.summary.contains("Basis van")
            || parts.summary.contains("beginnerslessen")
            || parts.summary.contains("danslessen")
            || parts.summary.contains("workshop")
            || parts.summary.starts_with("Folkbal Wilhelmina")
            || parts.summary.starts_with("Socialles ")
            || parts.summary.starts_with("Proefles ")
            || parts.summary.starts_with("DenneFeest")
            || parts.description.contains("Dansworkshop")
            || parts.description.contains("Workshopbeschrijving")
            || parts.description.contains("Workshop ")
            || parts.description.contains("dans uitleg")
            || parts.description.contains("dansuitleg")
            || parts.description.contains(" leren ")
            || parts.description.contains("Vooraf dansuitleg")
            || parts.description.contains("de Docent")
    }

    fn social(parts: &EventParts) -> bool {
        parts.summary.contains("Social dance")
            || parts.summary.contains("Balfolkbal")
            || parts.summary.contains("Avondbal")
            || parts.summary.contains("Bal in")
            || parts.summary.contains("Balfolk Bal")
            || parts.summary.contains("Seizoensafsluiting")
            || parts.summary.contains("Vuurbal")
            || parts.summary.contains("dansfeest")
            || parts.summary.contains("en BalFolk")
            || parts.summary.contains("Nieuwjaarsbal")
            || parts.summary.starts_with("Balfolk Deventer")
            || parts.summary.starts_with("Balfolk Groningen")
            || parts.summary.starts_with("Balfolk Wilhelmina")
            || parts.summary.starts_with("Balfolk in Kleve")
            || parts.summary.starts_with("Balfolk met ")
            || parts.summary.starts_with("BalFolk met ")
            || parts.summary.starts_with("Balfolk op de")
            || parts.summary.starts_with("BresBal")
            || parts.summary.starts_with("Dansavond")
            || parts.summary.starts_with("Drakenbal")
            || parts.summary.starts_with("Fest Noz")
            || parts.summary.starts_with("Folkwoods")
            || parts.summary.starts_with("Folkbal")
            || parts.summary.starts_with("Halloweenbal")
            || parts.summary.starts_with("Huiskamerbal")
            || parts.summary.starts_with("Socialles ")
            || parts.summary.starts_with("Superette Bal")
            || parts.summary.starts_with("Verjaardagsbal")
            || parts.summary.starts_with("Balfolk Utrecht Bal")
            || parts.summary.starts_with("Verjaardagsbal")
            || parts.summary.starts_with("Vrijdagavondbal")
            || parts.summary.starts_with("Balfolk café Nijmegen")
            || parts.summary.starts_with("DenneFeest")
            || parts.summary.starts_with("Dansavond")
            || parts.summary.starts_with("Wageningen Junushoff")
            || parts.description.contains("Bal deel")
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(
        location_parts: &Option<Vec<String>>,
    ) -> Result<Option<(String, Option<String>, String)>, Report> {
        let mut city = if let Some(location_parts) = location_parts {
            match location_parts.len() {
                8 => location_parts[3].to_string(),
                4.. => location_parts[2].to_string(),
                _ => {
                    warn!("Invalid location \"{:?}\"", location_parts,);
                    "".to_string()
                }
            }
        } else {
            warn!("Event missing location.");
            "Unknown city".to_string()
        };
        let country;
        if city == "Kleve (D)" {
            country = "Germany".to_string();
            city = "Kleve".to_string();
        } else {
            country = "Netherlands".to_string();
        }
        Ok(Some((country, None, city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        // Try to skip music workshops.
        if event.name.starts_with("Muziekstage") {
            info!("Skipping \"{}\" {}", event.name, event.links[0]);
            return None;
        }

        // Remove city from end of name.
        let raw_name = event.name.rsplitn(2, ',').last().unwrap();
        if let Some(details) = &event.details {
            // Remove name from start of details.
            let details = details
                .trim_start_matches(&format!("{}, ", raw_name))
                .trim()
                .to_owned();
            event.details = Some(details);
        }
        event.name = shorten_name(&raw_name);

        Some(event)
    }
}

fn shorten_name(raw_name: &str) -> String {
    raw_name
        .split(" met ")
        .next()
        .unwrap()
        .replace(" (Rotterdam)", "")
        .replace(" (D) bij Nijmegen", "")
        .replace(" - ", " — ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_location() {
        assert_eq!(
            BalfolkNl::location(&None).unwrap(),
            Some(("Netherlands".to_string(), None, "Unknown city".to_string()))
        );
        assert_eq!(
            BalfolkNl::location(&Some(vec![
                "City".to_string(),
                "postcode".to_string(),
                "Nederland".to_string(),
            ]))
            .unwrap(),
            Some(("Netherlands".to_string(), None, "".to_string()))
        );
        assert_eq!(
            BalfolkNl::location(&Some(vec![
                "Balfolk Zeist".to_string(),
                "Thorbeckelaan 5".to_string(),
                "Zeist".to_string(),
                "3705 KJ".to_string(),
                "Nederland".to_string(),
            ]))
            .unwrap(),
            Some(("Netherlands".to_string(), None, "Zeist".to_string()))
        );
    }
}
