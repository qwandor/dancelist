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

mod bool_as_int;
mod int_as_string;

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: u32,
    pub name: String,
    pub recurrence: u32,
    #[serde(rename = "type")]
    pub event_type: EventType,
    #[serde(with = "bool_as_int")]
    pub cancelled: bool,
    #[serde(with = "bool_as_int")]
    pub deleted: bool,
    #[serde(with = "bool_as_int")]
    pub checked: bool,
    pub dates: Vec<NaiveDate>,
    pub location: Location,
    pub prices: Vec<Price>,
    pub thumbnail: String,
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
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Ball,
    Course,
    Festival,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub address: Address,
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub name: String,
    #[serde(with = "int_as_string")]
    pub price: i32,
    pub free_contribution: u32,
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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub id: u32,
    pub title: String,
    pub start: NaiveTime,
    pub end: NaiveTime,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ball {
    pub initiation_start: Option<NaiveTime>,
    pub initiation_end: Option<NaiveTime>,
    pub initiators: Vec<Teacher>,
    pub performances: Vec<Performance>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Performance {
    pub start: Option<NaiveTime>,
    pub end: Option<NaiveTime>,
    pub band: Band,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    pub id: u32,
    pub name: String,
    pub nl: String,
    pub fr: String,
    pub en: String,
    pub country: Country,
    #[serde(with = "bool_as_int")]
    pub placeholder: bool,
    pub websites: Vec<Website>,
    pub tags: Vec<String>,
    pub musicians: Vec<Musician>,
    pub image: Option<String>,
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    pub thumbnail: String,
    pub image: Option<String>,
    pub address: Option<Address>,
}
