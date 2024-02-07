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

use crate::{model::event::EventTime, util::local_datetime_to_fixed_offset};
use eyre::{bail, eyre, Report};
use icalendar::{CalendarDateTime, Component, DatePerhapsTime, Event};

pub fn get_time(event: &Event) -> Result<EventTime, Report> {
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

pub fn unescape(s: &str) -> String {
    s.replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\n", "\n")
        .replace("&amp;", "&")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&nbsp;", " ")
}
