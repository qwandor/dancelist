use crate::model::event::Event;
use chrono::{Date, Utc};
use icalendar::{Calendar, Component};
use std::fmt::Write;

pub fn events_to_calendar(events: &[&Event]) -> Calendar {
    events
        .iter()
        .map(|event| event_to_event(event))
        .collect::<Calendar>()
        .name("Folk dance events")
        .done()
}

fn event_to_event(event: &Event) -> icalendar::Event {
    let mut description = String::new();
    if let Some(details) = &event.details {
        writeln!(description, "{}", details).unwrap();
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
        writeln!(description, "Price: {}", price).unwrap();
    }

    icalendar::Event::new()
        .summary(&event.name)
        // TODO: Use proper timezones rather than assuming everything is UTC.
        .start_date(Date::<Utc>::from_utc(event.start_date, Utc))
        .end_date(Date::<Utc>::from_utc(event.end_date, Utc))
        .location(&format!("{}, {}", event.city, event.country))
        .description(&description)
        .done()
}
