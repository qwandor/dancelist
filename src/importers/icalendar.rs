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

pub mod balfolknl;
pub mod cdss;

use crate::{
    model::{
        event::{self, EventTime},
        events::Events,
    },
    util::local_datetime_to_fixed_offset,
};
use eyre::{bail, eyre, Report};
use icalendar::{
    Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, Event, EventLike,
};

/// Fetches the iCalendar file from the given URL, then converts events from it using the given
/// `convert` function.
async fn import_events(
    url: &str,
    convert: impl Fn(EventParts) -> Result<Option<event::Event>, Report>,
) -> Result<Events, Report> {
    let calendar = reqwest::get(url)
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
                    match get_parts(event) {
                        Ok(parts) => convert(parts).transpose(),
                        Err(e) => Some(Err(e)),
                    }
                } else {
                    None
                }
            })
            .collect::<Result<_, _>>()?,
    })
}

fn get_parts(event: &Event) -> Result<EventParts, Report> {
    let url = event
        .get_url()
        .ok_or_else(|| eyre!("Event {:?} missing url.", event))?
        .to_owned();
    let summary = unescape(
        event
            .get_summary()
            .ok_or_else(|| eyre!("Event {:?} missing summary.", event))?,
    );
    let description = unescape(
        event
            .get_description()
            .ok_or_else(|| eyre!("Event {:?} missing description.", event))?,
    );
    let time = get_time(event)?;
    let location_parts = event.get_location().map(|location| {
        location
            .split("\\, ")
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    });
    let organiser = if let Some(organiser) = event.properties().get("ORGANIZER") {
        let organiser_name = organiser
            .params()
            .get("CN")
            .ok_or_else(|| eyre!("Event {:?} missing organiser name", event))?
            .value();
        Some(organiser_name[1..organiser_name.len() - 1].to_owned())
    } else {
        None
    };
    let categories = get_categories(event);
    Ok(EventParts {
        url,
        summary,
        description,
        time,
        location_parts,
        organiser,
        categories,
    })
}

fn get_categories(event: &Event) -> Option<Vec<String>> {
    Some(
        event
            .multi_properties()
            .get("CATEGORIES")?
            .first()?
            .value()
            .split(',')
            .map(ToOwned::to_owned)
            .collect(),
    )
}

/// Returns strings from the slice which are contained in one of the two lowercase strings passed.
fn lowercase_matches(needles: &[&str], a: &str, b: &str) -> Vec<String> {
    needles
        .iter()
        .filter_map(|needle| {
            let needle_lower = needle.to_lowercase();
            if a.contains(&needle_lower) || b.contains(&needle_lower) {
                Some(needle.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EventParts {
    pub url: String,
    pub summary: String,
    pub description: String,
    pub time: EventTime,
    pub location_parts: Option<Vec<String>>,
    pub organiser: Option<String>,
    pub categories: Option<Vec<String>>,
}

fn get_time(event: &Event) -> Result<EventTime, Report> {
    let start = event
        .get_start()
        .ok_or_else(|| eyre!("Event {:?} missing start time.", event))?;
    let end = event
        .get_end()
        .ok_or_else(|| eyre!("Event {:?} missing end time.", event))?;
    Ok(match (start, end) {
        (DatePerhapsTime::Date(start_date), DatePerhapsTime::Date(end_date)) => {
            EventTime::DateOnly {
                start_date,
                // iCalendar DTEND is non-inclusive, so subtract one day.
                end_date: end_date.pred_opt().unwrap(),
            }
        }
        (
            DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone {
                date_time: start,
                tzid: start_tzid,
            }),
            DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone {
                date_time: end,
                tzid: end_tzid,
            }),
        ) => {
            let start_timezone = start_tzid
                .parse()
                .map_err(|e| eyre!("Invalid timezone: {}", e))?;
            let end_timezone = end_tzid
                .parse()
                .map_err(|e| eyre!("Invalid timezone: {}", e))?;
            EventTime::DateTime {
                start: local_datetime_to_fixed_offset(&start, start_timezone)
                    .ok_or_else(|| eyre!("Ambiguous datetime for event {:?}", event))?,
                end: local_datetime_to_fixed_offset(&end, end_timezone)
                    .ok_or_else(|| eyre!("Ambiguous datetime for event {:?}", event))?,
            }
        }
        _ => bail!("Mismatched start and end times."),
    })
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

    use crate::model::event::EventTime;
    use chrono::{FixedOffset, TimeZone};
    use icalendar::Property;

    #[test]
    fn parse_datetime() {
        let start = Property::new("DTSTART", "20220401T190000")
            .add_parameter("TZID", "Europe/Amsterdam")
            .done();
        let end = Property::new("DTEND", "20220401T190000")
            .add_parameter("TZID", "Europe/Amsterdam")
            .done();
        let event = Event::new()
            .append_property(start)
            .append_property(end)
            .done();

        assert_eq!(
            get_time(&event).unwrap(),
            EventTime::DateTime {
                start: FixedOffset::east_opt(7200)
                    .unwrap()
                    .with_ymd_and_hms(2022, 4, 1, 19, 0, 0)
                    .single()
                    .unwrap(),
                end: FixedOffset::east_opt(7200)
                    .unwrap()
                    .with_ymd_and_hms(2022, 4, 1, 19, 0, 0)
                    .single()
                    .unwrap(),
            }
        );
    }
}
