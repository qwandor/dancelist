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
use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The prefix which Facebook event URLs start with.
const FACEBOOK_EVENT_PREFIX: &str = "https://www.facebook.com/events/";

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// The name of the event.
    pub name: String,
    /// More details describing the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// URLs with more information about the event, including the Facebook event page if any.
    #[serde(default)]
    pub links: Vec<String>,
    /// The first day of the event, in the local timezone.
    pub start_date: NaiveDate,
    /// The last day of the event, in the local timezone. Events which finish some hours after
    /// midnight should be considered to finish the day before.
    pub end_date: NaiveDate,
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
}

impl Event {
    /// Check that the event information is valid. Returns an empty list if it is, or a list of
    /// problems if not.
    pub fn validate(&self) -> Vec<&'static str> {
        let mut problems = vec![];

        if !self.workshop && !self.social {
            problems.push("Must have at least a workshop or a social.")
        }

        if self.start_date > self.end_date {
            problems.push("Start date must not be before or equal to end date.");
        }

        if self.styles.is_empty() {
            problems.push("Must include at least one style of dance.")
        }

        problems
    }

    /// Get the URL of the event's Facebook event, if any.
    pub fn facebook_event(&self) -> Option<&String> {
        self.links
            .iter()
            .find(|link| link.starts_with(FACEBOOK_EVENT_PREFIX))
    }

    /// Get the event's first non-Facebook link.
    pub fn main_link(&self) -> Option<&String> {
        self.links
            .iter()
            .find(|link| !link.starts_with(FACEBOOK_EVENT_PREFIX))
    }

    /// Gets any further links, which are not the first and not the Facebook event.
    pub fn further_links(&self) -> Vec<&String> {
        self.links
            .iter()
            .skip(1)
            .filter(|link| !link.starts_with(FACEBOOK_EVENT_PREFIX))
            .collect()
    }

    /// Checks whether the event lasts more than one day.
    pub fn multiday(&self) -> bool {
        self.start_date != self.end_date
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
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
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
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
    }

    pub fn matches(&self, event: &Event, today: NaiveDate) -> bool {
        match self.date {
            DateFilter::Future if event.end_date < today => return false,
            DateFilter::Past if event.start_date >= today => return false,
            _ => {}
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

        true
    }
}
