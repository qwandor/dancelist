// Copyright 2024 the dancelist authors.
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

use super::{EventParts, IcalendarSource, import_new_events};
use crate::{
    importers::combine_events,
    model::{
        dancestyle::DanceStyle,
        event::{Event, EventTime},
        events::Events,
    },
    util::local_datetime_to_fixed_offset,
};
use chrono_tz::Tz;
use eyre::Report;
use regex::Regex;

const ORGANISATION: &str = "Folktanz Dresden e.V.";

/// Imports events from both Dresden sources, preserving the given previously imported events if
/// appropriate.
pub async fn import_events(old_events: Events) -> Result<Events, Report> {
    let mut new_events = import_new_events::<Dresden>().await?;
    new_events
        .events
        .extend(import_new_events::<DresdenWeekly>().await?.events);
    new_events.sort();
    Ok(combine_events(old_events, new_events))
}

struct Dresden;

impl IcalendarSource for Dresden {
    const URLS: &'static [&'static str] =
        &["https://www.gugelhupf-dresden.de/tanz-in-dresden/calendar/icslist/calendar.ics"];
    const DEFAULT_ORGANISATION: &'static str = ORGANISATION;
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/Berlin");

    fn workshop(parts: &EventParts) -> bool {
        let summary_lower = parts.summary.to_lowercase();
        summary_lower.contains("tanzfest")
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        let city = if parts.summary.contains("Hohnstein") {
            "Hohnstein"
        } else {
            "Dresden"
        };
        Ok(Some(("Germany".to_string(), None, city.to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        common_fixup(&mut event);
        event.links.insert(
            0,
            "https://www.gugelhupf-dresden.de/tanz-in-dresden/".to_string(),
        );
        Some(event)
    }

    fn fix_before_parse(source: String) -> String {
        let source = Regex::new("\n(u.a. mit.*|mit .*)\nORGANIZER")
            .unwrap()
            .replace_all(&source, "\nDESCRIPTION:$1\nORGANIZER");
        source.into_owned()
    }
}

struct DresdenWeekly;

impl IcalendarSource for DresdenWeekly {
    const URLS: &'static [&'static str] =
        &["https://www.gugelhupf-dresden.de/tanz-am-dienstag/calendar/icslist/calendar.ics"];
    const DEFAULT_ORGANISATION: &'static str = ORGANISATION;
    const DEFAULT_TIMEZONE: Option<&'static str> = Some("Europe/Berlin");

    fn workshop(_parts: &EventParts) -> bool {
        true
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(_parts: &EventParts) -> Vec<DanceStyle> {
        vec![DanceStyle::Balfolk]
    }

    fn location(_parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        Ok(Some(("Germany".to_string(), None, "Dresden".to_string())))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        common_fixup(&mut event);
        event.links.insert(
            0,
            "https://www.gugelhupf-dresden.de/tanz-am-dienstag/".to_string(),
        );
        Some(event)
    }

    fn fix_before_parse(source: String) -> String {
        let source = Regex::new("(Einführungsstunde:.*)\nORGANIZER")
            .unwrap()
            .replace_all(&source, "DESCRIPTION:$1\nORGANIZER");
        let source = Regex::new("(Einführungsstunde:.*)\n(.*)\nORGANIZER")
            .unwrap()
            .replace_all(&source, "DESCRIPTION:$1\\n$2\nORGANIZER");
        source.into_owned()
    }
}

fn common_fixup(event: &mut Event) {
    event.organisation = Some(ORGANISATION.to_string());
    if let EventTime::DateTime { start, end } = &mut event.time {
        // Fix times, they claim to be in UTC but are actually local time.
        *start = local_datetime_to_fixed_offset(&start.naive_utc(), Tz::Europe__Berlin)
            .expect("Error fixing start time");
        *end = local_datetime_to_fixed_offset(&end.naive_utc(), Tz::Europe__Berlin)
            .expect("Error fixing end time");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_description() {
        assert_eq!(DresdenWeekly::fix_before_parse("".to_string()), "");
        assert_eq!(DresdenWeekly::fix_before_parse("foo".to_string()), "foo");
        assert_eq!(
            DresdenWeekly::fix_before_parse(
                "Einführungsstunde: foo\nORGANIZER;CN=\"Henry\":\n".to_string()
            ),
            "DESCRIPTION:Einführungsstunde: foo\nORGANIZER;CN=\"Henry\":\n"
        );
        assert_eq!(
            DresdenWeekly::fix_before_parse(
                "Einführungsstunde:\nfoo\nORGANIZER;CN=\"Henry\":\n".to_string()
            ),
            "DESCRIPTION:Einführungsstunde:\\nfoo\nORGANIZER;CN=\"Henry\":\n"
        );
    }
}
