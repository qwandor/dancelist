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

use super::event::Event;
use chrono::Utc;
use eyre::{bail, Report, WrapErr};
use log::trace;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Events {
    pub events: Vec<Event>,
}

impl Events {
    pub fn load(directory: &Path) -> Result<Self, Report> {
        let mut events = vec![];
        for entry in read_dir(directory)? {
            let filename = entry?.path();
            if filename.extension() != Some(OsStr::new("yaml")) {
                trace!("Not reading events from {:?}", filename);
                continue;
            }
            trace!("Reading events from {:?}", filename);
            let contents =
                read_to_string(&filename).wrap_err_with(|| format!("Reading {:?}", filename))?;
            let file_events = serde_yaml::from_str::<Events>(&contents)
                .wrap_err_with(|| format!("Reading {:?}", filename))?
                .events;
            for event in &file_events {
                let problems = event.validate();
                if !problems.is_empty() {
                    bail!(
                        "Problems with event '{}' in {:?}: {:?}",
                        event.name,
                        filename,
                        problems
                    );
                }
            }
            events.extend(file_events);
        }
        Ok(Self { events })
    }

    /// Get all events finishing on or after the present day.
    pub fn future(&self) -> Vec<&Event> {
        let today = Utc::now().naive_utc().date();
        self.events
            .iter()
            .filter(|event| event.end_date >= today)
            .collect()
    }

    /// Gets all bands who play for at least one event, in alphabetical order.
    pub fn bands(&self) -> Vec<String> {
        let mut bands: Vec<String> = self
            .events
            .iter()
            .flat_map(|event| event.bands.clone())
            .collect();
        bands.sort();
        bands.dedup();
        bands
    }

    /// Gets all callers who call for at least one event, in alphabetical order.
    pub fn callers(&self) -> Vec<String> {
        let mut callers: Vec<String> = self
            .events
            .iter()
            .flat_map(|event| event.callers.clone())
            .collect();
        callers.sort();
        callers.dedup();
        callers
    }

    /// Gets all dance organisations, in alphabetical order.
    pub fn organisations(&self) -> Vec<String> {
        let mut organisations: Vec<String> = self
            .events
            .iter()
            .filter_map(|event| event.organisation.clone())
            .collect();
        organisations.sort();
        organisations.dedup();
        organisations
    }
}
