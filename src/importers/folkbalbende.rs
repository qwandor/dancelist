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

use eyre::Report;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub event_type: Type,
    pub cancelled: u32,
    pub dates: Vec<String>,
    pub location: Location,
    pub prices: Vec<Price>,
    pub reservation_type: u32,
    pub reservation_url: String,
    #[serde(default)]
    pub courses: Vec<Course>,
    pub ball: Option<Ball>,
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Ball,
    Course,
    Festival,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub address: Address,
    pub duplicate_of: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
pub struct Price {
    pub name: String,
    pub price: String,
    pub free_contribution: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Course {
    pub id: u32,
    pub title: String,
    pub start: String,
    pub end: String,
    pub teachers: Vec<Teacher>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Teacher {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ball {
    pub initiation_start: Option<String>,
    pub initiation_end: Option<String>,
    pub performances: Vec<Performance>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Performance {
    pub start: Option<String>,
    pub end: Option<String>,
    pub band: Band,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Band {
    pub id: u32,
    pub name: String,
}

pub async fn minimal_events() -> Result<Vec<Event>, Report> {
    let json = reqwest::get("https://folkbalbende.be/interface/minimal_events.php?start=2022-02-01&end=3000-01-01&type=ball,course,festal").await?.text().await?;
    let events = serde_json::from_str(&json)?;
    Ok(events)
}
