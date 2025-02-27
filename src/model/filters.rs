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
use serde::{de::IntoDeserializer, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Filters {
    #[serde(default, skip_serializing_if = "is_default")]
    pub date: DateFilter,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "strings_ser",
        deserialize_with = "strings_de"
    )]
    pub country: HashSet<String>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "strings_ser",
        deserialize_with = "strings_de"
    )]
    pub state: HashSet<String>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "strings_ser",
        deserialize_with = "strings_de"
    )]
    pub city: HashSet<String>,
    #[serde(
        alias = "style", // For backwards compatibility with old URLs.
        default,
        skip_serializing_if = "is_default",
        serialize_with = "styles_ser",
        deserialize_with = "styles_de"
    )]
    pub styles: HashSet<DanceStyle>,
    pub multiday: Option<bool>,
    pub workshop: Option<bool>,
    pub social: Option<bool>,
    pub band: Option<String>,
    pub caller: Option<String>,
    pub organisation: Option<String>,
    pub cancelled: Option<bool>,
}

fn styles_ser<S: Serializer>(
    styles: &HashSet<DanceStyle>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut style_tags: Vec<_> = styles.iter().map(|style| style.tag()).collect();
    // Sort so as to maintain a consistent serialisation.
    style_tags.sort();
    serializer.serialize_str(&style_tags.join(","))
}

fn styles_de<'de, D: Deserializer<'de>>(deserializer: D) -> Result<HashSet<DanceStyle>, D::Error> {
    let string = String::deserialize(deserializer)?;
    string
        .split(',')
        .map(|style_tag| DanceStyle::deserialize(style_tag.into_deserializer()))
        .collect()
}

fn strings_ser<S: Serializer>(strings: &HashSet<String>, serializer: S) -> Result<S::Ok, S::Error> {
    let mut strings: Vec<_> = strings.iter().map(ToOwned::to_owned).collect();
    // Sort so as to maintain a consistent serialisation.
    strings.sort();
    serializer.serialize_str(&strings.join(","))
}

fn strings_de<'de, D: Deserializer<'de>>(deserializer: D) -> Result<HashSet<String>, D::Error> {
    let string = String::deserialize(deserializer)?;
    Ok(string.split(',').map(|item| item.to_owned()).collect())
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
        !self.country.is_empty()
            || !self.state.is_empty()
            || !self.city.is_empty()
            || !self.styles.is_empty()
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

        if !self.country.is_empty() && !self.country.contains(&event.country) {
            return false;
        }
        if !self.state.is_empty()
            && !self
                .state
                .contains(event.state.as_deref().unwrap_or_default())
        {
            return false;
        }
        if !self.city.is_empty() && !self.city.contains(&event.city) {
            return false;
        }
        if !self.styles.is_empty() && !event.styles.iter().any(|style| self.styles.contains(style))
        {
            return false;
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
        let style = if self.styles.is_empty() {
            "Folk dance".to_string()
        } else {
            let mut styles: Vec<_> = self.styles.iter().collect();
            // Sort to ensure a consistent title.
            styles.sort();
            let styles: Vec<_> = styles
                .into_iter()
                .map(|style| uppercase_first_letter(style.name()))
                .collect();
            join_words(&styles)
        };
        let countries = if self.country.is_empty() {
            None
        } else {
            Some(join_cities(&self.country))
        };
        let states = if self.state.is_empty() {
            None
        } else {
            Some(join_cities(&self.state))
        };
        let cities = if self.city.is_empty() {
            None
        } else {
            Some(join_cities(&self.city))
        };

        match (countries, states, cities) {
            (None, None, None) => format!("{} events", style),
            (Some(countries), None, None) => {
                if countries == "UK" || countries == "USA" {
                    format!("{} events in the {}", style, countries)
                } else {
                    format!("{} events in {}", style, countries)
                }
            }
            (None, None, Some(cities)) => format!("{} events in {}", style, cities),
            (None, Some(states), None) => {
                format!("{} events in {}", style, states)
            }
            (None, Some(states), Some(cities)) => {
                format!("{} events in {}, {}", style, cities, states)
            }
            (Some(country), None, Some(cities)) => {
                format!("{} events in {}, {}", style, cities, country)
            }
            (Some(country), Some(states), None) => {
                format!("{} events in {}, {}", style, states, country)
            }
            (Some(country), Some(states), Some(cities)) => {
                format!("{} events in {}, {}, {}", style, cities, states, country)
            }
        }
    }

    /// Makes a new set of filters like this one but with the given country filter and no state or
    /// city filter.
    pub fn with_country(&self, country: Option<&str>) -> Self {
        Self {
            country: country.iter().map(|country| country.to_string()).collect(),
            state: HashSet::new(),
            city: HashSet::new(),
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given state filter and no city filter.
    pub fn with_state(&self, state: Option<&str>) -> Self {
        Self {
            state: state.iter().map(|state| state.to_string()).collect(),
            city: HashSet::new(),
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given city filter.
    pub fn with_city(&self, city: Option<&str>) -> Self {
        Self {
            city: city.iter().map(|city| city.to_string()).collect(),
            ..self.clone()
        }
    }

    /// Makes a new set of filters like this one but with the given dance style filter.
    pub fn with_style(&self, style: Option<DanceStyle>) -> Self {
        Self {
            styles: style.into_iter().collect(),
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

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

fn join_cities(cities: &HashSet<String>) -> String {
    let mut cities: Vec<_> = cities.iter().map(ToOwned::to_owned).collect();
    cities.sort();
    join_words(&cities)
}

fn join_words(parts: &[String]) -> String {
    let mut s = String::new();
    for (i, part) in parts.iter().enumerate() {
        s += part;
        match parts.len().cmp(&(i + 2)) {
            Ordering::Greater => {
                s += ", ";
            }
            Ordering::Equal => {
                s += " and ";
            }
            Ordering::Less => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filters_title() {
        let filters = Filters::default();
        assert_eq!(filters.make_title(), "Folk dance events");
    }

    #[test]
    fn one_style_country_title() {
        let filters = Filters {
            styles: [DanceStyle::EnglishCountryDance].into_iter().collect(),
            country: ["New Zealand".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "ECD events in New Zealand");
    }

    #[test]
    fn two_style_title() {
        let filters = Filters {
            styles: [DanceStyle::Balfolk, DanceStyle::Contra]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Balfolk and Contra events");
    }

    #[test]
    fn three_style_title() {
        let filters = Filters {
            styles: [
                DanceStyle::Balfolk,
                DanceStyle::Contra,
                DanceStyle::Scandinavian,
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Balfolk, Contra and Scandi events");
    }

    #[test]
    fn one_city_title() {
        let filters = Filters {
            city: ["London".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Folk dance events in London");
    }

    #[test]
    fn special_country_title() {
        let filters = Filters {
            country: ["UK".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Folk dance events in the UK");
        let filters = Filters {
            country: ["USA".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Folk dance events in the USA");
        let filters = Filters {
            country: ["USA".to_string(), "UK".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.make_title(), "Folk dance events in UK and USA");
    }

    #[test]
    fn empty_filters_query_string() {
        let filters = Filters::default();
        assert_eq!(filters.to_query_string().unwrap(), "");
    }

    #[test]
    fn style_filters_query_string() {
        let filters = Filters {
            styles: [DanceStyle::EnglishCountryDance].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.to_query_string().unwrap(), "styles=ecd");
    }

    #[test]
    fn styles_filters_query_string() {
        let filters = Filters {
            styles: [
                DanceStyle::Balfolk,
                DanceStyle::Contra,
                DanceStyle::EnglishCeilidh,
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        assert_eq!(
            filters.to_query_string().unwrap(),
            "styles=balfolk%2Ccontra%2Ce-ceilidh"
        );
    }

    #[test]
    fn cities_filters_query_string() {
        let filters = Filters {
            city: ["London".to_string(), "Cambridge".to_string()]
                .into_iter()
                .collect(),
            ..Default::default()
        };
        assert_eq!(
            filters.to_query_string().unwrap(),
            "city=Cambridge%2CLondon"
        );
    }

    #[test]
    fn multiple_filters_query_string() {
        let filters = Filters {
            country: ["UK".to_string()].into_iter().collect(),
            city: ["London".to_string()].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(filters.to_query_string().unwrap(), "country=UK&city=London");
    }

    #[test]
    fn deserialize_styles_filters() {
        let query_string = "styles=balfolk%2Ccontra%2Ce-ceilidh";
        assert_eq!(
            serde_urlencoded::from_str::<Filters>(query_string).unwrap(),
            Filters {
                styles: [
                    DanceStyle::Balfolk,
                    DanceStyle::Contra,
                    DanceStyle::EnglishCeilidh,
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            }
        );
    }
}
