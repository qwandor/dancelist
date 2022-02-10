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
use chrono::{Date, DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use eyre::Report;
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
    // TODO: Should start and end require time or just date? What about timezone?
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
    // TODO: Should free events be distinguished from events with unknown price?
    /// The organisation who run the event.
    #[serde(default)]
    pub organisation: Option<String>,
    /// Whether the event has been cancelled.
    #[serde(default, skip_serializing_if = "Not::not")]
    pub cancelled: bool,
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

impl Event {
    /// Check that the event information is valid. Returns an empty list if it is, or a list of
    /// problems if not.
    pub fn validate(&self) -> Vec<&'static str> {
        let mut problems = vec![];

        if !self.workshop && !self.social {
            problems.push("Must have at least a workshop or a social.")
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
            problems.push("Must include at least one style of dance.")
        }

        problems
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
            EventTime::DateTime { start, end } => start.date() != end.date(),
        }
    }

    /// Gets the event start time in UTC for the purposes of sorting.
    pub fn start_time_sort_key(&self) -> DateTime<Utc> {
        match self.time {
            EventTime::DateOnly {
                start_date,
                end_date: _,
            } => Date::<Utc>::from_utc(start_date, Utc).and_hms(0, 0, 0),
            EventTime::DateTime { start, end: _ } => start.with_timezone(&Utc),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Link {
    pub short_name: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Filters {
    #[serde(default)]
    pub date: DateFilter,
    pub country: Option<String>,
    pub city: Option<String>,
    pub style: Option<DanceStyle>,
    pub multiday: Option<bool>,
    pub workshop: Option<bool>,
    pub social: Option<bool>,
    pub band: Option<String>,
    pub caller: Option<String>,
    pub organisation: Option<String>,
    pub cancelled: Option<bool>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateFilter {
    /// Include only events which started before the current day.
    Past,
    /// Include only events which finish on or after the current day.
    Future,
    /// Include all events, past and future.
    All,
}

impl Default for DateFilter {
    fn default() -> Self {
        Self::Future
    }
}

impl Filters {
    pub fn has_some(&self) -> bool {
        self.country.is_some()
            || self.city.is_some()
            || self.style.is_some()
            || self.multiday.is_some()
            || self.workshop.is_some()
            || self.social.is_some()
            || self.band.is_some()
            || self.caller.is_some()
            || self.organisation.is_some()
            || self.cancelled.is_some()
    }

    pub fn to_query_string(&self) -> Result<String, Report> {
        Ok(serde_urlencoded::to_string(self)?)
    }

    pub fn matches(&self, event: &Event, now: DateTime<Utc>) -> bool {
        let today = now.naive_utc().date();
        match event.time {
            EventTime::DateOnly {
                start_date,
                end_date,
            } => match self.date {
                DateFilter::Future if end_date < today => return false,
                DateFilter::Past if start_date >= today => return false,
                _ => {}
            },
            EventTime::DateTime { start, end } => match self.date {
                DateFilter::Future if end < now => return false,
                DateFilter::Past if start >= now => return false,
                _ => {}
            },
        }

        if let Some(country) = &self.country {
            if &event.country != country {
                return false;
            }
        }
        if let Some(city) = &self.city {
            if &event.city != city {
                return false;
            }
        }
        if let Some(style) = &self.style {
            if !event.styles.contains(style) {
                return false;
            }
        }
        if let Some(multiday) = self.multiday {
            if event.multiday() != multiday {
                return false;
            }
        }
        if let Some(workshop) = self.workshop {
            if event.workshop != workshop {
                return false;
            }
        }
        if let Some(social) = self.social {
            if event.social != social {
                return false;
            }
        }
        if let Some(band) = &self.band {
            if !event.bands.contains(band) {
                return false;
            }
        }
        if let Some(caller) = &self.caller {
            if !event.callers.contains(caller) {
                return false;
            }
        }
        if let Some(organisation) = &self.organisation {
            if &event.organisation.as_deref().unwrap_or_default() != organisation {
                return false;
            }
        }
        if let Some(cancelled) = self.cancelled {
            if event.cancelled != cancelled {
                return false;
            }
        }

        true
    }
}
