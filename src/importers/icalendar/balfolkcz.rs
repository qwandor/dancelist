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
use crate::{
    importers::{bands::BANDS, lowercase_matches},
    model::{dancestyle::DanceStyle, event::Event},
};
use eyre::Report;

pub struct BalfolkCz;

impl IcalendarSource for BalfolkCz {
    const URLS: &'static [&'static str] = &[
        "https://balfolk.cz/?plugin=all-in-one-event-calendar&controller=ai1ec_exporter_controller&action=export_events&no_html=true",
    ];
    const DEFAULT_ORGANISATION: &'static str = "Balfolk.cz";

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        summary_lower.contains("biomechanika kruhových tanců")
            || summary_lower.contains("párové tance v balfolku")
            || summary_lower.contains("prague balfolk weekend")
            || summary_lower.contains("workshop")
    }

    fn social(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        summary_lower.contains("andělu")
            || summary_lower.contains("prague balfolk weekend")
            || summary_lower.contains("#si_zatancovat")
            || summary_lower.contains("vyšehradě")
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        if parts.summary.contains("do Brna") {
            Ok(Some(("Czechia".to_string(), None, "Brno".to_string())))
        } else {
            Ok(Some(("Czechia".to_string(), None, "Prague".to_string())))
        }
    }

    fn fixup(mut event: Event) -> Option<Event> {
        // Override bands from standard importer, as description is bogus.
        let name_lower = event.name.to_lowercase();
        event.bands = lowercase_matches(&BANDS, &name_lower, "");
        event.details = None;

        Some(event)
    }
}
