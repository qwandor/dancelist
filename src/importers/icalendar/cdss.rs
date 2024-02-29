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

use super::{EventParts, IcalendarSource};
use crate::model::{dancestyle::DanceStyle, event::Event};
use eyre::{eyre, Report};

pub struct Cdss;

impl IcalendarSource for Cdss {
    const URL: &'static str = "https://cdss.org/events/list/?ical=1";
    const DEFAULT_ORGANISATION: &'static str = "CDSS";

    fn workshop(parts: &EventParts) -> bool {
        let description_lower = parts.description.to_lowercase();
        (description_lower.contains("lesson") && !description_lower.contains("no lesson"))
            || description_lower.contains("skills session")
            || description_lower.contains("workshops")
            || description_lower.contains("basics session")
            || description_lower.contains("beginner session")
            || description_lower.contains("beginner teaching")
            || description_lower.contains("beginner workshop")
            || description_lower.contains("beginners workshop")
            || description_lower.contains("beginners introduction")
            || description_lower.contains("beginners’ workshop")
            || description_lower.contains("introductory session")
            || description_lower.contains("introductory workshop")
            || description_lower.contains("intro session")
            || description_lower.contains("newcomers’ session")
            || description_lower.contains("newcomers workshop")
    }

    fn social(_parts: &EventParts) -> bool {
        true
    }

    fn styles(parts: &EventParts) -> Vec<DanceStyle> {
        let categories = parts.categories.as_deref().unwrap_or_default();
        let summary_lowercase = parts.summary.to_lowercase();

        if categories.iter().any(|category| category == "Online Event")
            || summary_lowercase.contains("online")
        {
            // Ignore online events.
            return vec![];
        }

        let mut styles = Vec::new();
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

    fn location(
        location_parts: &Option<Vec<String>>,
    ) -> Result<Option<(String, Option<String>, String)>, Report> {
        let location_parts = location_parts
            .as_ref()
            .ok_or_else(|| eyre!("Event missing location."))?;
        if location_parts.len() < 3 {
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
        Ok(Some((country, state, city)))
    }

    fn fixup(mut event: Event) -> Option<Event> {
        event.name = shorten_name(&event.name);
        apply_fixes(&mut event);
        Some(event)
    }
}

fn shorten_name(name: &str) -> String {
    name.trim_start_matches("Portland Country Dance Community ")
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
        .trim_end_matches("—Richmond, VT")
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

/// Apply fixes for specific event series.
fn apply_fixes(event: &mut Event) {
    match event.name.as_str() {
        "3rd Saturday Contra Dance" if event.city == "Philadelphia" => {
            event
                .links
                .insert(0, "https://3rdsaturday.thursdaycontra.com/".to_string());
        }
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
        "Carolina English Country Dancers Saturday Dance" => {
            event.name = "Carolina English Country Dancers".to_string();
            event
                .links
                .insert(0, "https://carolinaenglishcountrydance.com/".to_string());
        }
        "CDK Contra Dance" | "CDK Contra & Square Dance" => {
            event
                .links
                .insert(0, "https://www.countrydancinginkalamazoo.com/".to_string());
        }
        "Contra Dance" if event.city == "Carrollton" && event.state.as_deref() == Some("TX") => {
            event.links.insert(0, "https://www.nttds.org/".to_string());
        }
        "ContraATL Weekly Dance" => {
            event.workshop = true;
            event
                .links
                .insert(0, "https://contradance.org/".to_string());
        }
        "Denver Contra Dance" => {
            event
                .links
                .insert(0, "https://www.cfootmad.org/".to_string());
        }
        "ECD Atlanta Regular Dance" => {
            event
                .links
                .insert(0, "https://ecdatlanta.org/schedule.htm".to_string());
        }
        "English Country Dance" if event.city == "Asheville" => {
            event.links.insert(
                0,
                "https://oldfarmersball.com/english-country-dance/".to_string(),
            );
        }
        "Fourth Friday Experienced Contra at Guiding Star Grange" => {
            event.name = "Experienced Contra at Guiding Star Grange".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "Friday Contra Dance, Nashville TN" => {
            event.name = "Friday Contra Dance".to_string();
            event.links.insert(
                0,
                "https://www.nashvillecountrydancers.org/contra-dances".to_string(),
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
        "Houston Area Traditional Dance Society 3rd Sunday English Country Dance" => {
            event.name = "3rd Sunday English Country Dance".to_string();
            event
                .links
                .insert(0, "https://hatds.org/ecd#hatds".to_string());
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
        "Mystic Pie Dance" => {
            event
                .links
                .insert(0, "https://www.mysticpiedance.org/".to_string());
            if event.price.as_deref() == Some("$3-$10") {
                event.price = Some("$7-$10".to_string());
            }
        }
        "Nashville Second Sunday English Country Dances" => {
            event.links.insert(
                0,
                "https://www.nashvillecountrydancers.org/english-country-dances".to_string(),
            );
        }
        "North Alabama Country Dance Society - Contra Dance" => {
            event.name = "NACDS Contra Dance".to_string();
        }
        "North Alabama Country Dance Society - NACDS Contra Dance" => {
            event.name = "NACDS Contra Dance".to_string();
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
        "Sacramento English Country Dance (Third Sunday)" => {
            event.name = "Sacramento English Country Dance".to_string();
            event
                .links
                .insert(0, "https://sactocds.wordpress.com/".to_string());
        }
        "Sacramento (CA) Contra Dance, 2nd and 4th Saturdays" => {
            event.name = "Sacramento Contra Dance".to_string();
            event
                .links
                .insert(0, "https://sactocds.wordpress.com/".to_string());
        }
        "Second Saturday TopHill Music Contradance Party at Guiding Star Grange" => {
            event.name = "TopHill Music Contradance Party".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "Second/Fourth Wednesday English Country Dance at Guiding Star Grange" => {
            event.name = "English Country Dance at Guiding Star Grange".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "Space Coast Contra Dance" => {
            event.links.insert(
                0,
                "https://spacecoastcontra.org/calendar-upcoming-contra-dances/".to_string(),
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
        "York Region English Country Dancers" => {
            event.links.insert(0, "http://www.yrecd.ca/".to_string());
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
        assert_eq!(Cdss::location(&Some(vec![])).unwrap(), None);
        assert_eq!(
            Cdss::location(&Some(vec!["USA".to_string()])).unwrap(),
            None
        );
        assert_eq!(
            Cdss::location(&Some(vec!["CA".to_string(), "USA".to_string()])).unwrap(),
            None
        );
        assert_eq!(
            Cdss::location(&Some(vec![
                "123 Some Street".to_string(),
                "Hayward".to_string(),
                "CA".to_string(),
                "94541".to_string(),
                "USA".to_string(),
            ]))
            .unwrap(),
            Some((
                "USA".to_string(),
                Some("CA".to_string()),
                "Hayward".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&Some(vec![
                "Pittsburgh".to_string(),
                "PA".to_string(),
                "1234".to_string(),
                "USA".to_string(),
            ]))
            .unwrap(),
            Some((
                "USA".to_string(),
                Some("PA".to_string()),
                "Pittsburgh".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&Some(vec![
                "Toronto".to_string(),
                "Ontario".to_string(),
                "1234".to_string(),
                "Canada".to_string(),
            ]))
            .unwrap(),
            Some((
                "Canada".to_string(),
                Some("Ontario".to_string()),
                "Toronto".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&Some(vec![
                "London".to_string(),
                "N10AB".to_string(),
                "United Kingdom".to_string()
            ]))
            .unwrap(),
            Some(("UK".to_string(), None, "London".to_string()))
        );
        assert_eq!(
            Cdss::location(&Some(vec![
                "Venue Name".to_string(),
                "Address".to_string(),
                "London".to_string(),
                "N10AB".to_string(),
                "United Kingdom".to_string()
            ]))
            .unwrap(),
            Some(("UK".to_string(), None, "London".to_string()))
        );
    }
}
