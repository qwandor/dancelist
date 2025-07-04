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

use enum_iterator::{Sequence, all};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(
    Copy,
    Clone,
    Debug,
    Deserialize,
    Eq,
    Hash,
    JsonSchema,
    Ord,
    PartialEq,
    PartialOrd,
    Sequence,
    Serialize,
)]
pub enum DanceStyle {
    #[serde(rename = "balfolk")]
    Balfolk,
    #[serde(rename = "contra")]
    Contra,
    #[serde(rename = "e-ceilidh")]
    EnglishCeilidh,
    #[serde(rename = "ceili")]
    IrishCeili,
    #[serde(rename = "irish-set")]
    IrishSet,
    #[serde(rename = "italian")]
    Italian,
    #[serde(rename = "ecd")]
    EnglishCountryDance,
    #[serde(rename = "polish")]
    Polish,
    #[serde(rename = "reeling")]
    Reeling,
    #[serde(rename = "s-ceilidh")]
    ScottishCeilidh,
    #[serde(rename = "scd")]
    ScottishCountryDance,
    #[serde(rename = "scandi")]
    Scandinavian,
}

impl DanceStyle {
    pub fn values() -> impl Iterator<Item = Self> {
        all::<Self>()
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::Balfolk => "balfolk",
            Self::Contra => "contra",
            Self::EnglishCeilidh => "e-ceilidh",
            Self::IrishCeili => "ceili",
            Self::IrishSet => "irish-set",
            Self::Italian => "italian",
            Self::EnglishCountryDance => "ecd",
            Self::Polish => "polish",
            Self::Reeling => "reeling",
            Self::ScottishCeilidh => "s-ceilidh",
            Self::ScottishCountryDance => "scd",
            Self::Scandinavian => "scandi",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Balfolk => "balfolk",
            Self::Contra => "contra",
            Self::EnglishCeilidh => "English ceilidh",
            Self::IrishCeili => "Irish céilí",
            Self::IrishSet => "Irish set",
            Self::Italian => "Italian",
            Self::EnglishCountryDance => "ECD",
            Self::Polish => "Polish",
            Self::Reeling => "Scottish reeling",
            Self::ScottishCeilidh => "Scottish cèilidh",
            Self::ScottishCountryDance => "SCD",
            Self::Scandinavian => "Scandi",
        }
    }
}

impl Display for DanceStyle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}
