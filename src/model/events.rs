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

use super::event::{Event, Filters};
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
    pub fn cloned(events: Vec<&Event>) -> Self {
        Self {
            events: events.into_iter().cloned().collect(),
        }
    }

    /// Load events from all YAML files in the given directory.
    pub fn load_directory(directory: &Path) -> Result<Self, Report> {
        let mut events = vec![];
        for entry in read_dir(directory)? {
            let filename = entry?.path();
            if filename.extension() != Some(OsStr::new("yaml")) {
                trace!("Not reading events from {:?}", filename);
                continue;
            }
            let file_events = Self::load_file(&filename)?;
            events.extend(file_events.events);
        }
        Ok(Self { events })
    }

    /// Load events from the given YAML file.
    pub fn load_file(filename: &Path) -> Result<Self, Report> {
        trace!("Reading events from {:?}", filename);
        let contents =
            read_to_string(&filename).wrap_err_with(|| format!("Reading {:?}", filename))?;
        let events = serde_yaml::from_str::<Events>(&contents)
            .wrap_err_with(|| format!("Reading {:?}", filename))?;
        for event in &events.events {
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
        Ok(events)
    }

    /// Get all events matching the given filters.
    pub fn matching(&self, filters: &Filters) -> Vec<&Event> {
        let today = Utc::now().naive_utc().date();
        self.events
            .iter()
            .filter(|event| filters.matches(event, today))
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::model::{dancestyle::DanceStyle, event::DateFilter};
    use chrono::NaiveDate;

    #[test]
    fn filter_past() {
        let past_event = Event {
            name: "Past".to_string(),
            start_date: NaiveDate::from_ymd(1000, 1, 1),
            end_date: NaiveDate::from_ymd(1000, 1, 1),
            details: None,
            links: vec![],
            country: "Test".to_string(),
            city: "Test".to_string(),
            styles: vec![DanceStyle::Playford],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
        };
        let future_event = Event {
            name: "Future".to_string(),
            start_date: NaiveDate::from_ymd(3000, 1, 1),
            end_date: NaiveDate::from_ymd(3000, 1, 1),
            details: None,
            links: vec![],
            country: "Test".to_string(),
            city: "Test".to_string(),
            styles: vec![DanceStyle::Playford],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
        };
        let events = Events {
            events: vec![past_event.clone(), future_event.clone()],
        };

        assert_eq!(events.matching(&Filters::default()), vec![&future_event]);
        assert_eq!(
            events.matching(&Filters {
                date: DateFilter::Past,
                ..Filters::default()
            }),
            vec![&past_event]
        );
        assert_eq!(
            events.matching(&Filters {
                date: DateFilter::All,
                ..Filters::default()
            }),
            vec![&past_event, &future_event]
        );
    }
}
