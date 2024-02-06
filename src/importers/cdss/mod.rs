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

use super::icalendar_utils::{get_time, unescape};
use crate::model::{dancestyle::DanceStyle, event, events::Events};
use eyre::{eyre, Report, WrapErr};
use icalendar::{Calendar, CalendarComponent, Component, Event, EventLike};
use log::error;
use regex::Regex;
use std::cmp::{max, min};

const BANDS: [&str; 56] = [
    "AlleMonOh Stringband",
    "Aubergine",
    "Bare Necessities",
    "Ben Bolker and Susanne Maziarz",
    "Big Fun",
    "Brook Farm String Band",
    "Bunny Bread Bandits",
    "Calico",
    "Chimney Swift",
    "Cojiro",
    "Contraverts",
    "Dead Sea Squirrels",
    "Devilish Mary",
    "Dogtown",
    "Elixir",
    "Eloise & Co.",
    "First Time Stringband",
    "Good Intentions",
    "GrayScale",
    "Headwaters",
    "Joyance",
    "Kingfisher",
    "Lackawanna Longnecks",
    "Lake Effect",
    "Larks in the Attic",
    "Liberty String Band",
    "Lone Star Pirates",
    "Long Forgotten String Band",
    "Mevilish Merry",
    "Nova",
    "Playing with Fyre",
    "Pont Ondulé",
    "Red Case Band",
    "River Road",
    "River Music",
    "Serendipity",
    "Smith, Campeau & Nelson",
    "Snappin' Bug Stringband",
    "Spintuition",
    "SpringTide",
    "Starling",
    "Stomp Rocket",
    "Supertrad",
    "Swingology",
    "Take a Dance",
    "The Dam Beavers",
    "The Fiddling Thomsons",
    "The Flying Elbows",
    "The Free Raisins",
    "The Gaslight Tinkers",
    "The Turning Stile",
    "Unbowed",
    "Warleggan Village Band",
    "Wee Merry Banshees",
    "Wheels of the World",
    "Wild Asparagus",
];
const CALLERS: [&str; 59] = [
    "Adina Gordon",
    "Alan Rosenthal",
    "Alice Raybourn",
    "Andrew Swaine",
    "Barrett Grimm",
    "Ben Sachs-Hamilton",
    "Bob Frederking",
    "Billy Fischer",
    "Bob Isaacs",
    "Bridget Whitehead",
    "Bronwyn Chelette",
    "Cathy Campbell",
    "Christine Merryman",
    "Cindy Harris",
    "Dan Blim",
    "Darlene Underwood",
    "Dave Berman",
    "Dave Smukler",
    "Don Heinold",
    "Don Veino",
    "Dorothy Cummings",
    "Gaye Fifer",
    "George Marshall",
    "George Thompson",
    "Janine Smith",
    "Jen Jasenski",
    "Joanna Reiner Wilkinson",
    "John Krumm",
    "Jordan Kammeyer",
    "Kalia Kliban",
    "Katy Heine",
    "Ken Gall",
    "Laura Beraha",
    "Lindsey Dono",
    "Lisa Greenleaf",
    "Liz Nelson",
    "Maeve Devlin",
    "Marc Airhart",
    "Marlin Whitaker",
    "Martha Kent",
    "Mary Wesley",
    "Michael Karchar",
    "Nils Fredland",
    "Orly Krasner",
    "Paul Wilde",
    "Rich MacMath",
    "Rick Szumski",
    "River Abel",
    "Steph West",
    "Steve Zakon-Anderson",
    "Susan English",
    "Susie Kendig",
    "Tara Bolker",
    "Tod Whittemore",
    "Tom Greene",
    "Val Medve",
    "Vicki Morrison",
    "Walter Zagorski",
    "Will Mentor",
];

pub async fn import_events() -> Result<Events, Report> {
    let calendar = reqwest::get("https://cdss.org/events/list/?ical=1")
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
                    convert(event).transpose()
                } else {
                    None
                }
            })
            .collect::<Result<_, _>>()?,
    })
}

fn convert(event: &Event) -> Result<Option<event::Event>, Report> {
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

    let categories = event
        .multi_properties()
        .get("CATEGORIES")
        .ok_or_else(|| eyre!("Event {:#?} missing categories.", event))?
        .first()
        .ok_or_else(|| eyre!("Event {:#?} has empty categories.", event))?
        .value()
        .split(',')
        .collect::<Vec<_>>();

    let name = summary
        .trim_start_matches("Portland Country Dance Community ")
        .trim_start_matches("Contra Dance with ")
        .trim_end_matches(" - Asheville NC")
        .trim_end_matches(" (Masks Optional)")
        .trim_end_matches(" of Macon County, NC")
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
        .to_owned();

    let mut styles = Vec::new();
    let summary_lowercase = summary.to_lowercase();
    if categories.contains(&"Online Event") || summary_lowercase.contains("online") {
        return Ok(None);
    }
    if categories.contains(&"Contra Dance") {
        styles.push(DanceStyle::Contra);
    }
    if categories.contains(&"English Country Dance") {
        styles.push(DanceStyle::EnglishCountryDance);
    }
    if summary_lowercase.contains("bal folk") || summary_lowercase.contains("balfolk") {
        styles.push(DanceStyle::Balfolk);
    }
    if styles.is_empty() {
        return Ok(None);
    }

    let location = event
        .get_location()
        .ok_or_else(|| eyre!("Event {:?} missing location.", event))?;
    let location_parts = location.split("\\, ").collect::<Vec<_>>();
    if location_parts.len() < 3 {
        error!("Invalid location {:?} for {}", location_parts, url);
        return Ok(None);
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

    let organisation = Some(
        if let Some(organiser) = event.properties().get("ORGANIZER") {
            let organiser_name = organiser
                .params()
                .get("CN")
                .ok_or_else(|| eyre!("Event {:?} missing organiser name", event))?
                .value();
            organiser_name[1..organiser_name.len() - 1].to_owned()
        } else {
            "CDSS".to_owned()
        },
    );

    // Figure out price from description.
    let price_regex = Regex::new(r"\$([0-9]+)").unwrap();
    let mut min_price = u32::MAX;
    let mut max_price = u32::MIN;
    for capture in price_regex.captures_iter(&description) {
        let price: u32 = capture
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .wrap_err("Invalid price")?;
        min_price = min(price, min_price);
        max_price = max(price, max_price);
    }
    let price = if min_price == u32::MAX {
        None
    } else if min_price == max_price {
        Some(format!("${}", min_price))
    } else {
        Some(format!("${}-${}", min_price, max_price))
    };

    let bands = BANDS
        .iter()
        .filter_map(|band| {
            if description.contains(band) || summary.contains(band) {
                Some(band.to_string())
            } else {
                None
            }
        })
        .collect();
    let callers = CALLERS
        .iter()
        .filter_map(|caller| {
            if description.contains(caller) || summary.contains(caller) {
                Some(caller.to_string())
            } else {
                None
            }
        })
        .collect();

    let description_lower = description.to_lowercase();
    let workshop = (description_lower.contains("lesson")
        && !description_lower.contains("no lesson"))
        || description_lower.contains("skills session")
        || description_lower.contains("workshops")
        || description_lower.contains("beginners workshop")
        || description_lower.contains("introductory session")
        || description_lower.contains("introductory workshop")
        || description_lower.contains("intro session");

    let details = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    let mut event = event::Event {
        name,
        details,
        links: vec![url],
        time,
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

/// Apply fixes for specific event series.
fn apply_fixes(event: &mut event::Event) {
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
