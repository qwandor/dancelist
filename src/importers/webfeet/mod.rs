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

pub mod types;

use self::types::{EventRecord, Eventlist};
use eyre::Report;

pub async fn events() -> Result<Vec<EventRecord>, Report> {
    let xml = reqwest::get("https://www.webfeet.org/dance.xml")
        .await?
        .text()
        .await?;
    let xml = replace_entities(&xml);
    let event_list: Eventlist = quick_xml::de::from_str(&xml)?;
    // Sort by ID to give a stable order.
    //events.sort_by_key(|event| event.id);
    Ok(event_list.event_record)
}

fn replace_entities(source: &str) -> String {
    source.replace("&icirc;", "&#238;")
}
