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

pub mod balfolknl;
pub mod cdss;
pub mod folkbalbende;
mod icalendar_utils;
pub mod webfeet;

use crate::{github::choose_file_for_event, model::events::Events};
use eyre::Report;

/// Attempts to add all the given events from an import to existing files.
pub fn add_all(existing_events: &Events, new_events: &Events) -> Result<(), Report> {
    for event in &new_events.events {
        println!("Trying to merge {}", event.name);
        match choose_file_for_event(existing_events, event) {
            Ok(chosen_file) => println!("  {}", chosen_file),
            Err(_) => println!("  Duplicate"),
        }
    }
    Ok(())
}
