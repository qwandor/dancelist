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

use chrono::{DateTime, FixedOffset, NaiveDateTime, Offset, TimeZone};
use chrono_tz::Tz;

pub const DEFAULT_TIMEZONES: [((&str, Option<&str>), Tz); 5] = [
    (("Austria", None), Tz::Europe__Vienna),
    (("Belgium", None), Tz::Europe__Brussels),
    (("New Zealand", None), Tz::Pacific__Auckland),
    (("UK", None), Tz::Europe__London),
    (("USA", Some("AZ")), Tz::US__Mountain),
];

fn to_fixed_offset(date_time: DateTime<Tz>) -> DateTime<FixedOffset> {
    let fixed_offset = date_time.offset().fix();
    date_time.with_timezone(&fixed_offset)
}

pub fn local_datetime_to_fixed_offset(
    local: &NaiveDateTime,
    timezone: Tz,
) -> Option<DateTime<FixedOffset>> {
    Some(to_fixed_offset(
        timezone.from_local_datetime(local).single()?,
    ))
}
