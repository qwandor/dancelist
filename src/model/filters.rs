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

use super::{
    dancestyle::DanceStyle,
    event::{Event, EventTime},
};
use chrono::{DateTime, Utc};
use enum_iterator::{all, Sequence};
use eyre::Report;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Filters {
    #[serde(default, skip_serializing_if = "is_default")]
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

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Sequence, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DateFilter {
    /// Include only events which started before the current day.
    Past,
    /// Include only events which finish on or after the current day.
    Future,
    /// Include all events, past and future.
    All,
}

impl DateFilter {
    pub fn values() -> impl Iterator<Item = Self> {
        all::<DateFilter>()
    }
}

impl Default for DateFilter {
    fn default() -> Self {
        Self::Future
    }
}

impl Display for DateFilter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match self {
            Self::Past => "past",
            Self::Future => "future",
            Self::All => "all",
        };
        f.write_str(s)
    }
}

impl Filters {
    pub fn all() -> Self {
        Self {
            date: DateFilter::All,
            ..Default::default()
        }
    }

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
            if event.organisation.as_deref().unwrap_or_default() != organisation {
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

    /// Make a page title for this set of filters.
    pub fn make_title(&self) -> String {
        let style = if let Some(style) = self.style {
            uppercase_first_letter(style.name())
        } else {
            "Folk dance".to_string()
        };

        match (&self.country, &self.city) {
            (None, None) => format!("{} events", style),
            (Some(country), None) => {
                if country == "UK" || country == "USA" {
                    format!("{} events in the {}", style, country)
                } else {
                    format!("{} events in {}", style, country)
                }
            }
            (None, Some(city)) => format!("{} events in {}", style, city),
            (Some(country), Some(city)) => format!("{} events in {}, {}", style, city, country),
        }
    }

    /// Makes a new set of filters like this one but with the given country filter and no city filter.
    pub fn with_country(&self, country: Option<&str>) -> Self {
        Self {
            country: owned(country),
            city: None,
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given city filter.
    pub fn with_city(&self, city: Option<&str>) -> Self {
        Self {
            city: owned(city),
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given dance style filter.
    pub fn with_style(&self, style: Option<DanceStyle>) -> Self {
        Self {
            style,
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given date filter.
    pub fn with_date(&self, date: DateFilter) -> Self {
        Self {
            date,
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given multi-day filter.
    pub fn with_multiday(&self, multiday: Option<bool>) -> Self {
        Self {
            multiday,
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given social filter.
    pub fn with_social(&self, social: Option<bool>) -> Self {
        Self {
            social,
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given workshop filter.
    pub fn with_workshop(&self, workshop: Option<bool>) -> Self {
        Self {
            workshop,
            ..self.clone()
        }
    }
}

/// Make the first letter of the given string uppercase.
fn uppercase_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        String::new()
    }
}

fn owned<T: ToOwned + ?Sized>(option_ref: Option<&T>) -> Option<T::Owned> {
    option_ref.map(ToOwned::to_owned)
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
