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

use self::types::{EventRecord, Eventlist, Style};
use crate::model::{
    dancestyle::DanceStyle,
    event::{Event, EventTime},
    events::Events,
};
use chrono::NaiveDate;
use eyre::Report;

pub async fn events() -> Result<Vec<EventRecord>, Report> {
    let xml = reqwest::get("https://www.webfeet.org/dance.xml")
        .await?
        .text()
        .await?;
    let xml = replace_entities(&xml);
    let event_list: Eventlist = quick_xml::de::from_str(&xml)?;
    let mut events = event_list.event_record;
    // Sort by ID to give a stable order.
    events.sort_by(|a, b| a.id.value.cmp(&b.id.value));
    for event in &events {
        if event.canonical_date.isoformat.contains("-") {
            eprintln!("{}", event.canonical_date.isoformat);
        }
        if let Some(text_date) = &event.text_date {
            if event.canonical_date.isoformat != text_date.isoformat {
                eprintln!(
                    "{} != {}",
                    event.canonical_date.isoformat, text_date.isoformat
                );
            }
        }
    }
    Ok(events)
}

pub async fn import_events() -> Result<Events, Report> {
    let event_records = events().await?;

    let mut events = vec![];
    let mut merging_event: Option<Event> = None;
    for event in &event_records {
        if let Some(converted) = convert(event) {
            if let Some(previous_event) = merging_event {
                if let Some(merged) = previous_event.merge(&converted) {
                    merging_event = Some(merged);
                } else {
                    events.push(previous_event);
                    merging_event = Some(converted);
                }
            } else {
                merging_event = Some(converted);
            }
        }
    }
    events.extend(merging_event);

    Ok(Events { events })
}

fn replace_entities(source: &str) -> String {
    source
        .replace("&icirc;", "&#238;")
        .replace("&Ouml;", "&#214;")
        .replace("&Agrave;", "&#192;")
}

fn convert(event: &EventRecord) -> Option<Event> {
    let mut details = None;
    let bands: Vec<String> = event
        .band_collection
        .band
        .iter()
        .map(|band| band.value.clone())
        .collect();
    let city = event.location_collection.location.value.clone();

    let mut name = format!("{} in {}", bands.join(" & "), city);
    let mut cancelled = false;
    if let Some(event) = event.event_collection.event.first() {
        if event.value.chars().next().unwrap() == '[' {
            if event.value == "[Cancelled]" || event.value == "[Postponed]" {
                cancelled = true;
            }
            details = Some(event.value.clone());
        } else {
            name = event.value.clone();
        }
    }

    let mut callers = vec![];
    let mut styles = vec![];
    let mut links = vec![event
        .reference
        .url
        .replace("https://en-gb.facebook.com/", "https://www.facebook.com/")];
    for event in &event.event_collection.event {
        if let Some(style) = event.style {
            styles.extend(convert_style(style));
        }
    }
    for band in &event.band_collection.band {
        if let Some(style) = band.style {
            styles.extend(convert_style(style));
        }
    }
    for caller in &event.caller_collection.caller {
        if let Some(style) = caller.style {
            styles.extend(convert_style(style));
        }
        let value_lowercase = caller.value.to_lowercase();
        if value_lowercase == "ceilidh" {
            styles.push(DanceStyle::EnglishCeilidh);
        } else if value_lowercase == "barn dance" {
        } else if caller.value.starts_with("http") {
            links.push(caller.value.clone());
        } else {
            callers.push(caller.value.clone());
        }
    }
    styles.sort();
    styles.dedup();

    if styles.is_empty() {
        eprintln!("Dropping {} with no styles.", name);
        None
    } else if city == "Zoom" {
        eprintln!("Dropping {} on Zoom.", name);
        None
    } else if city == "Cecil Sharp House, Camden" {
        eprintln!("Dropping {} at Cecil Sharp House.", name);
        None
    } else {
        Some(Event {
            name,
            details,
            links,
            time: parse_date(&event.canonical_date.isoformat),
            country: "UK".to_string(),
            city,
            styles,
            workshop: false,
            social: true,
            bands,
            callers,
            price: None,
            organisation: Some("Webfeet".to_string()),
            cancelled,
        })
    }
}

fn parse_date(date_str: &str) -> EventTime {
    let start_date = NaiveDate::parse_from_str(&date_str[0..8], "%Y%m%d").unwrap();
    let end_date = if date_str.len() > 8 {
        let end_date_string = format!("{}{}", &date_str[0..17 - date_str.len()], &date_str[9..]);
        NaiveDate::parse_from_str(&end_date_string, "%Y%m%d").unwrap()
    } else {
        start_date
    };
    EventTime::DateOnly {
        start_date,
        end_date,
    }
}

fn convert_style(style: Style) -> Option<DanceStyle> {
    match style {
        Style::Contra | Style::DanceContra | Style::DanceAmericanAmericanContra => {
            Some(DanceStyle::Contra)
        }
        Style::DanceEurobal | Style::DanceEuropean | Style::DanceFrenchBreton => {
            Some(DanceStyle::Balfolk)
        }
        Style::DanceCountryDance => Some(DanceStyle::Playford),
        Style::DanceEnglishCeilidh | Style::DanceCeilidh | Style::DanceCeildh => {
            Some(DanceStyle::EnglishCeilidh)
        }
        Style::DanceEnglishFolk => None, // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dates() {
        assert_eq!(
            parse_date("20210114"),
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd(2021, 1, 14),
                end_date: NaiveDate::from_ymd(2021, 1, 14),
            }
        );
        assert_eq!(
            parse_date("20210114-16"),
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd(2021, 1, 14),
                end_date: NaiveDate::from_ymd(2021, 1, 16),
            }
        );
        assert_eq!(
            parse_date("20210114-0203"),
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd(2021, 1, 14),
                end_date: NaiveDate::from_ymd(2021, 2, 3),
            }
        );
        assert_eq!(
            parse_date("20210114-20220607"),
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd(2021, 1, 14),
                end_date: NaiveDate::from_ymd(2022, 6, 7),
            }
        );
    }
}
