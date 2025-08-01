use crate::model::event::{Event, EventTime};
use axum::{
    body::Body,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use icalendar::{Calendar, Component, EventLike, EventStatus};
use std::fmt::Write;

pub fn events_to_calendar(events: &[Event], name: &str) -> Calendar {
    events
        .iter()
        .map(|event| event_to_event(event))
        .collect::<Calendar>()
        .name(name)
        .done()
}

fn event_to_event(event: &Event) -> icalendar::Event {
    let mut description = String::new();
    if let Some(details) = &event.details {
        writeln!(description, "{details}").unwrap();
    }
    writeln!(
        description,
        "Dance styles: {}",
        event
            .styles
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();
    writeln!(
        description,
        "{}",
        match (event.workshop, event.social) {
            (true, false) => "Workshop only.",
            (true, true) => "Workshop and social dance.",
            (false, true) => "Social dance only.",
            (false, false) => "",
        }
    )
    .unwrap();
    if !event.bands.is_empty() {
        writeln!(description, "Bands: {}", event.bands.join(", ")).unwrap();
    }
    if !event.callers.is_empty() {
        writeln!(description, "Callers: {}", event.callers.join(", ")).unwrap();
    }
    if let Some(price) = &event.price {
        writeln!(description, "Price: {price}").unwrap();
    }
    for link in &event.links {
        writeln!(description, "{link}").unwrap();
    }

    let categories = event
        .styles
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    let mut calendar_event = icalendar::Event::new();
    calendar_event
        .summary(&event.name)
        .location(&format!("{}, {}", event.city, event.country))
        .description(&description)
        .status(if event.cancelled {
            EventStatus::Cancelled
        } else {
            EventStatus::Confirmed
        })
        .add_property("CATEGORIES", &categories);
    match event.time {
        EventTime::DateOnly {
            start_date,
            end_date,
        } => {
            calendar_event
                .starts(start_date)
                // iCalendar DTEND is non-inclusive, so add one day.
                .ends(end_date.succ_opt().unwrap());
        }
        EventTime::DateTime { start, end } => {
            calendar_event
                .starts(start.with_timezone(&Utc))
                .ends(end.with_timezone(&Utc));
        }
    }
    for link in &event.links {
        calendar_event.add_multi_property("ATTACH", link);
    }
    calendar_event
}

#[derive(Debug)]
pub struct Ics(pub Calendar);

impl IntoResponse for Ics {
    fn into_response(self) -> Response {
        let mut res = Response::new(Body::from(self.0.to_string()));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/calendar"),
        );
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body;
    use chrono::TimeZone;

    #[tokio::test]
    async fn empty() {
        let calendar = Calendar::new();
        let ics = Ics(calendar);
        let response = ics.into_response();
        assert_eq!(
            body::to_bytes(response.into_body(), 2000).await.unwrap(),
            "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:ICALENDAR-RS\r
CALSCALE:GREGORIAN\r
END:VCALENDAR\r
"
        );
    }

    #[tokio::test]
    async fn description_newline() {
        let calendar = Calendar::new()
            .push(
                icalendar::Event::new()
                    .uid("20fabeca-4caa-45b3-8e65-aff7b21c4779")
                    .timestamp(Utc.with_ymd_and_hms(2025, 2, 22, 1, 2, 3).unwrap())
                    .description("foo\nbar")
                    .done(),
            )
            .done();
        let ics = Ics(calendar);
        let response = ics.into_response();
        assert_eq!(
            body::to_bytes(response.into_body(), 2000).await.unwrap(),
            "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:ICALENDAR-RS\r
CALSCALE:GREGORIAN\r
BEGIN:VEVENT\r
DESCRIPTION:foo\\nbar\r
DTSTAMP:20250222T010203Z\r
UID:20fabeca-4caa-45b3-8e65-aff7b21c4779\r
END:VEVENT\r
END:VCALENDAR\r
"
        );
    }
}
