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

use super::{
    icalendar::{lowercase_matches, EventParts},
    BANDS,
};
use crate::model::{dancestyle::DanceStyle, event, events::Events};
use eyre::Report;
use icalendar::Event;
use log::{info, warn};

pub async fn import_events() -> Result<Events, Report> {
    super::icalendar::import_events("https://www.balfolk.nl/events.ics", convert).await
}

fn convert(event: &Event, parts: EventParts) -> Result<Option<event::Event>, Report> {
    let summary = parts.summary.replace("\\,", ",");
    // Remove city from end of summary and use em dash where appropriate.
    let raw_name = summary.rsplitn(2, ',').last().unwrap();
    let name = raw_name
        .replace(" (Rotterdam)", "")
        .replace(" - ", " — ")
        .replace(" met Musac", "")
        .replace(" (D) bij Nijmegen", "");

    // Try to skip music workshops.
    if name.starts_with("Muziekstage") {
        info!("Skipping \"{}\" {}", name, parts.url);
        return Ok(None);
    }

    // Remove name from start of description
    let details = parts
        .description
        .trim_start_matches(&format!("{}, ", raw_name))
        .trim()
        .to_owned();
    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let mut city = if let Some(location_parts) = parts.location_parts {
        match location_parts.len() {
            8 => location_parts[3].to_string(),
            4.. => location_parts[2].to_string(),
            _ => {
                warn!(
                    "Invalid location \"{:?}\" for {}",
                    location_parts, parts.url
                );
                "".to_string()
            }
        }
    } else {
        warn!("Event {:?} missing location.", event);
        "Unknown city".to_string()
    };
    let country;
    if city == "Kleve (D)" {
        country = "Germany".to_string();
        city = "Kleve".to_string();
    } else {
        country = "Netherlands".to_string();
    }

    let workshop = name.contains("Fundamentals")
        || name.contains("Basis van")
        || name.contains("beginnerslessen")
        || name.contains("danslessen")
        || name.contains("workshop")
        || name.starts_with("Folkbal Wilhelmina")
        || name.starts_with("Socialles ")
        || name.starts_with("Proefles ")
        || name == "DenneFeest"
        || parts.description.contains("Dansworkshop")
        || parts.description.contains("Workshopbeschrijving")
        || parts.description.contains("Workshop ")
        || parts.description.contains("dans uitleg")
        || parts.description.contains("dansuitleg")
        || parts.description.contains(" leren ")
        || parts.description.contains("Vooraf dansuitleg")
        || parts.description.contains("de Docent");
    let social = name.contains("Social dance")
        || name.contains("Balfolkbal")
        || name.contains("Avondbal")
        || name.contains("Bal in")
        || name.contains("Balfolk Bal")
        || name.contains("Seizoensafsluiting")
        || name.contains("Vuurbal")
        || name.contains("dansfeest")
        || name.contains("en BalFolk")
        || name.contains("Nieuwjaarsbal")
        || name.starts_with("Balfolk Groningen")
        || name.starts_with("Balfolk Wilhelmina")
        || raw_name.starts_with("Balfolk in Kleve")
        || raw_name.starts_with("Balfolk met ")
        || raw_name.starts_with("BalFolk met ")
        || name.starts_with("Balfolk op de")
        || name.starts_with("BresBal")
        || name.starts_with("Dansavond")
        || name.starts_with("Drakenbal")
        || name.starts_with("Fest Noz")
        || name.starts_with("Folkwoods")
        || name.starts_with("Folkbal")
        || name.starts_with("Halloweenbal")
        || name.starts_with("Socialles ")
        || name.starts_with("Superette Bal")
        || name.starts_with("Verjaardagsbal")
        || name.starts_with("Balfolk Utrecht Bal")
        || name.starts_with("Verjaardagsbal")
        || name.starts_with("Vrijdagavondbal")
        || name.starts_with("Balfolk café Nijmegen")
        || name == "DenneFeest"
        || name == "Dansavond"
        || parts.description.contains("Bal deel");

    let bands = lowercase_matches(
        &BANDS,
        &parts.description.to_lowercase(),
        &raw_name.to_lowercase(),
    );

    let organisation = Some(parts.organiser.unwrap_or_else(|| "balfolk.nl".to_owned()));

    Ok(Some(event::Event {
        name,
        details,
        links: vec![parts.url],
        time: parts.time,
        country,
        state: None,
        city,
        styles: vec![DanceStyle::Balfolk],
        workshop,
        social,
        bands,
        callers: vec![],
        price: None,
        organisation,
        cancelled: false,
        source: None,
    }))
}
