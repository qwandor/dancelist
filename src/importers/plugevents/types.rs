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
    pub interest_tags: Vec<InterestTag>,
    pub interest_slugs: Vec<String>,
    pub interests: Vec<Interest>,
    pub featured_participants: Vec<FeaturedParticipant>,
    pub subinterests: Option<Vec<EventFormat>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Interest {
    pub subs: Vec<EventFormat>,
    pub tag: InterestTag,
    pub slug: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum InterestTag {
    Art,
    Bachata,
    Balfolk,
    #[serde(rename = "Balfolk dance")]
    Balfolkdance,
    #[serde(rename = "Balfolk Music")]
    BalfolkMusic,
    BalfolkNL,
    #[serde(rename = "Blues Dance")]
    BluesDance,
    #[serde(rename = "Community Service")]
    CommunityService,
    #[serde(rename = "Contra Dance")]
    ContraDance,
    #[serde(rename = "Couple dance")]
    CoupleDance,
    Dance,
    DancingBodies,
    #[serde(rename = "Folk Dance")]
    FolkDance,
    #[serde(rename = "Folk Music")]
    FolkMusic,
    #[serde(rename = "Forró Dance")]
    ForroDance,
    Fusion,
    Music,
    NeoTrad,
    #[serde(rename = "Salsa Dance")]
    SalsaDance,
    #[serde(rename = "Social Dance")]
    SocialDance,
    #[serde(rename = "Swedish Folk Dance")]
    SwedishFolkDance,
    #[serde(rename = "Swedish Traditional Music")]
    SwedishTraditionalMusic,
    Tango,
    Teacher,
    #[serde(rename = "Trad Music")]
    TradMusic,
    #[serde(rename = "West Coast Swing")]
    WestCoastSwing,
    Workshop,
    Zouk,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub enum EventFormat {
    #[default]
    Advanced,
    Bal,
    Balfolk,
    BalfolkNL,
    #[serde(rename = "Balmuziek leren")]
    BalmuziekLeren,
    Buurtvereniging,
    Class,
    #[serde(rename = "Community Association")]
    CommunityAssociation,
    Concert,
    Course,
    Dance,
    #[serde(rename = "Dance Class")]
    DanceClass,
    DancingBodies,
    Dansavond,
    #[serde(rename = "Dansen leren")]
    DansenLeren,
    Dansles,
    Event,
    Festival,
    Flashmob,
    Folkbal,
    FolkBal,
    Intensive,
    Jam,
    Learning,
    Les,
    #[serde(rename = "Lesson-series")]
    LessonSeries,
    #[serde(rename = "Live Music")]
    LiveMusic,
    #[serde(rename = "Live Muziek")]
    LiveMuziek,
    Meeting,
    #[serde(rename = "Music Class")]
    MusicClass,
    Musiekles,
    Optreden,
    Organisator,
    Organiser,
    Overig,
    Party,
    Performance,
    Piano,
    Practica,
    Social,
    #[serde(rename = "Social Class")]
    SocialClass,
    #[serde(rename = "Social dance")]
    SocialDance,
    #[serde(rename = "Social Dancing")]
    SocialDancing,
    Sociales,
    Teacher,
    Workshop,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FeaturedParticipant {
    pub kind: u8,
    pub slug: String,
    pub name: String,
    pub subtitle: Option<String>,
    pub create_date_iso: DateTime<Utc>,
    pub readable_create_date: String,
    pub event_readable_time: String,
    pub image_url: String,
    pub thumb_image_url: String,
    pub status: u8,
    pub plug_url: Option<String>,
}
