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

use std::collections::BTreeMap;

use super::to_fixed_offset;
use crate::model::{
    dancestyle::DanceStyle,
    event::{self, EventTime},
    events::Events,
};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use chrono_tz::Europe::Amsterdam;
use eyre::{eyre, Context, Report};
use icalendar::{Calendar, CalendarComponent, Component, Event, Property};
use log::{info, warn};

pub async fn import_events() -> Result<Events, Report> {
    let calendar = reqwest::get("https://www.balfolk.nl/events.ics")
        .await?
        .text()
        .await?
        .parse::<Calendar>()
        .map_err(|e| eyre!("Error parsing iCalendar file: {}", e))?;

    Ok(Events {
        events: calendar
            .iter()
            .filter_map(|component| {
                if let CalendarComponent::Event(event) = component {
                    convert(event).transpose()
                } else {
                    None
                }
            })
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event) -> Result<Option<event::Event>, Report> {
    let properties = event.properties();

    let url = get_property_value(properties, "URL")?;

    let summary = get_property_value(properties, "SUMMARY")?.replace("\\,", ",");
    // Remove city from end of summary.
    let name = summary.rsplitn(2, ",").last().unwrap().to_owned();

    // Try to skip music workshops.
    if name.starts_with("Muziekstage") {
        info!("Skipping \"{}\" {}", name, url);
        return Ok(None);
    }

    let description = unescape(&get_property_value(properties, "DESCRIPTION")?);
    // Remove name from start of description
    let details = description
        .trim_start_matches(&format!("{}, ", name))
        .to_owned();
    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let dtstart = get_property(properties, "DTSTART")?;
    let dtend = get_property(properties, "DTEND")?;
    let time = if dtstart.value().len() > 8 {
        EventTime::DateTime {
            start: convert_datetime(dtstart)?,
            end: convert_datetime(dtend)?,
        }
    } else {
        EventTime::DateOnly {
            start_date: convert_date(dtstart)?,
            end_date: convert_date(dtend)?,
        }
    };

    let location = get_property_value(properties, "LOCATION")?;
    let location_parts = location.split("\\, ").collect::<Vec<_>>();
    let city = if location_parts.len() < 5 {
        warn!("Invalid location \"{}\" for {}", location, url);
        "".to_string()
    } else {
        location_parts[2].to_string()
    };

    let workshop = name.contains("Fundamentals")
        || name.contains("Basis van")
        || name == "DenneFeest"
        || description.contains("Dansworkshop")
        || description.contains("Workshopbeschrijving")
        || description.contains("Workshop ")
        || description.contains("dans uitleg")
        || description.contains("dansuitleg")
        || description.contains(" leren ")
        || description.contains("Vooraf dansuitleg")
        || description.contains("de Docent");
    let social = name.contains("Social dance")
        || name.contains("Balfolkbal")
        || name.contains("Avondbal")
        || name.contains("Bal in")
        || name.contains("Balfolk Bal")
        || name.starts_with("Balfolk Wilhelmina")
        || name.starts_with("Fest Noz")
        || name.starts_with("Verjaardagsbal")
        || name == "Balfolk café Nijmegen"
        || name == "DenneFeest"
        || description.contains("Bal deel");

    Ok(Some(event::Event {
        name,
        details,
        links: vec![url],
        time,
        country: "Netherlands".to_string(),
        city,
        styles: vec![DanceStyle::Balfolk],
        workshop,
        social,
        bands: vec![],
        callers: vec![],
        price: None,
        organisation: Some("balfolk.nl".to_string()),
        cancelled: false,
    }))
}

fn convert_datetime(property: &Property) -> Result<DateTime<FixedOffset>, Report> {
    let amsterdam_datetime = Amsterdam
        .datetime_from_str(property.value(), "%Y%m%dT%H%M%S")
        .wrap_err_with(|| format!("Error parsing datetime {:?}", property))?;
    Ok(to_fixed_offset(amsterdam_datetime))
}

fn convert_date(property: &Property) -> Result<NaiveDate, Report> {
    NaiveDate::parse_from_str(property.value(), "%Y%m%d")
        .wrap_err_with(|| format!("Error parsing date {:?}", property))
}

fn get_property_value(
    properties: &BTreeMap<String, Property>,
    property_name: &str,
) -> Result<String, Report> {
    Ok(get_property(properties, property_name)?.value().to_owned())
}

fn get_property<'a>(
    properties: &'a BTreeMap<String, Property>,
    property_name: &str,
) -> Result<&'a Property, Report> {
    properties
        .get(property_name)
        .ok_or_else(|| eyre!("Event {:?} missing {}.", properties, property_name))
}

fn unescape(s: &str) -> String {
    s.replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\n", "\n")
        .replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_datetime() {
        let property = Property::new("DTSTART", "20220401T190000")
            .add_parameter("TZID", "Europe/Amsterdam")
            .done();

        assert_eq!(
            convert_datetime(&property).unwrap(),
            FixedOffset::east(7200).ymd(2022, 4, 1).and_hms(19, 0, 0)
        );
    }
}
