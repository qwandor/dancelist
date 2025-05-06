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
pub mod boulder;
pub mod bristolcontra;
pub mod burghausen;
pub mod cdss;
pub mod ceilidhclub;
pub mod contrabridge;
pub mod dresden;
pub mod freiburg;
pub mod kalender;
pub mod lancastercontra;
pub mod marburg;
pub mod skandia;
pub mod spreefolk;

use super::{bands::BANDS, callers::CALLERS, combine_events, lowercase_matches};
use crate::{
    model::{
        dancestyle::DanceStyle,
        event::{self, EventTime},
        events::Events,
    },
    util::to_fixed_offset,
};
use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Utc};
use chrono_tz::Tz;
use eyre::{Report, WrapErr, bail, eyre};
use icalendar::{
    Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime, Event, EventLike,
};
use log::{debug, error};
use regex::Regex;
use rrule::RRule;
use std::cmp::{max, min};

const MAX_FUTURE_RECURRENCES: TimeDelta = TimeDelta::days(365);

trait IcalendarSource {
    const URLS: &'static [&'static str];
    const DEFAULT_ORGANISATION: &'static str;
    const DEFAULT_TIMEZONE: Option<&'static str> = None;

    /// Returns whether the event includes a workshop.
    fn workshop(parts: &EventParts) -> bool;

    /// Returns whether the event includes a social.
    fn social(parts: &EventParts) -> bool;

    /// Returns which dance styles the event includes.
    fn styles(parts: &EventParts) -> Vec<DanceStyle>;

    /// Converts location parts to (country, state, city).
    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report>;

    /// Returns links for the event.
    fn links(parts: &EventParts) -> Vec<String> {
        parts.url.clone().into_iter().collect()
    }

    /// Applies any further changes to the event after conversion, or returns `None` to skip it.
    fn fixup(event: event::Event) -> Option<event::Event>;

    /// Applies fixes to iCalendar file before parsing.
    fn fix_before_parse(source: String) -> String {
        source
    }
}

fn convert<S: IcalendarSource>(parts: EventParts) -> Result<Option<event::Event>, Report> {
    let styles = S::styles(&parts);
    if styles.is_empty() {
        return Ok(None);
    }

    let workshop = S::workshop(&parts);
    let social = S::social(&parts);
    let Some((country, state, city)) =
        S::location(&parts).wrap_err_with(|| format!("For event {:?}", parts))?
    else {
        error!(
            "Invalid location {:?} for {:?} '{}'",
            parts.location_parts, parts.url, parts.summary
        );
        return Ok(None);
    };
    let links = S::links(&parts);
    let price = get_price(&parts.description)?;
    let description_lower = parts.description.to_lowercase();
    let summary_lower = parts.summary.to_lowercase();
    let bands = lowercase_matches(&BANDS, &description_lower, &summary_lower);
    let callers = lowercase_matches(&CALLERS, &description_lower, &summary_lower);

    let details = parts.description.trim().to_owned();
    let details = if details.is_empty() {
        None
    } else {
        Some(details)
    };

    let organisation = Some(
        parts
            .organiser
            .unwrap_or_else(|| S::DEFAULT_ORGANISATION.to_owned()),
    );

    Ok(S::fixup(event::Event {
        name: parts.summary.trim().to_owned(),
        details,
        links,
        time: parts.time,
        country,
        state,
        city,
        styles,
        workshop,
        social,
        bands,
        callers,
        price,
        organisation,
        cancelled: false,
        source: None,
    }))
}

/// Figure out price from description.
fn get_price(description: &str) -> Result<Option<String>, Report> {
    let price_regexes = [
        ("$", Regex::new(r"\$([0-9]+)").unwrap()),
        ("£", Regex::new(r"£([0-9]+)").unwrap()),
        ("€", Regex::new(r"€([0-9]+)").unwrap()),
        ("€", Regex::new(r"€ ([0-9]+)").unwrap()),
        ("€", Regex::new(r"([0-9]+) €").unwrap()),
        ("€", Regex::new(r"([0-9]+) Euro").unwrap()),
    ];
    for (currency, regex) in price_regexes {
        let mut min_price = u32::MAX;
        let mut max_price = u32::MIN;
        for capture in regex.captures_iter(description) {
            let price: u32 = capture
                .get(1)
                .unwrap()
                .as_str()
                .parse()
                .wrap_err("Invalid price")?;
            min_price = min(price, min_price);
            max_price = max(price, max_price);
        }
        if min_price == u32::MAX {
            continue;
        } else if min_price == max_price {
            return Ok(Some(format!("{}{}", currency, min_price)));
        } else {
            return Ok(Some(format!(
                "{}{}-{}{}",
                currency, min_price, currency, max_price,
            )));
        }
    }
    Ok(None)
}

/// Imports events from the given source, preserving the given previously imported events if
/// appropriate.
#[allow(private_bounds)]
pub async fn import_events<S: IcalendarSource>(old_events: Events) -> Result<Events, Report> {
    let new_events = import_new_events::<S>().await?;
    Ok(combine_events(old_events, new_events))
}

/// Fetches the iCalendar file for the given source, then converts events from it.
async fn import_new_events<S: IcalendarSource>() -> Result<Events, Report> {
    let mut events = Events::default();
    for url in S::URLS {
        let calendar = S::fix_before_parse(reqwest::get(*url).await?.text().await?)
            .parse::<Calendar>()
            .map_err(|e| eyre!("Error parsing iCalendar file: {}", e))?;
        let timezone = calendar.get_timezone().or(S::DEFAULT_TIMEZONE);
        for component in calendar.iter() {
            if let CalendarComponent::Event(event) = component {
                for parts in get_parts(event, timezone)? {
                    events.events.extend(convert::<S>(parts)?);
                }
            }
        }
    }
    events.sort();
    events.events.dedup();
    Ok(events)
}

fn get_parts(event: &Event, timezone: Option<&str>) -> Result<Vec<EventParts>, Report> {
    let url = event.get_url().map(|url| {
        if url.contains("://") {
            url.to_string()
        } else {
            format!("http://{}", url)
        }
    });
    let summary = unescape(event.get_summary().unwrap_or_default());
    let description = unescape(event.get_description().unwrap_or_default());
    let times = get_times(event, timezone)?;
    let location_parts = event.get_location().map(|location| {
        location
            .split(", ")
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    });
    let organiser = if let Some(organiser) = event.properties().get("ORGANIZER") {
        let organiser_name = organiser
            .params()
            .get("CN")
            .ok_or_else(|| eyre!("Event {:?} missing organiser name", event))?
            .value();
        Some(organiser_name.to_owned())
    } else if let Some(attendee) = event
        .multi_properties()
        .get("ATTENDEE")
        .and_then(|attendees| attendees.first())
    {
        Some(attendee.value().to_owned())
    } else {
        None
    };
    let categories = get_categories(event);
    let uid = event.get_uid().map(ToOwned::to_owned);
    Ok(times
        .into_iter()
        .map(|time| EventParts {
            url: url.clone(),
            summary: summary.clone(),
            description: description.clone(),
            time,
            location_parts: location_parts.clone(),
            organiser: organiser.clone(),
            categories: categories.clone(),
            uid: uid.clone(),
        })
        .collect())
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct EventParts {
    pub url: Option<String>,
    pub summary: String,
    pub description: String,
    pub time: EventTime,
    pub location_parts: Option<Vec<String>>,
    pub organiser: Option<String>,
    pub categories: Option<Vec<String>>,
    pub uid: Option<String>,
}

impl Default for EventParts {
    fn default() -> Self {
        Self {
            url: Default::default(),
            summary: Default::default(),
            description: Default::default(),
            time: EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            },
            location_parts: Default::default(),
            organiser: Default::default(),
            categories: Default::default(),
            uid: Default::default(),
        }
    }
}

fn datetime_instances(
    rrule: Option<&str>,
    start_with_tz: DateTime<Tz>,
    end_with_tz: DateTime<Tz>,
) -> Result<Vec<EventTime>, Report> {
    if let Some(rrule) = rrule {
        let duration = end_with_tz - start_with_tz;
        debug!("raw rrule: {}", rrule);
        let rrule: RRule<_> = rrule.parse()?;

        debug!("start: {}", start_with_tz);
        match rrule.build(start_with_tz.with_timezone(&rrule::Tz::from(start_with_tz.timezone()))) {
            Ok(rruleset) => {
                debug!("rruleset: {}", rruleset);
                let max_datetime = Utc::now() + MAX_FUTURE_RECURRENCES;
                Ok(rruleset
                    .into_iter()
                    .map_while(|instance| {
                        debug!("Instance: {}", instance);
                        if instance > max_datetime {
                            None
                        } else {
                            Some(EventTime::DateTime {
                                start: to_fixed_offset(instance),
                                end: to_fixed_offset(instance + duration),
                            })
                        }
                    })
                    .collect())
            }
            Err(e) => {
                error!("Error building rruleset: {}", e);
                Ok(vec![])
            }
        }
    } else {
        Ok(vec![EventTime::DateTime {
            start: to_fixed_offset(start_with_tz),
            end: to_fixed_offset(end_with_tz),
        }])
    }
}

fn get_times(event: &Event, timezone: Option<&str>) -> Result<Vec<EventTime>, Report> {
    let start = event
        .get_start()
        .ok_or_else(|| eyre!("Event {:?} missing start time.", event))?;
    let end = event
        .get_end()
        .ok_or_else(|| eyre!("Event {:?} missing end time.", event))?;
    let rrule = event.property_value("RRULE");

    match (start, end) {
        (DatePerhapsTime::Date(start_date), DatePerhapsTime::Date(end_date)) => {
            // TODO: Support RRULE for date-only events too.
            Ok(vec![EventTime::DateOnly {
                start_date,
                // iCalendar DTEND is non-inclusive, so subtract one day.
                end_date: end_date.pred_opt().unwrap(),
            }])
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
            let start_timezone: Tz = start_tzid
                .parse()
                .map_err(|e| eyre!("Invalid timezone: {}", e))?;
            let end_timezone: Tz = end_tzid
                .parse()
                .map_err(|e| eyre!("Invalid timezone: {}", e))?;
            let start_with_tz = start_timezone
                .from_local_datetime(&start)
                .earliest()
                .ok_or_else(|| eyre!("Ambiguous start datetime for event {:?}", event))?;
            let end_with_tz = end_timezone
                .from_local_datetime(&end)
                .earliest()
                .ok_or_else(|| eyre!("Ambiguous end datetime for event {:?}", event))?;

            datetime_instances(rrule, start_with_tz, end_with_tz)
        }
        (
            DatePerhapsTime::DateTime(CalendarDateTime::Utc(start)),
            DatePerhapsTime::DateTime(CalendarDateTime::Utc(end)),
        ) => {
            let timezone = timezone.ok_or_else(|| {
                eyre!(
                    "Neither event nor calendar specified timezone: {:?}.",
                    event
                )
            })?;
            let timezone: Tz = timezone
                .parse()
                .map_err(|e| eyre!("Invalid timezone {}: {}", timezone, e))?;
            let start_with_tz = start.with_timezone(&timezone);
            let end_with_tz = end.with_timezone(&timezone);
            datetime_instances(rrule, start_with_tz, end_with_tz)
        }
        (
            DatePerhapsTime::DateTime(CalendarDateTime::Floating(start)),
            DatePerhapsTime::DateTime(CalendarDateTime::Floating(end)),
        ) => {
            let timezone = timezone.ok_or_else(|| {
                eyre!(
                    "Neither event nor calendar specified timezone: {:?}.",
                    event
                )
            })?;
            let timezone: Tz = timezone
                .parse()
                .map_err(|e| eyre!("Invalid timezone {}: {}", timezone, e))?;
            let start_with_tz = start
                .and_local_timezone(timezone)
                .single()
                .ok_or_else(|| eyre!("Ambiguous local time {}", start))?;
            let end_with_tz = end
                .and_local_timezone(timezone)
                .single()
                .ok_or_else(|| eyre!("Ambiguous local time {}", start))?;
            datetime_instances(rrule, start_with_tz, end_with_tz)
        }
        (start, end) => bail!("Mismatched start ({:?}) and end ({:?}) times.", start, end),
    }
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
            get_times(&event, None).unwrap(),
            vec![EventTime::DateTime {
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
            }]
        );
    }

    #[test]
    fn parse_datetime_utc() {
        let start = Property::new("DTSTART", "20220401T170000Z").done();
        let end = Property::new("DTEND", "20220401T170000Z").done();
        let event = Event::new()
            .append_property(start)
            .append_property(end)
            .done();

        assert_eq!(
            get_times(&event, Some("Europe/Amsterdam")).unwrap(),
            vec![EventTime::DateTime {
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
            }]
        );
    }

    #[test]
    fn parse_datetime_weekly_tzid() {
        // Start before daylight savings starts, but recur into daylight savings time, including one
        // event that spans the start.
        let start = Property::new("DTSTART", "20220312T190000")
            .add_parameter("TZID", "Europe/Amsterdam")
            .done();
        let end = Property::new("DTEND", "20220313T040000")
            .add_parameter("TZID", "Europe/Amsterdam")
            .done();
        let event = Event::new()
            .append_property(start)
            .append_property(end)
            .add_property("RRULE", "FREQ=WEEKLY;UNTIL=20220403T000000Z")
            .done();

        assert_eq!(
            get_times(&event, None).unwrap(),
            vec![
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 12, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 13, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 19, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 20, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 26, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 27, 5, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 4, 2, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 4, 3, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
            ]
        );
    }

    #[test]
    fn parse_datetime_weekly_utc() {
        let start = Property::new("DTSTART", "20220312T180000Z").done();
        let end = Property::new("DTEND", "20220313T030000Z").done();
        let event = Event::new()
            .append_property(start)
            .append_property(end)
            .add_property("RRULE", "FREQ=WEEKLY;UNTIL=20220403T000000Z")
            .done();

        assert_eq!(
            get_times(&event, Some("Europe/Amsterdam")).unwrap(),
            vec![
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 12, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 13, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 19, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 20, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(3600)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 26, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 3, 27, 5, 0, 0)
                        .single()
                        .unwrap(),
                },
                EventTime::DateTime {
                    start: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 4, 2, 19, 0, 0)
                        .single()
                        .unwrap(),
                    end: FixedOffset::east_opt(7200)
                        .unwrap()
                        .with_ymd_and_hms(2022, 4, 3, 4, 0, 0)
                        .single()
                        .unwrap(),
                },
            ]
        );
    }
}
