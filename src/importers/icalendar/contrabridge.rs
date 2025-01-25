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

pub struct Contrabridge;

impl IcalendarSource for Contrabridge {
    const URLS: &'static [&'static str] = &["https://contrabridge.org/events.ics"];
    const DEFAULT_ORGANISATION: &'static str = "Contrabridge";

    fn workshop(_parts: &EventParts) -> bool {
        true
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Contra]
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("UK".to_string(), None, "Cambridge".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        event
            .links
            .insert(0, "https://contrabridge.org/events/".to_string());
        event.name = "Contrabridge".to_string();
        if event.price.is_none() {
            event.price = Some("£7-£15".to_string());
        }
        Some(event)
    }
}
