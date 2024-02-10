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

use super::super::{BANDS, CALLERS};
use super::{lowercase_matches, EventParts};
use crate::model::{dancestyle::DanceStyle, event::Event, events::Events};
use eyre::{eyre, Report, WrapErr};
use log::error;
use regex::Regex;
use std::cmp::{max, min};

pub async fn import_events() -> Result<Events, Report> {
    super::import_events("https://cdss.org/events/list/?ical=1", convert).await
}

fn convert(parts: EventParts) -> Result<Option<Event>, Report> {
    let categories = parts.categories.as_deref().unwrap_or_default();
    let name = shorten_name(&parts.summary);

    let summary_lowercase = parts.summary.to_lowercase();
    let styles = get_styles(categories, &parts.summary);
    if categories.iter().any(|category| category == "Online Event")
        || summary_lowercase.contains("online")
    {
        return Ok(None);
    }
    if styles.is_empty() {
        return Ok(None);
    }

    let location_parts = parts
        .location_parts
        .as_ref()
        .ok_or_else(|| eyre!("Event {:?} missing location.", parts))?;
    let Some((country, state, city)) = parse_location(location_parts) else {
        error!("Invalid location {:?} for {}", location_parts, parts.url);
        return Ok(None);
    };

    let organisation = Some(parts.organiser.unwrap_or_else(|| "CDSS".to_owned()));

    let price = get_price(&parts.description)?;

    let description_lower = parts.description.to_lowercase();
    let summary_lower = parts.summary.to_lowercase();
    let bands = lowercase_matches(&BANDS, &description_lower, &summary_lower);
    let callers = lowercase_matches(&CALLERS, &description_lower, &summary_lower);

    let workshop = (description_lower.contains("lesson")
        && !description_lower.contains("no lesson"))
        || description_lower.contains("skills session")
        || description_lower.contains("workshops")
        || description_lower.contains("beginner workshop")
        || description_lower.contains("beginners workshop")
        || description_lower.contains("introductory session")
        || description_lower.contains("introductory workshop")
        || description_lower.contains("intro session");

    let details = if parts.description.is_empty() {
        None
    } else {
        Some(parts.description)
    };

    let mut event = Event {
        name,
        details,
        links: vec![parts.url],
        time: parts.time,
        country,
        state,
        city,
        styles,
        workshop,
        social: true,
        bands,
        callers,
        price,
        organisation,
        cancelled: false,
        source: None,
    };
    apply_fixes(&mut event);
    Ok(Some(event))
}

/// Converts location parts to (country, state, city)
fn parse_location(location_parts: &[String]) -> Option<(String, Option<String>, String)> {
    if location_parts.len() < 3 {
        return None;
    }
    let mut country = location_parts[location_parts.len() - 1].to_owned();
    if country == "United States" {
        country = "USA".to_owned();
    } else if country == "United Kingdom" {
        country = "UK".to_owned();
    }
    let (state, city) = if ["Canada", "USA"].contains(&country.as_str()) {
        (
            Some(location_parts[location_parts.len() - 3].to_owned()),
            location_parts[location_parts.len() - 4].to_owned(),
        )
    } else {
        (None, location_parts[location_parts.len() - 3].to_owned())
    };
    Some((country, state, city))
}

fn shorten_name(summary: &str) -> String {
    summary
        .trim_start_matches("Portland Country Dance Community ")
        .trim_start_matches("Contra Dance with ")
        .trim_end_matches(" - Asheville NC")
        .trim_end_matches(" (Masks Optional)")
        .trim_end_matches(" of Macon County, NC")
        .trim_end_matches(" in Dallas")
        .trim_end_matches(" in Peterborough, NH")
        .trim_end_matches(" in Philadelphia")
        .trim_end_matches(" in Carrollton, TX")
        .trim_end_matches(" in Nelson, NH")
        .trim_end_matches(" in Van Nuys")
        .replace("Berkeley, CA", "Berkeley")
        .replace("Dover NH", "Dover")
        .replace("Richmond VA", "Richmond")
        .replace("Richmond, VA", "Richmond")
        .replace("Rochester, NY", "Rochester")
        .replace("Hayward CA", "Hayward")
        .replace("Hayward, CA", "Hayward")
        .replace("Lancaster, PA", "Lancaster")
        .replace("Williamsburg (VA)", "Williamsburg")
        .to_owned()
}

fn get_styles(categories: &[String], summary: &str) -> Vec<DanceStyle> {
    let mut styles = Vec::new();
    let summary_lowercase = summary.to_lowercase();
    if categories.iter().any(|category| category == "Contra Dance") {
        styles.push(DanceStyle::Contra);
    }
    if categories
        .iter()
        .any(|category| category == "English Country Dance")
    {
        styles.push(DanceStyle::EnglishCountryDance);
    }
    if summary_lowercase.contains("bal folk") || summary_lowercase.contains("balfolk") {
        styles.push(DanceStyle::Balfolk);
    }
    if summary_lowercase.contains("contra") {
        styles.push(DanceStyle::Contra);
    }
    styles.sort();
    styles.dedup();
    styles
}

/// Figure out price from description.
fn get_price(description: &str) -> Result<Option<String>, Report> {
    let price_regex = Regex::new(r"\$([0-9]+)").unwrap();
    let mut min_price = u32::MAX;
    let mut max_price = u32::MIN;
    for capture in price_regex.captures_iter(description) {
        let price: u32 = capture
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .wrap_err("Invalid price")?;
        min_price = min(price, min_price);
        max_price = max(price, max_price);
    }
    Ok(if min_price == u32::MAX {
        None
    } else if min_price == max_price {
        Some(format!("${}", min_price))
    } else {
        Some(format!("${}-${}", min_price, max_price))
    })
}

/// Apply fixes for specific event series.
fn apply_fixes(event: &mut Event) {
    match event.name.as_str() {
        "Anaheim Contra Dance" => {
            event.links.insert(
                0,
                "https://www.thelivingtradition.org/tltbodydance.html".to_string(),
            );
        }
        "Capital English Country Dancers" => {
            event.links.insert(
                0,
                "https://www.danceflurry.org/series/capital-english-country-dancers/".to_string(),
            );
        }
        "CDK Contra Dance" => {
            event
                .links
                .insert(0, "https://www.countrydancinginkalamazoo.com/".to_string());
        }
        "Contra Dance" if event.city == "Carrollton" && event.state.as_deref() == Some("TX") => {
            event.links.insert(0, "https://www.nttds.org/".to_string());
        }
        "Denver Contra Dance" => {
            event
                .links
                .insert(0, "https://www.cfootmad.org/".to_string());
        }
        "Fourth Friday Experienced Contra at Guiding Star Grange" => {
            event.name = "Experienced Contra at Guiding Star Grange".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "Friday Night Contra & Square Dance" => {
            event
                .links
                .insert(0, "https://fsgw.org/Friday-contra-square-dance".to_string());
        }
        "Goshen Community Contra Dance" => {
            event.links.insert(0, "http://godancing.org/".to_string());
            if event.price.as_deref() == Some("$3-$18") {
                event.price = Some("$3-$8".to_string());
            }
        }
        "Hayward Contra Dance" => {
            event
                .links
                .insert(0, "https://sfbaycontra.org/".to_string());
        }
        "Indy Contra Dance" => {
            event
                .links
                .insert(0, "https://www.indycontra.org/".to_string());
        }
        "Lancaster Contra Dance" => {
            event
                .links
                .insert(0, "https://lancastercontra.org/".to_string());
        }
        "Montpelier Contra Dance" => {
            event.links.insert(
                0,
                "https://capitalcitygrange.org/dancing/contradancing/".to_string(),
            );
        }
        "North Alabama Country Dance Society - Contra Dance" => {
            event.name = "North Alabama Country Dance Society".to_string();
        }
        "Orlando Contra Dance" => {
            event.links.insert(
                0,
                "https://orlandocontra.org/dances-and-events/".to_string(),
            );
        }
        "Ottawa Contra Dance" => {
            event
                .links
                .insert(0, "https://ottawacontra.ca/".to_string());
        }
        "Pittsburgh Contra Dance" => {
            event
                .links
                .insert(0, "https://pittsburghcontra.org/".to_string());
        }
        "Quiet Corner Contra Dance" => {
            event
                .links
                .insert(0, "http://www.hcdance.org/quiet-corner-contra/".to_string());
        }
        "Richmond Wednesday English Country Dance" => {
            event.links.insert(
                0,
                "https://colonialdanceclubofrichmond.com/english-dance-calendar".to_string(),
            );
        }
        "Second/Fourth Wednesday English Country Dance at Guiding Star Grange" => {
            event.name = "English Country Dance at Guiding Star Grange".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "TECDA Friday Evening Dance" | "TECDA Tuesday Evening English Country Dance" => {
            event
                .links
                .insert(0, "https://www.tecda.ca/weekly_dances.html".to_string());
        }
        "Third Sunday English Regency Dancing" => {
            event.links.insert(
                0,
                "https://www.valleyareaenglishregencysociety.org/".to_string(),
            );
        }
        "Thursday Contra Dance" if event.city == "Philadelphia" => {
            event
                .links
                .insert(0, "https://thursdaycontra.com/".to_string());
        }
        "Williamsburg Tuesday Night English Dance" => {
            event
                .links
                .insert(0, "https://williamsburgheritagedancers.org/".to_string());
        }
        _ => {}
    }

    if event.city == "401 Chapman St" && event.state.as_deref() == Some("Greenfield") {
        event.city = "Greenfield".to_string();
        event.state = Some("MA".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_location() {
        assert_eq!(parse_location(&[]), None);
        assert_eq!(parse_location(&["USA".to_string()]), None);
        assert_eq!(parse_location(&["CA".to_string(), "USA".to_string()]), None);
        assert_eq!(
            parse_location(&[
                "123 Some Street".to_string(),
                "Hayward".to_string(),
                "CA".to_string(),
                "94541".to_string(),
                "USA".to_string(),
            ]),
            Some((
                "USA".to_string(),
                Some("CA".to_string()),
                "Hayward".to_string(),
            ))
        );
        assert_eq!(
            parse_location(&[
                "Pittsburgh".to_string(),
                "PA".to_string(),
                "1234".to_string(),
                "USA".to_string(),
            ]),
            Some((
                "USA".to_string(),
                Some("PA".to_string()),
                "Pittsburgh".to_string(),
            ))
        );
        assert_eq!(
            parse_location(&[
                "Toronto".to_string(),
                "Ontario".to_string(),
                "1234".to_string(),
                "Canada".to_string(),
            ]),
            Some((
                "Canada".to_string(),
                Some("Ontario".to_string()),
                "Toronto".to_string(),
            ))
        );
        assert_eq!(
            parse_location(&[
                "London".to_string(),
                "N10AB".to_string(),
                "United Kingdom".to_string()
            ]),
            Some(("UK".to_string(), None, "London".to_string()))
        );
        assert_eq!(
            parse_location(&[
                "Venue Name".to_string(),
                "Address".to_string(),
                "London".to_string(),
                "N10AB".to_string(),
                "United Kingdom".to_string()
            ]),
            Some(("UK".to_string(), None, "London".to_string()))
        );
    }
}
