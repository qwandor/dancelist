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

use super::{lowercase_matches, EventParts};
use crate::{
    importers::BANDS,
    model::{dancestyle::DanceStyle, event::Event, events::Events},
};
use eyre::Report;

pub async fn import_events() -> Result<Events, Report> {
    super::import_events("https://spreefolk.de/?mec-ical-feed=1", convert).await
}

fn convert(parts: EventParts) -> Result<Option<Event>, Report> {
    let name = shorten_name(&parts.summary);
    let details = parts.description.trim().to_owned();
    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };
    let summary_lower = parts.summary.to_lowercase();
    let description_lower = parts.description.to_lowercase();
    let workshop = description_lower.contains("tanzworkshop") || summary_lower.contains("workshop");
    let social = !summary_lower.contains("workshop");
    let bands = lowercase_matches(&BANDS, &description_lower, &summary_lower);

    Ok(Some(Event {
        name,
        details,
        links: vec![parts.url],
        time: parts.time,
        country: "Germany".to_string(),
        state: None,
        city: "Berlin".to_string(),
        styles: vec![DanceStyle::Balfolk],
        workshop,
        social,
        bands,
        callers: vec![],
        price: None,
        organisation: Some("Spreefolk eV".to_string()),
        cancelled: false,
        source: None,
    }))
}

fn shorten_name(summary: &str) -> String {
    summary
        .split(" – ")
        .next()
        .unwrap()
        .split(" mit ")
        .next()
        .unwrap()
        .to_owned()
}
