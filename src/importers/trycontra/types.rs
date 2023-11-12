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

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub typical_month: String,
    pub name: String,
    pub callers: Vec<String>,
    pub bands: Vec<String>,
    pub roles: String,
    pub date: String,
    pub date_end: Option<String>,
    pub location: String,
    pub url: String,
    pub year: u32,
    pub latlng: Option<Vec<f64>>,
}
