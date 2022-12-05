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

use super::dancestyle::DanceStyle;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Not;

/// The prefix which Facebook event URLs start with.
const FACEBOOK_EVENT_PREFIX: &str = "https://www.facebook.com/events/";
const FBB_EVENT_PREFIX: &str = "https://folkbalbende.be/event/";

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Event {
    /// The name of the event.
    pub name: String,
    /// More details describing the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// URLs with more information about the event, including the Facebook event page if any.
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(flatten)]
    pub time: EventTime,
    pub country: String,
    pub city: String,
    // TODO: What about full address?
    /// The dance styles included in the event.
    #[serde(default)]
    pub styles: Vec<DanceStyle>,
    /// The event includes one or more workshops or lessons.
    #[serde(default)]
    pub workshop: bool,
    /// The event includes one or more social dances.
    #[serde(default)]
    pub social: bool,
    /// The names of the bands playing at the event.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bands: Vec<String>,
    /// The names of the callers calling at the event, if applicable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub callers: Vec<String>,
    /// The price or price range of the event, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// The organisation who run the event.
    #[serde(default)]
    pub organisation: Option<String>,
    /// Whether the event has been cancelled.
    #[serde(default, skip_serializing_if = "Not::not")]
    pub cancelled: bool,
    /// The name of the file in which this event is stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum EventTime {
    DateOnly {
        /// The first day of the event, in the local timezone.
        start_date: NaiveDate,
        /// The last day of the event, in the local timezone. Events which finish some hours after
        /// midnight should be considered to finish the day before.
        end_date: NaiveDate,
    },
    DateTime {
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    },
}

impl EventTime {
    /// Gets the start time in UTC for the purposes of sorting.
    pub fn start_time_sort_key(&self) -> DateTime<Utc> {
        match self {
            EventTime::DateOnly {
                start_date,
                end_date: _,
            } => Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap()),
            EventTime::DateTime { start, end: _ } => start.with_timezone(&Utc),
        }
    }
}

impl Event {
    /// Check that the event information is valid. Returns an empty list if it is, or a list of
    /// problems if not.
    pub fn validate(&self) -> Vec<&'static str> {
        let mut problems = vec![];

        if self.name.is_empty() {
            problems.push("Must have a name.");
        }
        if self.country.is_empty() {
            problems.push("Must specify a country.");
        }
        if self.city.is_empty() {
            problems.push("Must specify a city.");
        }

        if !self.workshop && !self.social {
            problems.push("Must have at least a workshop or a social.");
        }

        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date,
            } => {
                if start_date > end_date {
                    problems.push("Start date must be before or equal to end date.");
                }
            }
            EventTime::DateTime { start, end } => {
                if start > end {
                    problems.push("Start must be before or equal to end.");
                }
            }
        }

        if self.styles.is_empty() {
            problems.push("Must include at least one style of dance.");
        }

        problems
    }

    /// Merge this event and the other into a combined one, if they are similar enough.
    pub fn merge(&self, other: &Event) -> Option<Event> {
        if self.time == other.time && self.country == other.country && self.city == other.city {
            let mut links = self.links.clone();
            links.extend(other.links.clone());
            links.dedup();

            let mut styles = self.styles.clone();
            styles.extend(other.styles.clone());
            styles.sort();
            styles.dedup();

            let mut bands = self.bands.clone();
            bands.extend(other.bands.clone());
            bands.sort();
            bands.dedup();

            let mut callers = self.callers.clone();
            callers.extend(other.callers.clone());
            callers.sort();
            callers.dedup();

            let details = match (&self.details, &other.details) {
                (None, None) => None,
                (Some(d), None) | (None, Some(d)) => Some(d.clone()),
                (Some(a), Some(b)) => {
                    if a == b {
                        Some(a.clone())
                    } else {
                        Some(format!("{}\n{}", a, b))
                    }
                }
            };

            let name = if self.name.contains("TBA")
                || self.name.contains(" in ") && !other.name.contains("TBA")
            {
                other.name.clone()
            } else {
                self.name.clone()
            };

            let price = merge_strings(&self.price, &other.price);
            let organisation = merge_strings(&self.organisation, &other.organisation);
            let source = merge_strings(&self.source, &other.source);

            Some(Event {
                name,
                details,
                links,
                time: self.time.clone(),
                country: self.country.clone(),
                city: self.city.clone(),
                styles,
                workshop: self.workshop || other.workshop,
                social: self.social || other.social,
                bands,
                callers,
                price,
                organisation,
                cancelled: self.cancelled || other.cancelled,
                source,
            })
        } else {
            None
        }
    }

    /// Get the event's first non-Facebook non-FBB link.
    pub fn main_link(&self) -> Option<&String> {
        self.links.iter().find(|link| {
            !link.starts_with(FACEBOOK_EVENT_PREFIX) && !link.starts_with(FBB_EVENT_PREFIX)
        })
    }

    /// Gets any further links, which are not the first and not the Facebook event.
    pub fn further_links(&self) -> Vec<Link> {
        let mut facebook_links = vec![];
        let mut fbb_links = vec![];
        let mut other_links = vec![];
        let mut first_gone = false;
        for link in &self.links {
            if link.starts_with(FACEBOOK_EVENT_PREFIX) {
                facebook_links.push(Link {
                    short_name: "Facebook".to_string(),
                    url: link.to_owned(),
                })
            } else if link.starts_with(FBB_EVENT_PREFIX) {
                fbb_links.push(Link {
                    short_name: "FBB".to_string(),
                    url: link.to_owned(),
                })
            } else if first_gone {
                other_links.push(Link {
                    short_name: "…".to_string(),
                    url: link.to_owned(),
                })
            } else {
                first_gone = true;
            }
        }

        let mut links = facebook_links;
        links.extend(fbb_links);
        links.extend(other_links);
        links
    }

    /// Checks whether the event lasts more than one day.
    pub fn multiday(&self) -> bool {
        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date,
            } => start_date != end_date,
            // Subtract a few hours from the end time in case it finishes after midnight.
            EventTime::DateTime { start, end } => {
                start.date_naive() < (end - Duration::hours(5)).date_naive()
            }
        }
    }

    /// Gets the year in which the event starts.
    pub fn start_year(&self) -> i32 {
        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date: _,
            } => start_date.year(),
            EventTime::DateTime { start, end: _ } => start.year(),
        }
    }

    /// Gets the month in which the event starts.
    pub fn start_month(&self) -> u32 {
        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date: _,
            } => start_date.month(),
            EventTime::DateTime { start, end: _ } => start.month(),
        }
    }

    /// Formats the event start date/time, and end date/time if it is different,
    /// assuming that the start year and month is already known.
    pub fn short_time(&self) -> String {
        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date,
            } => {
                if !self.multiday() {
                    start_date.format("%a %e").to_string()
                } else if start_date.month() == end_date.month() {
                    format!(
                        "{}–{}",
                        start_date.format("%a %e"),
                        end_date.format("%a %e")
                    )
                } else {
                    format!(
                        "{}–{}",
                        start_date.format("%a %e"),
                        end_date.format("%a %e %B")
                    )
                }
            }
            EventTime::DateTime { start, end } => {
                if !self.multiday() {
                    format!(
                        "{}–{}",
                        start.format("%a %e %l:%M %P"),
                        end.format("%l:%M %P")
                    )
                } else if start.month() == end.month() {
                    format!(
                        "{}–{}",
                        start.format("%a %e %l:%M %P"),
                        end.format("%a %e %l:%M %P")
                    )
                } else {
                    format!(
                        "{}–{}",
                        start.format("%a %e %l:%M %P"),
                        end.format("%a %e %B %l:%M %P")
                    )
                }
            }
        }
    }
}

fn merge_strings(a: &Option<String>, b: &Option<String>) -> Option<String> {
    match (a, b) {
        (None, None) => None,
        (Some(o), None) | (None, Some(o)) => Some(o.clone()),
        (Some(a), Some(b)) => {
            if a == b {
                Some(a.clone())
            } else {
                // Can't merge different strings.
                None
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link {
    pub short_name: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn multiday() {
        // An event which starts in the evening and finishes a bit after midnight shouldn't count as
        // a multi-day event.
        let mut event = Event {
            name: "Test event".to_string(),
            details: None,
            links: vec![],
            time: EventTime::DateTime {
                start: FixedOffset::east_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2020, 1, 2, 19, 0, 0)
                    .single()
                    .unwrap(),
                end: FixedOffset::east_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2020, 1, 3, 4, 0, 0)
                    .single()
                    .unwrap(),
            },
            country: "Country".to_string(),
            city: "City".to_string(),
            styles: vec![],
            workshop: false,
            social: true,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        };
        assert!(!event.multiday());

        // Even if it starts in the morning it still shouldn't count as multi-day.
        event.time = EventTime::DateTime {
            start: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 2, 9, 0, 0)
                .single()
                .unwrap(),
            end: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 3, 4, 0, 0)
                .single()
                .unwrap(),
        };
        assert!(!event.multiday());

        // But if it starts a day earlier, it should.
        event.time = EventTime::DateTime {
            start: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 1, 19, 0, 0)
                .single()
                .unwrap(),
            end: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 3, 4, 0, 0)
                .single()
                .unwrap(),
        };
        assert!(event.multiday());

        // An event that starts in the evening and continues on into the next afternoon is multi-day.
        event.time = EventTime::DateTime {
            start: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 2, 21, 0, 0)
                .single()
                .unwrap(),
            end: FixedOffset::east_opt(0)
                .unwrap()
                .with_ymd_and_hms(2020, 1, 3, 16, 0, 0)
                .single()
                .unwrap(),
        };
        assert!(event.multiday());
    }
}
