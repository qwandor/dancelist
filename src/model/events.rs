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

use super::{dancestyle::DanceStyle, event::Event, filters::Filters};
use chrono::Utc;
use eyre::{Report, WrapErr, bail};
use log::trace;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    path::Path,
};

#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
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

    /// Load events from the given file, directory or URL.
    pub async fn load_events(path_or_url: &str) -> Result<Self, Report> {
        if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            Self::load_url(path_or_url).await
        } else {
            let path = Path::new(path_or_url);
            if path.is_dir() {
                Self::load_directory(path)
            } else {
                Self::load_file(path)
            }
        }
    }

    /// Load events from all YAML files in the given directory and its subdirectories.
    pub fn load_directory(directory: &Path) -> Result<Self, Report> {
        let mut events = vec![];
        for entry in read_dir(directory)? {
            let filename = entry?.path();
            let file_events = if filename.is_dir() {
                Self::load_directory(&filename)?
            } else if filename.extension() == Some(OsStr::new("yaml")) {
                Self::load_file(&filename)?
            } else {
                trace!("Not reading events from {filename:?}");
                continue;
            };
            events.extend(file_events.events);
        }
        Ok(Self { events })
    }

    /// Load and validate events from the given YAML file.
    pub fn load_file(filename: &Path) -> Result<Self, Report> {
        trace!("Reading events from {filename:?}");
        let contents =
            read_to_string(filename).wrap_err_with(|| format!("Reading {filename:?}"))?;
        let mut events =
            Self::load_str(&contents).wrap_err_with(|| format!("Reading {filename:?}"))?;
        events.validate()?;

        // Fill in the source with the filename, if the event doesn't already have one.
        if let Some(source) = filename.to_str() {
            for event in &mut events.events {
                if event.source.is_none() {
                    event.source = Some(source.to_owned());
                }
            }
        }

        Ok(events)
    }

    /// Loads events from the given YAML file, but doesn't validate them or add sources.
    pub fn load_file_without_validation(filename: &Path) -> Result<Self, Report> {
        let contents =
            read_to_string(filename).wrap_err_with(|| format!("Reading {filename:?}"))?;
        Self::load_str(&contents).wrap_err_with(|| format!("Reading {filename:?}"))
    }

    /// Loads events from the given YAML URL and validates them.
    pub async fn load_url(url: &str) -> Result<Self, Report> {
        let contents = reqwest::get(url).await?.text().await?;
        let events = Self::load_str(&contents).wrap_err_with(|| format!("Reading {url}"))?;
        events.validate()?;
        Ok(events)
    }

    /// Loads events from the given YAML string.
    ///
    /// Doesn't validate the events.
    pub fn load_str(s: &str) -> Result<Self, Report> {
        let events = serde_yaml::from_str::<Events>(s)?;
        Ok(events)
    }

    /// Validates all events, returning an error if any of them are invalid.
    fn validate(&self) -> Result<(), Report> {
        for event in &self.events {
            let problems = event.validate();
            if !problems.is_empty() {
                bail!("Problems with event '{}': {:?}", event.name, problems);
            }
        }
        Ok(())
    }

    /// Converts the events to a YAML string.
    pub fn to_yaml_string(&self) -> Result<String, Report> {
        let yaml = serde_yaml::to_string(self)?;
        Ok(yaml.replacen(
            "---",
            "# yaml-language-server: $schema=../../events_schema.json",
            1,
        ))
    }

    /// Sorts events by start time then location.
    pub fn sort(&mut self) {
        self.events.sort_by_key(Event::date_location_sort_key);
    }

    /// Get all events matching the given filters.
    pub fn matching(&self, filters: &Filters) -> Vec<&Event> {
        let now = Utc::now();
        self.events
            .iter()
            .filter(|event| filters.matches(event, now))
            .take(filters.limit.unwrap_or(usize::MAX))
            .collect()
    }

    /// Returns the event with the given SHA-1 hash, if any.
    pub fn with_hash(&self, hash: &str) -> Option<&Event> {
        self.events.iter().find(|event| event.hash_string() == hash)
    }

    /// Gets all bands who play for at least one event, in alphabetical order.
    pub fn bands(&self) -> Vec<Band> {
        let mut bands: Vec<Band> =
            count_duplicates(self.events.iter().flat_map(|event| event.bands.clone()))
                .into_iter()
                .map(|(name, event_count)| Band { name, event_count })
                .collect();
        bands.sort();
        bands
    }

    /// Gets all callers who call for at least one event, in alphabetical order.
    pub fn callers(&self) -> Vec<Caller> {
        let mut callers: Vec<Caller> =
            count_duplicates(self.events.iter().flat_map(|event| event.callers.clone()))
                .into_iter()
                .map(|(name, event_count)| Caller { name, event_count })
                .collect();
        callers.sort();
        callers
    }

    /// Gets all dance organisations, in alphabetical order.
    pub fn organisations(&self) -> Vec<Organisation> {
        let mut organisations: Vec<Organisation> = count_duplicates(
            self.events
                .iter()
                .filter_map(|event| event.organisation.clone()),
        )
        .into_iter()
        .map(|(name, event_count)| Organisation { name, event_count })
        .collect();
        organisations.sort();
        organisations
    }

    /// Gets all cities which have dance events matching the given filters, grouped by country and
    /// possibly state, in alphabetical order.
    pub fn countries(&self, filters: &Filters) -> Vec<Country> {
        let now = Utc::now();
        let mut countries = HashMap::new();
        for event in &self.events {
            if filters.matches(event, now) {
                let (cities, states) = countries
                    .entry(event.country.to_owned())
                    .or_insert_with(|| (Vec::new(), HashMap::<String, Vec<String>>::new()));
                if let Some(state) = &event.state {
                    states
                        .entry(state.to_owned())
                        .or_default()
                        .push(event.city.to_owned());
                } else {
                    cities.push(event.city.to_owned());
                }
            }
        }
        let mut countries: Vec<_> = countries
            .into_iter()
            .map(|(country, (mut cities, states_map))| {
                cities.sort();
                cities.dedup();
                let mut states: Vec<_> = states_map
                    .into_iter()
                    .map(|(state, mut cities)| {
                        cities.sort();
                        cities.dedup();
                        State {
                            name: state,
                            cities,
                        }
                    })
                    .collect();
                states.sort();
                Country {
                    name: country,
                    states,
                    cities,
                }
            })
            .collect();
        countries.sort();
        countries
    }

    /// Gets all states which have dance events matching the given filters, in alphabetical order.
    pub fn states(&self, filters: &Filters) -> Vec<String> {
        let now = Utc::now();
        let mut states = vec![];
        for event in &self.events {
            if filters.matches(event, now) {
                if let Some(state) = event.state.as_ref() {
                    states.push(state.to_owned());
                }
            }
        }
        states.sort();
        states.dedup();
        states
    }

    /// Gets all cities which have dance events matching the given filters, in alphabetical order.
    pub fn cities(&self, filters: &Filters) -> Vec<String> {
        let now = Utc::now();
        let mut cities = vec![];
        for event in &self.events {
            if filters.matches(event, now) {
                cities.push(event.city.to_owned());
            }
        }
        cities.sort();
        cities.dedup();
        cities
    }

    /// Gets all dance styles which have events matching the given filters, in order.
    pub fn styles(&self, filters: &Filters) -> Vec<DanceStyle> {
        let now = Utc::now();
        let mut styles = vec![];
        for event in &self.events {
            if filters.matches(event, now) {
                styles.extend_from_slice(&event.styles);
            }
        }
        styles.sort();
        styles.dedup();
        styles
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Country {
    pub name: String,
    pub states: Vec<State>,
    pub cities: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct State {
    pub name: String,
    pub cities: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Band {
    pub name: String,
    pub event_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Caller {
    pub name: String,
    pub event_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Organisation {
    pub name: String,
    pub event_count: usize,
}

/// Counts the number of occurrences of duplicate items in the iterator.
fn count_duplicates(elements: impl Iterator<Item = String>) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for element in elements {
        *counts.entry(element).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::model::{dancestyle::DanceStyle, event::EventTime, filters::DateFilter};
    use chrono::NaiveDate;

    #[test]
    fn countries() {
        let london_event_1 = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "UK".to_string(),
            state: None,
            city: "London".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let london_event_2 = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "UK".to_string(),
            state: None,
            city: "London".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let oxford_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "UK".to_string(),
            state: None,
            city: "Oxford".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let amsterdam_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "Netherlands".to_string(),
            state: None,
            city: "Amsterdam".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let berkeley_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "USA".to_string(),
            state: Some("CA".to_string()),
            city: "Berkeley".to_string(),
            styles: vec![DanceStyle::Contra],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let events = Events {
            events: vec![
                oxford_event,
                london_event_1,
                amsterdam_event,
                london_event_2,
                berkeley_event,
            ],
        };
        assert_eq!(
            events.countries(&Filters::all()),
            vec![
                Country {
                    name: "Netherlands".to_string(),
                    states: vec![],
                    cities: vec!["Amsterdam".to_string()]
                },
                Country {
                    name: "UK".to_string(),
                    states: vec![],
                    cities: vec!["London".to_string(), "Oxford".to_string()]
                },
                Country {
                    name: "USA".to_string(),
                    states: vec![State {
                        name: "CA".to_string(),
                        cities: vec!["Berkeley".to_string()]
                    }],
                    cities: vec![],
                }
            ]
        );
    }

    #[test]
    fn states() {
        let oxford_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "UK".to_string(),
            state: None,
            city: "Oxford".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let berkeley_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "USA".to_string(),
            state: Some("CA".to_string()),
            city: "Berkeley".to_string(),
            styles: vec![DanceStyle::Contra],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let sf_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "USA".to_string(),
            state: Some("CA".to_string()),
            city: "San Francisco".to_string(),
            styles: vec![DanceStyle::Contra],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let boston_event = Event {
            name: "Name".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "USA".to_string(),
            state: Some("MA".to_string()),
            city: "Boston".to_string(),
            styles: vec![DanceStyle::Contra],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let events = Events {
            events: vec![oxford_event, berkeley_event, sf_event, boston_event],
        };
        assert_eq!(
            events.states(&Filters::all()),
            vec!["CA".to_string(), "MA".to_string()]
        );
    }

    #[test]
    fn filter_past() {
        let past_event = Event {
            name: "Past".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "Test".to_string(),
            state: None,
            city: "Test".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        let future_event = Event {
            name: "Future".to_string(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(3000, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(3000, 1, 1).unwrap(),
            },
            details: None,
            links: vec![],
            country: "Test".to_string(),
            state: None,
            city: "Test".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
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
