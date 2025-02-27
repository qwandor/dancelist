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

pub struct Marburg;

impl IcalendarSource for Marburg {
    const URLS: &'static [&'static str] = &[
        "https://www.folkclub-marburg.de/wp/wp-content/plugins/bal-folk-eventlist/ical/Bal-Folk-Marburg-Calendar.ics",
    ];
    const DEFAULT_ORGANISATION: &'static str = "Folkclub Marburg";
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/Berlin");

    fn workshop(parts: &EventParts) -> bool {
        let description_lower = parts.description.to_lowercase();
        description_lower.contains("tanzworkshop")
            || description_lower.contains("tanz-workshop")
            || description_lower.contains("tanzeinführungsworkshop")
            || description_lower.contains("tanzeinführung")
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("Germany".to_string(), None, "Marburg".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        if let Some((_, name)) = event.name.split_once(" – ") {
            event.name = name.to_owned();
        }
        if let Some(details) = &mut event.details {
            *details = details
                .split("Please accept YouTube cookies to play this video. By accepting you will be accessing content from YouTube, a service provided by an external third party.  YouTube privacy policy  If you accept this notice, your choice will be saved and the page will refresh.  Accept YouTube Content")
                .next()
                .unwrap()
                .trim()
                .to_owned();
        }
        Some(event)
    }
}
