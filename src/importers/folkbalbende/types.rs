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

use chrono::{NaiveDate, NaiveTime};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as _, Unexpected},
};
use std::fmt::{self, Display, Formatter, Write};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: u32,
    pub name: String,
    pub recurrence: u32,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub cancelled: bool,
    pub deleted: bool,
    pub checked: bool,
    pub dates: Vec<NaiveDate>,
    pub location: Location,
    pub prices: Vec<Price>,
    pub thumbnail: Option<String>,
    pub reservation_type: u32,
    pub reservation_url: String,
    pub websites: Vec<Website>,
    #[serde(default)]
    pub courses: Vec<Course>,
    pub ball: Option<Ball>,
    pub facebook_event: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub tags: Vec<String>,
    pub image: Option<String>,
    pub organisation: Option<Organisation>,
    pub hidden: bool,
    pub creator: String,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    #[default]
    Ball,
    Course,
    Festival,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub address: Address,
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub id: u32,
    pub street: Option<String>,
    pub number: Option<String>,
    #[serde(rename = "zip-city")]
    pub zip_city: String,
    pub city: String,
    pub zip: String,
    pub lat: f32,
    pub lng: f32,
    pub region: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub name: String,
    pub price: PriceFix,
    pub free_contribution: bool,
}

/// A price as a number of cents.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PriceFix(pub i32);

impl Display for PriceFix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let cents = self.0.abs() % 100;
        let euros = self.0.abs() / 100;
        if self.0 < 0 {
            f.write_char('-')?;
        }
        if cents == 0 {
            euros.fmt(f)
        } else {
            write!(f, "{euros}.{cents:02}")
        }
    }
}

impl Serialize for PriceFix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cents = self.0.abs() % 100;
        let euros = self.0.abs() / 100;
        let sign = if self.0 < 0 { "-" } else { "" };
        let string = format!("{sign}{euros}.{cents:02}");
        serializer.serialize_str(&string)
    }
}

impl<'de> Deserialize<'de> for PriceFix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let (euros, cents) = string
            .split_once('.')
            .ok_or_else(|| D::Error::invalid_value(Unexpected::Str(&string), &"Price"))?;
        let euros = euros.parse::<i32>().map_err(D::Error::custom)?;
        let cents = cents.parse::<i32>().map_err(D::Error::custom)?;
        if euros >= 0 {
            Ok(Self(euros * 100 + cents))
        } else {
            Ok(Self(euros * 100 - cents))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Website {
    pub id: u32,
    #[serde(rename = "type")]
    pub website_type: WebsiteType,
    pub url: String,
    pub icon: Option<String>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebsiteType {
    Facebook,
    Instagram,
    #[serde(rename = "last.fm")]
    LastFm,
    Mail,
    MySpace,
    ReverbNation,
    SoundCloud,
    Spotify,
    #[serde(rename = "Vi.be")]
    ViBe,
    Website,
    #[serde(rename = "websites")]
    Websites,
    Wikipedia,
    Youtube,
    YouTube,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub id: u32,
    pub title: String,
    pub start: NaiveTime,
    pub end: Option<NaiveTime>,
    pub teachers: Vec<Teacher>,
    pub nl: String,
    pub fr: String,
    pub en: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Teacher {
    pub id: u32,
    pub name: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub thumbnail: Option<String>,
    pub image: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ball {
    pub initiation_start: Option<NaiveTime>,
    pub initiation_end: Option<NaiveTime>,
    pub initiators: Vec<Teacher>,
    pub performances: Vec<Performance>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Performance {
    pub start: Option<NaiveTime>,
    pub end: Option<NaiveTime>,
    pub band: Option<Band>,
    pub tbd: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    pub id: u32,
    pub name: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub country: Country,
    pub placeholder: bool,
    pub websites: Vec<Website>,
    pub tags: Vec<String>,
    pub musicians: Vec<Musician>,
    pub image: Option<String>,
    pub duplicate_of: Option<String>,
    pub editable: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Country {
    pub code: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Musician {
    pub id: u32,
    pub name: String,
    pub instruments: String,
    pub country: Country,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Organisation {
    pub id: u32,
    pub name: String,
    pub websites: Vec<Website>,
    pub thumbnail: Option<String>,
    pub image: Option<String>,
    pub address: Option<Address>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_price() {
        assert_eq!(PriceFix(0).to_string(), "0");
        assert_eq!(PriceFix(-100).to_string(), "-1");
        assert_eq!(PriceFix(-5).to_string(), "-0.05");
        assert_eq!(PriceFix(40).to_string(), "0.40");
        assert_eq!(PriceFix(1234).to_string(), "12.34");
        assert_eq!(PriceFix(1230).to_string(), "12.30");
    }
}
