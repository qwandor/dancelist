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

use crate::{
    model::{
        dancestyle::DanceStyle,
        event::{Event, EventTime},
    },
    util::{default_timezone_for, local_datetime_to_fixed_offset},
};
use chrono::{NaiveDate, NaiveDateTime};
use chrono_tz::{TZ_VARIANTS, Tz};
use serde::{Deserialize, Deserializer, de::IntoDeserializer};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct EventForm {
    #[serde(deserialize_with = "trim")]
    pub name: String,
    #[serde(deserialize_with = "trim_non_empty")]
    pub details: Option<String>,
    #[serde(deserialize_with = "trim_non_empty_vec")]
    pub links: Vec<String>,
    #[serde(default)]
    pub with_time: bool,
    #[serde(deserialize_with = "date_or_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(deserialize_with = "date_or_none")]
    pub end_date: Option<NaiveDate>,
    #[serde(deserialize_with = "datetime_or_none")]
    pub start: Option<NaiveDateTime>,
    #[serde(deserialize_with = "datetime_or_none")]
    pub end: Option<NaiveDateTime>,
    pub timezone: Option<Tz>,
    #[serde(deserialize_with = "trim")]
    pub country: String,
    #[serde(deserialize_with = "trim_non_empty")]
    pub state: Option<String>,
    #[serde(deserialize_with = "trim")]
    pub city: String,
    #[serde(default)]
    pub styles: Vec<DanceStyle>,
    #[serde(default)]
    pub workshop: bool,
    #[serde(default)]
    pub social: bool,
    #[serde(deserialize_with = "trim_non_empty_vec")]
    pub bands: Vec<String>,
    #[serde(deserialize_with = "trim_non_empty_vec")]
    pub callers: Vec<String>,
    #[serde(deserialize_with = "trim_non_empty")]
    pub price: Option<String>,
    #[serde(deserialize_with = "trim_non_empty")]
    pub organisation: Option<String>,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(deserialize_with = "trim_non_empty")]
    pub email: Option<String>,
}

impl EventForm {
    pub fn start_date_string(&self) -> String {
        if let Some(start_date) = self.start_date {
            start_date.to_string()
        } else {
            String::default()
        }
    }

    pub fn end_date_string(&self) -> String {
        if let Some(end_date) = self.end_date {
            end_date.to_string()
        } else {
            String::default()
        }
    }

    pub fn start_string(&self) -> String {
        if let Some(start) = self.start {
            start.to_string()
        } else {
            String::default()
        }
    }

    pub fn end_string(&self) -> String {
        if let Some(end) = self.end {
            end.to_string()
        } else {
            String::default()
        }
    }

    pub fn from_event(event: &Event) -> Self {
        let (with_time, start_date, end_date, start, end, timezone) = match event.time {
            EventTime::DateOnly {
                start_date,
                end_date,
            } => (false, Some(start_date), Some(end_date), None, None, None),
            EventTime::DateTime { start, end } => {
                let timezone = default_timezone_for(&event.country, event.state.as_deref())
                    .filter(|timezone| {
                        // Check that timezone is plausible.
                        local_datetime_to_fixed_offset(&start.naive_local(), *timezone)
                            == Some(start)
                    })
                    .or_else(|| {
                        //  Find a plausible timezone
                        TZ_VARIANTS.into_iter().find(|timezone| {
                            local_datetime_to_fixed_offset(&start.naive_local(), *timezone)
                                == Some(start)
                        })
                    });
                (
                    true,
                    None,
                    None,
                    Some(start.naive_local()),
                    Some(end.naive_local()),
                    timezone,
                )
            }
        };

        Self {
            name: event.name.clone(),
            details: event.details.clone(),
            links: event.links.clone(),
            with_time,
            start_date,
            end_date,
            start,
            end,
            timezone,
            country: event.country.clone(),
            state: event.state.clone(),
            city: event.city.clone(),
            styles: event.styles.clone(),
            workshop: event.workshop,
            social: event.social,
            bands: event.bands.clone(),
            callers: event.callers.clone(),
            price: event.price.clone(),
            organisation: event.organisation.clone(),
            cancelled: event.cancelled,
            email: None,
        }
    }
}

impl TryFrom<EventForm> for Event {
    type Error = Vec<&'static str>;

    fn try_from(form: EventForm) -> Result<Self, Self::Error> {
        let time = if form.with_time {
            let timezone = form.timezone.ok_or_else(|| vec!["Missing timezone"])?;
            EventTime::DateTime {
                start: local_datetime_to_fixed_offset(
                    &form.start.ok_or_else(|| vec!["Missing start time"])?,
                    timezone,
                )
                .ok_or_else(|| vec!["Invalid time for timezone"])?,
                end: local_datetime_to_fixed_offset(
                    &form.end.ok_or_else(|| vec!["Missing end time"])?,
                    timezone,
                )
                .ok_or_else(|| vec!["Invalid time for timezone"])?,
            }
        } else {
            EventTime::DateOnly {
                start_date: form.start_date.ok_or_else(|| vec!["Missing start date"])?,
                end_date: form.end_date.ok_or_else(|| vec!["Missing end date"])?,
            }
        };
        let event = Self {
            name: form.name,
            details: form.details,
            links: form.links,
            time,
            country: form.country,
            state: form.state,
            city: form.city,
            styles: form.styles,
            workshop: form.workshop,
            social: form.social,
            bands: form
                .bands
                .into_iter()
                .filter_map(trimmed_non_empty)
                .collect(),
            callers: form
                .callers
                .into_iter()
                .filter_map(trimmed_non_empty)
                .collect(),
            price: form.price,
            organisation: form.organisation,
            cancelled: form.cancelled,
            source: None,
        };
        let problems = event.validate();
        if problems.is_empty() {
            Ok(event)
        } else {
            Err(problems)
        }
    }
}

fn trim<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(String::deserialize(deserializer)?.trim().to_string())
}

fn trimmed_non_empty(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.replace("\r\n", "\n"))
    }
}

fn trim_non_empty<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.and_then(trimmed_non_empty))
}

fn trim_non_empty_vec<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let s = Vec::<String>::deserialize(deserializer)?;
    Ok(s.into_iter().filter_map(trimmed_non_empty).collect())
}

fn date_or_none<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<NaiveDate>, D::Error> {
    if let Some(str) = Option::<String>::deserialize(deserializer)? {
        if str.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NaiveDate::deserialize(str.into_deserializer())?))
        }
    } else {
        Ok(None)
    }
}

fn datetime_or_none<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error> {
    if let Some(str) = Option::<String>::deserialize(deserializer)? {
        if str.is_empty() {
            Ok(None)
        } else {
            Ok(Some(NaiveDateTime::deserialize(
                format!("{str}:00").into_deserializer(),
            )?))
        }
    } else {
        Ok(None)
    }
}
