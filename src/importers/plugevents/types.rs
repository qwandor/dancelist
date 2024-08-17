// Copyright 2023 the dancelist authors.
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

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EventList {
    pub events: Vec<Event>,
    pub powered_by_message: String,
    pub plug_logo_url: String,
    pub event_list_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Event {
    pub slug: String,
    pub status: u8,
    pub created_at_iso: DateTime<Utc>,
    pub modified_at_iso: DateTime<Utc>,
    pub published_by_name: Option<String>,
    pub published_by_org_slug: Option<String>,
    pub start_date_time_iso: DateTime<Utc>,
    pub end_date_time_iso: DateTime<Utc>,
    pub is_all_day: bool,
    pub timezone: Tz,
    pub time_display: String,
    pub plug_url: String,
    pub banner_image_url: Option<String>,
    pub name: String,
    pub description: String,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
    pub venue_locale: Option<String>,
    pub low_price: Option<u32>,
    pub high_price: Option<u32>,
    pub low_price2: Option<u32>,
    pub high_price2: Option<u32>,
    pub currency: Option<String>,
    pub is_free: bool,
    pub price_display: Option<String>,
    pub is_expanded: bool,
    pub date_grouping_label: String,
    pub subinterests: Vec<EventFormat>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub enum EventFormat {
    #[default]
    Advanced,
    BalfolkNL,
    Class,
    Course,
    Event,
    Festival,
    Intensive,
    Learning,
    #[serde(rename = "Lesson-series")]
    LessonSeries,
    Meeting,
    Organiser,
    Party,
    Social,
    Teacher,
}
