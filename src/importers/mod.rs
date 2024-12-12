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

pub mod bands;
pub mod callers;
pub mod folkbalbende;
pub mod icalendar;
pub mod plugevents;
pub mod trycontra;
pub mod webfeet;

use std::{
    collections::HashMap,
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
};

use crate::{github::to_safe_filename, model::events::Events};
use eyre::Report;
use log::info;

/// Adds any old events older than the oldest new event, and returns the combination.
///
/// This is useful to preserve past events for importers for sources which don't include events in the past.
fn combine_events(old_events: Events, new_events: Events) -> Events {
    let Some(earliest_finish) = new_events
        .events
        .iter()
        .map(|e| e.time.end_time_sort_key())
        .min()
    else {
        // If there are no new events then keep all the old events.
        return old_events;
    };

    let mut events = new_events;
    events.events.extend(
        old_events
            .events
            .into_iter()
            .filter(|event| event.time.end_time_sort_key() < earliest_finish),
    );
    events.sort();
    events.events.dedup();
    events
}

/// Given a set of events, splits them by country then writes one file for each country.
///
/// If the file already exists for that country then applies the logic from [`combine_events`] to
/// preserve old events in it.
pub fn write_by_country(events: Events, filename: &Path) -> Result<(), Report> {
    let mut events_by_country: HashMap<String, Events> = HashMap::new();
    for event in events.events {
        events_by_country
            .entry(event.country.clone())
            .or_default()
            .events
            .push(event);
    }
    for (country, mut country_events) in events_by_country {
        let mut country_filename = PathBuf::new();
        country_filename.push("events");
        country_filename.push(to_safe_filename(&country));
        country_filename.push(filename);
        info!(
            "Writing {} events to {:?}",
            country_events.events.len(),
            country_filename
        );
        if country_filename.exists() {
            // Load without validating, as imports may be invalid.
            let old_events = Events::load_file_without_validation(&country_filename)?;
            country_events = combine_events(old_events, country_events);
        } else {
            create_dir_all(country_filename.parent().unwrap())?;
        }
        write(country_filename, country_events.to_yaml_string()?)?;
    }
    Ok(())
}

/// Returns strings from the slice which are contained in one of the two lowercase strings passed.
///
/// Also finds matches where "&" is replaced with "and".
fn lowercase_matches(needles: &[&str], a: &str, b: &str) -> Vec<String> {
    needles
        .iter()
        .filter_map(|needle| {
            let needle_lower = needle.to_lowercase();
            let needle_and = needle_lower.replace("&", "and");
            if a.contains(&needle_lower)
                || b.contains(&needle_lower)
                || a.contains(&needle_and)
                || b.contains(&needle_and)
            {
                Some(needle.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        dancestyle::DanceStyle,
        event::{Event, EventTime},
    };
    use chrono::NaiveDate;

    fn make_event(name: &str, time: EventTime) -> Event {
        Event {
            name: name.to_string(),
            time,
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
        }
    }

    #[test]
    fn combine_no_old() {
        let old_events = Events::default();
        let new_events = Events {
            events: vec![
                make_event(
                    "New 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "New 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };

        let combined = combine_events(old_events, new_events.clone());
        assert_eq!(combined, new_events);
    }

    #[test]
    fn combine_no_new() {
        let old_events = Events {
            events: vec![
                make_event(
                    "Old 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "Old 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };
        let new_events = Events::default();

        let combined = combine_events(old_events.clone(), new_events);
        assert_eq!(combined, old_events);
    }

    #[test]
    fn combine_same() {
        let events = Events {
            events: vec![
                make_event(
                    "Old 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "Old 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };

        let combined = combine_events(events.clone(), events.clone());
        assert_eq!(combined, events);
    }

    #[test]
    fn combine_overlap() {
        let old1 = make_event(
            "Old 1",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            },
        );
        let old3 = make_event(
            "Old 3",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 3).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 3).unwrap(),
            },
        );
        let old_events = Events {
            events: vec![old1.clone(), old3.clone()],
        };
        let new2 = make_event(
            "New 2",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
            },
        );
        let new4 = make_event(
            "New 4",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 4).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 4).unwrap(),
            },
        );
        let new_events = Events {
            events: vec![new2.clone(), new4.clone()],
        };

        let combined = combine_events(old_events, new_events);
        assert_eq!(combined.events, vec![old1, new2, new4]);
    }

    #[test]
    fn match_band() {
        const TEST_BANDS: [&str; 3] = ["Matt Norman & Edward Wallace", "Nozzy", "Nubia"];

        assert_eq!(
            lowercase_matches(&TEST_BANDS, "with nozzy", "and nubia"),
            vec!["Nozzy".to_string(), "Nubia".to_string()]
        );
        assert_eq!(
            lowercase_matches(
                &TEST_BANDS,
                "bob morgan with matt norman and edward wallace",
                ""
            ),
            vec!["Matt Norman & Edward Wallace".to_string()]
        );
    }
}
