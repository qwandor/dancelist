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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Eventlist {
    pub event_record: Vec<EventRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct EventRecord {
    pub id: Id,
    pub canonical_date: CanonicalDate,
    pub text_date: Option<TextDate>,
    #[serde(default)]
    pub event_collection: EventCollection,
    #[serde(default)]
    pub band_collection: BandCollection,
    #[serde(default)]
    pub caller_collection: CallerCollection,
    pub location_collection: LocationCollection,
    pub reference: Reference,
    pub rank: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Id {
    #[serde(rename = "@Type")]
    pub id_type: String,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct CanonicalDate {
    #[serde(rename = "@Isoformat")]
    pub isoformat: String,
    #[serde(rename = "@Uncertainty")]
    pub uncertainty: Option<u32>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct TextDate {
    #[serde(rename = "@Isoformat")]
    pub isoformat: String,
    #[serde(rename = "@Status")]
    pub status: String,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct EventCollection {
    pub event: Vec<Event>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Event {
    #[serde(rename = "@Style")]
    pub style: Option<Style>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct BandCollection {
    pub band: Vec<Band>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Band {
    #[serde(default, rename = "@Status")]
    pub status: Status,
    #[serde(rename = "@Style")]
    pub style: Option<Style>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct CallerCollection {
    pub caller: Vec<Caller>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Caller {
    #[serde(default, rename = "@Status")]
    pub status: Status,
    #[serde(rename = "@Style")]
    pub style: Option<Style>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct LocationCollection {
    pub location: Location,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Location {
    #[serde(rename = "@Status")]
    pub status: Option<String>,
    #[serde(rename = "@Area")]
    pub area: Option<String>,
    #[serde(rename = "@Mapref")]
    pub mapref: Option<String>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    Unrecognised,
    Recognised,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unrecognised
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Style {
    Contra,
    #[serde(rename = "Dance.American.American Contra")]
    DanceAmericanAmericanContra,
    #[serde(rename = "Dance.Bal")]
    DanceBal,
    #[serde(rename = "Dance.Eurobal")]
    DanceEurobal,
    #[serde(rename = "Dance.European")]
    DanceEuropean,
    #[serde(rename = "Dance.French/Breton")]
    DanceFrenchBreton,
    #[serde(rename = "Dance.Barn Dance")]
    DanceBarnDance,
    #[serde(rename = "Dance.Contra")]
    DanceContra,
    #[serde(rename = "Dance.Country Dance")]
    DanceCountryDance,
    #[serde(rename = "Dance.Ceilidh")]
    DanceCeilidh,
    #[serde(rename = "Dance.Ceildh")]
    DanceCeildh,
    #[serde(rename = "Dance.English Ceilidh")]
    DanceEnglishCeilidh,
    #[serde(rename = "Dance.English Folk")]
    DanceEnglishFolk,
    #[serde(rename = "Dance.Swedish")]
    DanceSwedish,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct Reference {
    pub source_data: SourceData,
    #[serde(rename = "URL")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "PascalCase")]
pub struct SourceData {
    #[serde(rename = "@Localcopy")]
    pub localcopy: String,
    #[serde(rename = "@SourceFormat")]
    pub source_format: SourceFormat,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SourceFormat {
    Anchor,
    Custom,
    DL,
    #[serde(rename = "JSON-LD")]
    JsonLd,
    PBR,
    Table,
    UL,
}
