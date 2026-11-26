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

pub struct Bls;

impl IcalendarSource for Bls {
    const URLS: &'static [&'static str] = &["https://www.bls.no/aktivitetar/dans/?&ical=1"];
    const DEFAULT_ORGANISATION: &'static str = "BLS";

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        let description_lower = parts.description.to_lowercase();
        summary_lower.contains("kurs") || description_lower.contains("kurs")
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Scandinavian]
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("Norway".to_string(), None, "Oslo".to_string())))
    }

    fn fixup(event: Event) -> Option<Event> {
        Some(event)
    }
}
