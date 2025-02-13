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

mod types;

use self::types::{Event, EventType};
use crate::{
    model::{
        dancestyle::DanceStyle,
        event::{self, EventTime},
        events::Events,
    },
    util::local_datetime_to_fixed_offset,
};
use chrono::{DateTime, Days, FixedOffset, NaiveDate, NaiveTime};
use chrono_tz::Europe::Brussels;
use eyre::Report;
use std::cmp::Ordering;

/// Times earlier than this will be assumed to be the next day.
#[allow(deprecated)]
const MORNING: NaiveTime = NaiveTime::from_hms(8, 0, 0);

async fn events() -> Result<Vec<Event>, Report> {
    let json = reqwest::get("https://folkbalbende.be/interface/events.php?start=2022-02-01&end=3000-01-01&type=ball,course,festal&image=0").await?.text().await?;
    let mut events: Vec<Event> = serde_json::from_str(&json)?;
    // Sort by ID to give a stable order.
    events.sort_by_key(|event| event.id);
    Ok(events)
}

pub async fn import_events() -> Result<Events, Report> {
    let events = events().await?;

    // Print warnings about cancelled, deleted and unchecked events.
    for event in &events {
        let dates = event
            .dates
            .iter()
            .map(|date| date.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if event.cancelled {
            eprintln!("Cancelled: {} {}", dates, event.name);
        }
        if event.deleted {
            eprintln!("Deleted: {} {}", dates, event.name);
        }
        if !event.checked {
            eprintln!("Not checked: {} {}", dates, event.name);
        }
    }

    Ok(Events {
        events: events
            .iter()
            .flat_map(|event| {
                if event.checked && !event.deleted {
                    convert(event)
                } else {
                    vec![]
                }
            })
            .collect(),
    })
}

fn convert(event: &Event) -> Vec<event::Event> {
    // Filter out "mailto:" URLs and duplicates in non-English languages.
    let mut links: Vec<String> = event
        .websites
        .iter()
        .filter_map(|website| {
            let url = &website.url;
            if url.starts_with("http")
                && !url.starts_with("https://frissefolk.be/fr/civicrm/event/info")
                && !url.starts_with("https://frissefolk.be/nl/civicrm/event/info")
            {
                Some(
                    url.trim()
                        .replace(
                            "nl/event-nl/Danslessen niveau ",
                            "en/event-en/dance-class-level-",
                        )
                        .replace(
                            "nl/event-nl/Folkdans voor beginners - niveau ",
                            "en/event-en/folk-dance-for-beginners-level-",
                        )
                        .replace(
                            "nl/event-nl/Practica in De Pianofabriek",
                            "en/event-en/practica-at-de-pianofabriek",
                        ),
                )
            } else {
                None
            }
        })
        .collect();
    links.push(format!("https://folkbalbende.be/event/{}", event.id));
    if !event.facebook_event.is_empty() {
        links.push(event.facebook_event.trim().to_owned());
    }

    let details = format!("{:?}", event.event_type);

    let mut workshop = event.event_type == EventType::Course || !event.courses.is_empty();
    if let Some(ball) = &event.ball {
        if ball.initiation_start.is_some() || !ball.initiators.is_empty() {
            workshop = true;
        }
    }

    let social = match event.event_type {
        EventType::Course => false,
        EventType::Ball | EventType::Festival => true,
    };

    let price = if event.prices.is_empty() {
        None
    } else {
        let prices: Vec<_> = event
            .prices
            .iter()
            .filter_map(|price| {
                if price.price == 0 {
                    None
                } else {
                    Some(price.price)
                }
            })
            .collect();
        let min_price = prices.iter().min();
        let max_price = prices.iter().max();
        if let (Some(min_price), Some(max_price)) = (min_price, max_price) {
            Some(if *min_price == -1 {
                "donation".to_string()
            } else if min_price == max_price {
                format!("€{}", min_price)
            } else {
                format!("€{}-€{}", min_price, max_price)
            })
        } else {
            None
        }
    };

    let bands = if let Some(ball) = &event.ball {
        ball.performances
            .iter()
            .filter_map(|performance| {
                if performance.band.placeholder || performance.band.name.contains("Practica") {
                    None
                } else {
                    Some(performance.band.name.trim().to_owned())
                }
            })
            .collect()
    } else {
        vec![]
    };

    let organisation = if let Some(organisation) = &event.organisation {
        Some(organisation.name.to_owned())
    } else if links.iter().any(|link| link.contains("eledanse.be")) {
        Some("EléDanse ASBL".to_owned())
    } else if links
        .iter()
        .any(|link| link.contains("folknammusiquetrad.be"))
    {
        Some("Folknam Musique Trad".to_owned())
    } else if links.iter().any(|link| link.contains("tey.be")) {
        Some("Muziekclub 't Ey".to_owned())
    } else if links.iter().any(|link| link.contains("frissefolk.be")) {
        Some("Frisse Folk Vzw/asbl".to_owned())
    } else if links.iter().any(|link| link.contains("tsmiske")) {
        Some("'t Smiske".to_owned())
    } else if links.iter().any(|link| link.contains("balhalla.be")) {
        Some("Balhalla".to_owned())
    } else if links.iter().any(|link| link.contains("rzf.be")) {
        Some("Rif Zans L'Fiesse".to_owned())
    } else if links.iter().any(|link| link.contains("cabalfolk.be")) {
        Some("CaBal".to_owned())
    } else if links.iter().any(|link| link.contains("oldafolk.be")) {
        Some("Ol Da Folk Brugge".to_owned())
    } else if links.iter().any(|link| link.contains("stanistil.be")) {
        Some("Stanistil".to_owned())
    } else if links.iter().any(|link| link.contains("pasdlayau.be")) {
        Some("Pas d'La Yau".to_owned())
    } else if links.iter().any(|link| {
        link.contains("muziekcentrumdranouter.be") | link.contains("dranoutercentrum.be")
    }) {
        Some("Muziekcentrum Dranouter".to_owned())
    } else {
        None
    };

    let (start_time, end_time) = find_start_end_time(event);

    let city = match event.location.address.city.as_str() {
        "Antwerpen" => "Antwerp",
        "Assebroek" => "Bruges",
        "Brugge" => "Bruges",
        "Brussel" => "Brussels",
        "Bruxelles" => "Brussels",
        "Courtrai" => "Kortrijk",
        "Elsene" => "Brussels",
        "Etterbeek" => "Brussels",
        "Gand" => "Gent",
        "Heverlee" => "Leuven",
        "Hombeek" => "Mechelen",
        "Ixelles" => "Brussels",
        "Jette" => "Brussels",
        "Saint-Gilles" => "Brussels",
        "Schoten" => "Antwerp",
        "Sint-Gillis" => "Brussels",
        "Watermael-Boitsfort" => "Watermaal-Bosvoorde",
        "Wijgmaal" => "Leuven",
        "Wilsele" => "Leuven",
        other => other,
    };

    // Remove duplicate dates.
    let mut dates = event.dates.clone();
    dates.dedup();

    let name = event
        .name
        .trim()
        .replace(
            "Practica In De Pianofabriek",
            "Practica at the Pianofabriek",
        )
        .replace("Danslessen Niveau", "Dance Class Level")
        .replace(
            "Folkdans Voor Beginners - Niveau",
            "Folk Dance for Beginners - Level",
        );

    dates
        .iter()
        .map(|&date| event::Event {
            name: name.clone(),
            details: Some(details.clone()),
            links: links.clone(),
            time: make_time(date, start_time, end_time),
            country: "Belgium".to_string(),
            state: None,
            city: city.to_owned(),
            styles: vec![DanceStyle::Balfolk],
            workshop,
            social,
            bands: bands.clone(),
            callers: vec![],
            price: price.clone(),
            organisation: organisation.clone(),
            cancelled: event.cancelled,
            source: None,
        })
        .collect()
}

fn find_start_end_time(event: &Event) -> (Option<NaiveTime>, Option<NaiveTime>) {
    // Find the earliest start time and latest finish time, if any.
    let mut start_times: Vec<NaiveTime> = event.courses.iter().map(|course| course.start).collect();
    let mut end_times: Vec<NaiveTime> = event
        .courses
        .iter()
        .map(|course| course.end.unwrap_or(course.start))
        .collect();
    if let Some(ball) = &event.ball {
        start_times.extend(ball.initiation_start);
        end_times.extend(ball.initiation_end);
        start_times.extend(
            ball.performances
                .iter()
                .flat_map(|performance| performance.start),
        );
        end_times.extend(
            ball.performances
                .iter()
                .flat_map(|performance| performance.end),
        );
    }
    let mut start_time = start_times.into_iter().min_by(compare_times);
    if start_time == Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap()) {
        start_time = None;
    }
    let end_time = end_times.into_iter().max_by(compare_times);
    (start_time, end_time)
}

/// Compares two times, assuming that times before `MORNING` are the next day.
fn compare_times(a: &NaiveTime, b: &NaiveTime) -> Ordering {
    if a < &MORNING && b >= &MORNING {
        Ordering::Greater
    } else if b < &MORNING && a >= &MORNING {
        Ordering::Less
    } else {
        a.cmp(b)
    }
}

fn make_time(
    date: NaiveDate,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
) -> EventTime {
    if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
        if let (Some(start), Some(end)) = (
            combine_date_time(date, start_time),
            combine_date_time(date, end_time),
        ) {
            return EventTime::DateTime { start, end };
        }
    }

    EventTime::DateOnly {
        start_date: date,
        end_date: date,
    }
}

fn combine_date_time(mut date: NaiveDate, time: NaiveTime) -> Option<DateTime<FixedOffset>> {
    if time < MORNING {
        date = date + Days::new(1);
    }
    local_datetime_to_fixed_offset(&date.and_time(time), Brussels)
}

#[cfg(test)]
mod tests {
    use super::types::{Ball, Course, Performance};
    use super::*;

    #[test]
    fn compare_morning() {
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(6, 0, 0).unwrap()
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(10, 0, 0).unwrap()
            ),
            Ordering::Greater
        );
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(12, 0, 0).unwrap()
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(10, 0, 0).unwrap()
            ),
            Ordering::Greater
        );
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(1, 0, 0).unwrap()
            ),
            Ordering::Less
        );
        assert_eq!(
            compare_times(
                &NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
                &NaiveTime::from_hms_opt(23, 0, 0).unwrap()
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn start_end_time() {
        let event = Event {
            checked: true,
            dates: vec![],
            courses: vec![Course {
                start: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                end: Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
                ..Default::default()
            }],
            ball: Some(Ball {
                performances: vec![
                    Performance {
                        start: NaiveTime::from_hms_opt(20, 0, 0),
                        end: NaiveTime::from_hms_opt(21, 0, 0),
                        band: Default::default(),
                    },
                    Performance {
                        start: NaiveTime::from_hms_opt(21, 0, 0),
                        end: NaiveTime::from_hms_opt(23, 0, 0),
                        band: Default::default(),
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };

        let (start_time, end_time) = find_start_end_time(&event);
        assert_eq!(start_time, NaiveTime::from_hms_opt(19, 0, 0));
        assert_eq!(end_time, NaiveTime::from_hms_opt(23, 0, 0));
    }

    #[test]
    fn start_end_time_after_midnight() {
        let event = Event {
            checked: true,
            dates: vec![],
            courses: vec![Course {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
                ..Default::default()
            }],
            ball: Some(Ball {
                performances: vec![
                    Performance {
                        start: NaiveTime::from_hms_opt(22, 0, 0),
                        end: NaiveTime::from_hms_opt(1, 0, 0),
                        band: Default::default(),
                    },
                    Performance {
                        start: NaiveTime::from_hms_opt(2, 0, 0),
                        end: NaiveTime::from_hms_opt(6, 0, 0),
                        band: Default::default(),
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        };

        let (start_time, end_time) = find_start_end_time(&event);
        assert_eq!(start_time, NaiveTime::from_hms_opt(10, 0, 0));
        assert_eq!(end_time, NaiveTime::from_hms_opt(6, 0, 0));
    }
}
