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

mod types;

use self::types::{Event, EventFormat, EventList, InterestTag};
use super::{bands::BANDS, lowercase_matches};
use crate::model::{
    dancestyle::DanceStyle,
    event::{self, EventTime},
    events::Events,
};
use chrono::Timelike;
use eyre::{Report, eyre};

pub async fn events(token: &str) -> Result<Vec<Event>, Report> {
    let json = reqwest::get(format!(
        "https://api1.plug.events/api1/embed/embed1?token={token}"
    ))
    .await?
    .text()
    .await?;
    let events: EventList = serde_json::from_str(&json)?;
    Ok(events.events)
}

pub async fn import_events(token: &str) -> Result<Events, Report> {
    let events = events(token).await?;
    let style = DanceStyle::Balfolk;

    Ok(Events {
        events: events
            .iter()
            .filter_map(|event| convert(event, style).transpose())
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event, default_style: DanceStyle) -> Result<Option<event::Event>, Report> {
    let Some(venue_locale) = &event.venue_locale else {
        eprintln!("Event \"{}\" has no venueLocale, skipping.", event.name);
        return Ok(None);
    };
    let locale_parts: Vec<_> = venue_locale.split(", ").collect();
    let country = locale_parts
        .last()
        .ok_or_else(|| eyre!("venueLocale only has one part: \"{}\"", venue_locale))?
        .to_string();

    let city = if locale_parts.len() > 3 {
        locale_parts[1]
    } else {
        locale_parts[0]
    }
    .to_string();

    let mut workshop = false;
    let mut social = false;
    let mut styles = Vec::new();
    for interest_tag in &event.interest_tags {
        match interest_tag {
            InterestTag::SocialDance => {
                social = true;
            }
            InterestTag::Workshop => {
                workshop = true;
            }
            InterestTag::Balfolk | InterestTag::Balfolkdance | InterestTag::BalfolkMusic => {
                styles.push(DanceStyle::Balfolk);
            }
            InterestTag::SwedishFolkDance | InterestTag::SwedishTraditionalMusic => {
                styles.push(DanceStyle::Scandinavian);
            }
            InterestTag::ContraDance => {
                styles.push(DanceStyle::Contra);
                social = true;
            }
            InterestTag::Art
            | InterestTag::Bachata
            | InterestTag::BalfolkNL
            | InterestTag::BluesDance
            | InterestTag::CommunityService
            | InterestTag::CoupleDance
            | InterestTag::Dance
            | InterestTag::DancingBodies
            | InterestTag::FolkDance
            | InterestTag::FolkMusic
            | InterestTag::ForroDance
            | InterestTag::Fusion
            | InterestTag::Music
            | InterestTag::NeoTrad
            | InterestTag::SalsaDance
            | InterestTag::Tango
            | InterestTag::Teacher
            | InterestTag::TradMusic
            | InterestTag::WestCoastSwing
            | InterestTag::Zouk
            | InterestTag::Zzz => {}
        }
    }
    for subinterest in event.subinterests.clone().unwrap_or_default() {
        match subinterest {
            EventFormat::Bal
            | EventFormat::Balfolk
            | EventFormat::BalfolkNL
            | EventFormat::Concert
            | EventFormat::Dance
            | EventFormat::Folkbal
            | EventFormat::FolkBal => {
                social = true;
            }
            EventFormat::Advanced
            | EventFormat::Class
            | EventFormat::Course
            | EventFormat::DanceClass
            | EventFormat::DansenLeren
            | EventFormat::Dansles
            | EventFormat::Event
            | EventFormat::Intensive
            | EventFormat::Les
            | EventFormat::Learning
            | EventFormat::LessonSeries
            | EventFormat::Workshop => {
                workshop = true;
            }
            EventFormat::Festival => {
                workshop = true;
                social = true;
            }
            EventFormat::Meeting => {
                social = true;
            }
            EventFormat::Organisator | EventFormat::Organiser => {}
            EventFormat::Dansavond
            | EventFormat::Flashmob
            | EventFormat::Jam
            | EventFormat::LiveMusic
            | EventFormat::LiveMuziek
            | EventFormat::Party
            | EventFormat::Piano
            | EventFormat::Social
            | EventFormat::SocialDance
            | EventFormat::SocialDancing => {
                social = true;
            }
            EventFormat::Practica => {
                social = true;
            }
            EventFormat::SocialClass | EventFormat::Sociales => {
                workshop = true;
                social = true;
            }
            EventFormat::BalmuziekLeren
            | EventFormat::Buurtvereniging
            | EventFormat::CommunityAssociation
            | EventFormat::DancingBodies
            | EventFormat::MusicClass
            | EventFormat::Musiekles
            | EventFormat::Optreden
            | EventFormat::Overig
            | EventFormat::Performance
            | EventFormat::Teacher => {}
        }
    }
    if styles.is_empty() {
        styles.push(default_style);
    } else {
        styles.sort();
        styles.dedup();
    }
    if event.name.contains("warsztatów") || event.description.contains("warsztaty") {
        workshop = true;
    }

    let name_lower = event.name.to_lowercase();
    let description_lower = event.description.to_lowercase();
    let mut bands = lowercase_matches(&BANDS, &description_lower, &name_lower);
    bands.extend(
        event
            .featured_participants
            .iter()
            .map(|featured_participant| featured_participant.name.to_owned()),
    );
    bands.sort();
    bands.dedup();

    if name_lower.contains("warsztaty") {
        workshop = true;
    }
    let mut name = event.name.clone();
    match name.as_str() {
        "Balfolk Środowy" => {
            name = "Wednesday Balfolk".to_string();
            social = true;
        }
        "Folktańcówka" => {
            social = true;
        }
        _ => {}
    }

    Ok(Some(event::Event {
        name,
        details: Some(event.description.clone()),
        links: vec![event.plug_url.clone()],
        time: EventTime::DateTime {
            start: event
                .start_date_time_iso
                .with_timezone(&event.timezone)
                .fixed_offset()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap(),
            end: event
                .end_date_time_iso
                .with_timezone(&event.timezone)
                .fixed_offset(),
        },
        country,
        state: None,
        city,
        styles,
        workshop,
        social,
        bands,
        callers: vec![],
        price: format_price(event),
        organisation: event.published_by_name.as_deref().map(fix_organisation),
        cancelled: false,
        source: None,
    }))
}

fn fix_organisation(published_by_name: &str) -> String {
    match published_by_name {
        "Chata Numinosum" | "Numinosum Festival" => "Numinosum".to_string(),
        _ => published_by_name.to_string(),
    }
}

fn format_price(event: &Event) -> Option<String> {
    if event.is_free {
        Some("free".to_string())
    } else {
        event.price_display.as_ref().map(|price| {
            let mut price = price.replace(" ", "");
            let currency = price.chars().next().unwrap();
            if "$£€".contains(currency) {
                price = price.replace("-", &format!("-{currency}"));
            }
            price = price.replace(".00", "");
            if let Some(stripped) = price.strip_prefix("zł") {
                price = format!("{stripped} PLN");
            }
            price
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(&Event::default()), None);
        assert_eq!(
            format_price(&Event {
                price_display: Some("€ 10".to_string()),
                ..Default::default()
            }),
            Some("€10".to_string())
        );
        assert_eq!(
            format_price(&Event {
                price_display: Some("€ 5-23".to_string()),
                ..Default::default()
            }),
            Some("€5-€23".to_string())
        );
        assert_eq!(
            format_price(&Event {
                price_display: Some("zł90.00".to_string()),
                ..Default::default()
            }),
            Some("90 PLN".to_string())
        );
    }
}
