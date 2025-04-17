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
use eyre::Report;
use log::warn;

pub struct Cdss;

impl IcalendarSource for Cdss {
    const URLS: &'static [&'static str] = &[
        "https://cdss.org/events/list/?ical=1",
        "https://cdss.org/events/list/page/2/?ical=1",
        "https://cdss.org/events/list/page/3/?ical=1",
        "https://cdss.org/events/list/page/4/?ical=1",
        "https://cdss.org/events/list/page/5/?ical=1",
        "https://cdss.org/events/list/page/6/?ical=1",
        "https://cdss.org/events/list/page/7/?ical=1",
        "https://cdss.org/events/list/page/8/?ical=1",
        "https://cdss.org/events/list/page/9/?ical=1",
        "https://cdss.org/events/list/page/10/?ical=1",
        "https://cdss.org/events/list/page/11/?ical=1",
        "https://cdss.org/events/list/?tribe_eventcategory%5B0%5D=143&ical=1",
        "https://cdss.org/events/list/?tribe_eventcategory%5B0%5D=162&ical=1",
    ];
    const DEFAULT_ORGANISATION: &'static str = "CDSS";

    fn workshop(parts: &EventParts) -> bool {
        let description_lower = parts.description.to_lowercase();
        (description_lower.contains("lesson") && !description_lower.contains("no lesson"))
            || description_lower.contains("“learn the ropes")
            || description_lower.contains("[learn the ropes")
            || description_lower.contains("basics session")
            || description_lower.contains("basics/review session")
            || description_lower.contains("beginner class")
            || description_lower.contains("beginner introduction")
            || description_lower.contains("beginner session")
            || description_lower.contains("beginner teaching")
            || description_lower.contains("beginner workshop")
            || description_lower.contains("beginner’s session")
            || description_lower.contains("beginner’s workshop")
            || description_lower.contains("beginners introduction")
            || description_lower.contains("beginners session")
            || description_lower.contains("beginners workshop")
            || description_lower.contains("beginners’ workshop")
            || description_lower.contains("beginning workshop")
            || description_lower.contains("class on the basics")
            || description_lower.contains("dance workshop")
            || description_lower.contains("intro session")
            || description_lower.contains("intro/refresher workshop")
            || description_lower.contains("introduction or review")
            || description_lower.contains("introduction to english dance basics")
            || description_lower.contains("introductory session")
            || description_lower.contains("introductory workshop")
            || description_lower.contains("new dancer intro")
            || description_lower.contains("newcomer session")
            || description_lower.contains("newcomers")
            || description_lower.contains("refresher session")
            || description_lower.contains("skills class")
            || description_lower.contains("skills session")
            || description_lower.contains("teaching at")
            || description_lower.contains("workshop")
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

    fn location(parts: &EventParts) -> Result<Option<(String, Option<String>, String)>, Report> {
        let Some(location_parts) = parts.location_parts.as_ref() else {
            warn!("Event missing location.");
            return Ok(None);
        };
        if location_parts.len() < 3 {
            return Ok(Some((
                "USA".to_owned(),
                location_parts.get(1).cloned(),
                location_parts.get(0).cloned().unwrap_or_default(),
            )));
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
        if (event.name == "NACDS Contra Dance"
            && event
                .details
                .as_deref()
                .unwrap_or_default()
                .starts_with("We meet in the gym at Faith Presbyterian Church"))
            || event
                .name
                .starts_with("Second Friday Bread & Roses Fusion Dance")
        {
            // Filter out bogus duplicate events.
            return None;
        }
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
        .trim_end_matches(", Vermont")
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
    let details_lower = event.details.as_deref().unwrap_or_default().to_lowercase();
    if event.price.is_none() && details_lower.contains("admission by donation") {
        event.price = Some("donation".to_string());
    }

    match (event.city.as_str(), event.state.as_deref()) {
        ("Henrico", Some("VA")) => {
            event.city = "Richmond".to_string();
        }
        _ => {}
    }
    match event.name.as_str() {
        "2nd Saturdays Contra Dance" | "4th Saturdays Contra Dance" if event.city == "Portland" => {
            event
                .links
                .insert(0, "https://portlandcountrydance.org/upcoming/".to_string());
        }
        "3rd Friday Hot Springs NC Contradance" => {
            event.name = "Hot Springs Contradance".to_string();
        }
        "3rd Saturday Contra Dance" if event.city == "Philadelphia" => {
            event
                .links
                .insert(0, "https://3rdsaturday.thursdaycontra.com/".to_string());
        }
        "3rd Saturdays Contra Dance, Grass Valley CA" => {
            event.name = "3rd Saturdays Contra Dance".to_string();
            event.links.insert(
                0,
                "https://www.kvmr.org/kvmr-event/third-saturdays-contra-dance-nevada-county/"
                    .to_string(),
            );
            event
                .links
                .insert(0, "https://www.contradancenc.org/".to_string());
        }
        "4th Saturday Kingston, NH, Contra Dance!" => {
            event.name = "Kingston Contra Dance".to_string();
        }
        "All-Ages Community Barn Dance" => {
            event.links.insert(
                0,
                "https://fvfs.org/WordPress/events/barn-dance/".to_string(),
            );
        }
        "Anaheim Contra Dance" => {
            event.links.insert(
                0,
                "https://www.thelivingtradition.org/tltbodydance.html".to_string(),
            );
        }
        "Ann Arbor Tuesday English Country Dance" => {
            event.name = "Tuesday English Country Dance".to_string();
            event
                .links
                .insert(0, "https://aactmad.org/english-country".to_string());
        }
        "Asheville Sunday Afternoon English Country Dance"
        | "Asheville Wednesday Evening English Country Dance" => {
            event.name = "English Country Dance".to_string();
            event.links.insert(
                0,
                "https://oldfarmersball.com/english-country-dance/".to_string(),
            );
        }
        "Ashland Country Dancers - English Country Dance" => {
            event.name = "Ashland Country Dancers".to_string();
            event.links.insert(
                0,
                "http://www.heatherandrose.org/activities/ongoing.shtml".to_string(),
            );
        }
        "BACDS Peninsula English Country Dance" => {
            event.links.insert(
                0,
                "https://www.bacds.org/series/english/peninsula/".to_string(),
            );
        }
        "Baltimore Folk Music Society Wednesday Contra Dance" => {
            event
                .links
                .insert(0, "https://www.bfms.org/squarecontra.php".to_string());
        }
        "Baton Rouge Contra Dance" => {
            event.links.insert(
                0,
                "https://louisianacontrasandsquares.com/events.html".to_string(),
            );
        }
        "Berea 4th Saturday English Country Dance" => {
            event.links.insert(
                0,
                "https://www.berea-folk-circle.org/english-country-dance".to_string(),
            );
        }
        "Blacksburg Contra Dance" => {
            event.links.insert(
                0,
                "https://blacksburgcontradance.com/contradance.html".to_string(),
            );
            event.workshop = true;
        }
        "Bloomington Contra Dance" => {
            event.links.insert(
                0,
                "https://bloomingtoncontra.org/wednesday-dances/".to_string(),
            );
        }
        "Buena Vista 1st Saturday Community Contra & Band Jam" => {
            event.name = "Buena Vista Community Contra & Band Jam".to_string();
            event
                .links
                .insert(0, "http://www.arkvalleymusicanddance.org/".to_string());
            event.state = Some("CO".to_string());
            event.city = "Buena Vista".to_string();
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
        "Central Illinois English Country Dance" => {
            event
                .links
                .insert(0, "http://www.centralillinoisecd.org/#sched".to_string());
        }
        "Chicago Contra Dance" | "Monday Night Contra & Square Dance" => {
            event
                .links
                .insert(0, "https://www.chicagobarndance.org/calendar/".to_string());
        }
        "Childgrove English Country Dance" => {
            event
                .links
                .insert(0, "https://www.childgrove.org/".to_string());
        }
        "Circle Left" => {
            event.links.insert(
                0,
                "https://www.queercontradance.org/circleleft.html".to_string(),
            );
        }
        "Cleveland Thursday English Country Dance"
        | "Cleveland Second Friday English Country Dance" => {
            event
                .links
                .insert(0, "https://englishcountryorg.wordpress.com/".to_string());
        }
        "Common Floor Contra Dance" => {
            event
                .links
                .insert(0, "https://www.commonfloorcontra.dance/".to_string());
        }
        "Concord NH English Country Dance"
        | "Concord NH 1st Sunday English Country Dance series" => {
            event.name = "Concord English Country Dance".to_string();
            event.links.insert(
                0,
                "https://manylives-oneworld.com/dave-bateman/nhecds/".to_string(),
            );
        }
        "Contra & Square Dance - Grand River Folk Art Society" => {
            event.name = "Contra & Square Dance".to_string();
            if matches!(event.organisation.as_deref(), None | Some("CDSS")) {
                event.organisation = Some("Grand River Folk Art Society".to_string());
            }
            event.links.insert(
                0,
                "https://sites.google.com/view/grandriverfolkarts/dances".to_string(),
            );
        }
        "Contra Dance" if event.city == "Carrollton" && event.state.as_deref() == Some("TX") => {
            event.links.insert(0, "https://www.nttds.org/".to_string());
        }
        "Contra Dance" if event.city == "Colorado Springs" => {
            event
                .links
                .insert(0, "https://www.pptdcontra.com/".to_string());
        }
        "Contra Dance at Tapestry Folkdance Center" => {
            event
                .links
                .insert(0, "https://www.tapestryfolkdance.org/contra".to_string());
            event.workshop = true;
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
        "Dover 2nd Friday New England Traditional Dance" => {
            event.name = "Dover New England Traditional Dance".to_string();
            event.links.insert(
                0,
                "https://manylives-oneworld.com/dave-bateman/dover-dance/".to_string(),
            );
        }
        "Dover English Country Dancers - Monthly Dance" => {
            event.name = "Dover Monthly English Country Dance".to_string();
            event
                .links
                .insert(0, "http://www.doverdancers.org/".to_string());
            event.state = Some("DE".to_string());
            event.country = "USA".to_string();
        }
        "East Derry Contra Dance!" => {
            event
                .links
                .insert(0, "https://www.wordworthy2.org/".to_string());
            event.name = "East Derry Contra Dance".to_string();
        }
        "ECD Atlanta Regular Dance" | "English Country Dance Atlanta" => {
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
        "English Country Dance" | "2nd Saturday English Country Dance"
            if event.city == "Dallas" =>
        {
            event.name = "English Country Dance".to_string();
            event.links.insert(0, "https://www.nttds.org/".to_string());
        }
        "English Country Dance" if event.city == "Richmond" => {
            event.links.insert(
                0,
                "http://burlingtoncountrydancers.org/english-country-dance-series/".to_string(),
            );
        }
        "English Country Dance - Norwich, VT" => {
            event.name = "English Country Dance".to_string();
        }
        "First Saturday Contra at Guiding Star Grange"
        | "Third Saturday Contra at Guiding Star Grange"
        | "Third Friday Contra at Guiding Star Grange"
        | "Fifth Friday Contra at Guiding Star Grange" => {
            event.name = "Contra at Guiding Star Grange".to_string();
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "English Country Dance at Tapestry Folkdance Center" => {
            event.workshop = false;
            event.links.insert(
                0,
                "https://www.tapestryfolkdance.org/english-country-dance".to_string(),
            );
        }
        "Columbia (SC) Contra Dance"
        | "Columbia Contra Dance at Arsenal Hill Center"
        | "Columbia Contra Dance at Arsenal Hill Park"
        | "Contra Dance at Lake Murray Contra Hall" => {
            event.name = "Columbia Contra Dance".to_string();
            event
                .links
                .insert(0, "https://www.contracola.org/".to_string());
        }
        "Contra at Guiding Star Grange" => {
            event.links.insert(
                0,
                "https://www.guidingstargrange.org/events.html".to_string(),
            );
        }
        "Contra for a Cause"
        | "Contra for a Cause!"
        | "Contra Dance for a Cause"
        | "Contra Dance for a Cause!" => {
            event.name = "Contra for a Cause".to_string();
            event
                .links
                .insert(0, "https://www.kofc109.com/contra".to_string());
        }
        "Contra Dance in Shelburne, VT" => {
            event.name = "Queen City Contra".to_string();
            event
                .links
                .insert(0, "https://queencitycontras.com/schedule".to_string());
        }
        "Contra Dance in St Louis" => {
            event
                .links
                .insert(0, "https://www.childgrove.org/".to_string());
        }
        "Contra Dancing in Houston, TX" => {
            event.name = "Contra Dance".to_string();
            event.links.insert(0, "https://hatds.org/".to_string());
        }
        "Dancing with Jane Austen" => {
            event
                .links
                .insert(0, "https://www.woosterdance.com/".to_string());
        }
        "English Country Dance Classes in Pasadena" => {
            event.name = "English Country Dance Class".to_string();
            event.links.insert(
                0,
                "https://historicalteaanddance.org/#Tuesdaydances".to_string(),
            );
            event.workshop = true;
            event.social = false;
        }
        "English Country Dance in Pittsburgh" => {
            event.name = "English Country Dance".to_string();
            event.links.insert(0, "https://cdssp.org/".to_string());
        }
        "English Country Dancing" if event.city == "Luray" || event.city == "Penn Laird" => {
            event.links.insert(
                0,
                "https://sites.google.com/corp/site/svcwed/home-1/2025-practice-schedule"
                    .to_string(),
            );
            event.price = Some("free".to_string());
        }
        "Folklore Society of Greater Washington (FSGW) English Country Dance" => {
            event.name = "FSGW English Country Dance".to_string();
        }
        "Four Corners Barn Dance!" => {
            event.name = "Four Corners Barn Dance".to_string();
            event.links.insert(
                0,
                "https://www.fourcornersarts.org/concerts-events/".to_string(),
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
        "Friday Night Contra at Glen Echo, Maryland" => {
            event.name = "Friday Night Contra at Glen Echo".to_string();
            event
                .links
                .insert(0, "https://www.fridaynightdance.com/".to_string());
        }
        "Friends of Traditional Dance Contra" => {
            event.links.insert(0, "https://fotd.org/".to_string());
        }
        "Floyd Contra Dance" => {
            event
                .links
                .insert(0, "https://www.floydcontradance.org/".to_string());
        }
        "Flying Shoes First Friday Community Dance & Contra Dance" => {
            event.name = "Flying Shoes Community Dance & Contra Dance".to_string();
            event.links.insert(
                0,
                "https://belfastflyingshoes.org/first-friday-dances/".to_string(),
            );
        }
        "Gainesville Florida English Country Dance" => {
            event.name = "Gainesville English Country Dance".to_string();
        }
        "Gentle English Country Dance" => {
            event
                .links
                .insert(0, "https://bfms.org/gentleEnglish.php".to_string());
        }
        "Goshen Community Contra Dance" => {
            event.links.insert(0, "http://godancing.org/".to_string());
            if event.price.as_deref() == Some("$3-$18") {
                event.price = Some("$3-$8".to_string());
            }
        }
        "Groton Grange Contra Dance" => {
            event.links.insert(
                0,
                "https://grotongrange.org/events/contra-dance/".to_string(),
            );
        }
        "Hartford Community Dance's 2nd Saturday Contra Dance"
        | "Hartford Community Dance’s 2nd Saturday Contra Dance" => {
            event.name = "Hartfort Community Dance Contra".to_string();
            event
                .links
                .insert(0, "http://www.hcdance.org/contra-dance/".to_string());
        }
        "Hayward Contra Dance" => {
            event
                .links
                .insert(0, "https://sfbaycontra.org/".to_string());
        }
        "Houston Area Traditional Dance Society 1st Sunday English Country Dance" => {
            event.name = "1st Sunday English Country Dance".to_string();
            event
                .links
                .insert(0, "https://hatds.org/ecd#hatds".to_string());
        }
        "Houston Area Traditional Dance Society 3rd Sunday English Country Dance" => {
            event.name = "3rd Sunday English Country Dance".to_string();
            event
                .links
                .insert(0, "https://hatds.org/ecd#hatds".to_string());
        }
        "Houston Area Traditional Dance Society 5th Sunday English Country Dance" => {
            event.name = "5th Sunday English Country Dance".to_string();
            event
                .links
                .insert(0, "https://hatds.org/ecd#hatds".to_string());
        }
        "English Country Dance in Houston" => {
            event.name = "English Country Dance".to_string();
            event
                .links
                .insert(0, "https://hatds.org/ecd#hatds".to_string());
        }
        "Hudson Valley Country Dancers - Port Ewen English Country Dance" => {
            event.name = "Port Ewen English Country Dance".to_string();
            event.links.insert(
                0,
                "https://www.hudsonvalleydance.org/english-country-1".to_string(),
            );
            if event.organisation.as_deref() == Some("cdss") {
                event.organisation = Some("Hudson Valley Country Dancers".to_string());
            }
        }
        "Indy Contra Dance" | "Indy Family Folk Dance" => {
            event
                .links
                .insert(0, "https://www.indycontra.org/".to_string());
        }
        "Indy English Country Dance" => {
            event
                .links
                .insert(0, "https://sites.google.com/view/indyecd/".to_string());
        }
        "Jax Contra Dance" => {
            event.links.insert(0, "https://jaxcontra.org/".to_string());
        }
        "Lake City Contra Dance" => {
            event
                .links
                .insert(0, "https://seattledance.org/contra/lakecity/".to_string());
        }
        "Lancaster Contra Dance" => {
            event
                .links
                .insert(0, "https://lancastercontra.org/".to_string());
        }
        "Las Vegas Contra Dance" => {
            event
                .links
                .insert(0, "https://www.lasvegascontradance.org/".to_string());
        }
        "Lawrence Barn Dance Association Contra"
        | "Lawrence Barn Dance Association Contra Dance"
        | "Community Contra Dance"
            if event.city == "Lawrence" =>
        {
            event.links.insert(
                0,
                "https://lawrencecontra.wordpress.com/calendar/".to_string(),
            );
        }
        "Lenox Contra Dance" => {
            event
                .links
                .insert(0, "https://lenoxcontradance.org/sched.php".to_string());
        }
        "LITMA Smithtown Contradance" => {
            event.links.insert(0, "https://litma.org/".to_string());
        }
        "Louisville Contra Dance" | "Louisville Third-Saturday Contra Dance" => {
            event.links.insert(
                0,
                "https://www.louisvillecountrydancers.org/dance-with-us-1".to_string(),
            );
        }
        "Louisville English Country Dance" => {
            event
                .links
                .insert(0, "https://www.louisvilleecd.org/".to_string());
        }
        "Mendocino English Country Dance" => {
            event
                .links
                .insert(0, "https://www.mendoecd.org/events/".to_string());
        }
        "Missoula Folklore Society Contra Dance" => {
            event.links.insert(
                0,
                "https://www.missoulafolk.org/dance-schedule-vizne".to_string(),
            );
        }
        "Missoula Folklore Society English Country Dance" => {
            event.links.insert(
                0,
                "https://www.missoulafolk.org/english-country-dancing".to_string(),
            );
        }
        "Moab, Utah Contra Dance" => {
            event.name = "Moab Contra Dance".to_string();
        }
        "Monrovia English Country Dance" => {
            event
                .links
                .insert(0, "https://monroviaecd.org/".to_string());
        }
        "Monterey Contra Dance" => {
            event
                .links
                .insert(0, "https://montereycontradance.org/index.html".to_string());
        }
        "Monthly American Folk Dance and Contra Series at Children's Museum of Oak Ridge" => {
            event.name = "Knoxville Contra Dance".to_string();
            event
                .links
                .insert(0, "https://www.knoxvillecontra.org/schedule".to_string());
        }
        "Montpelier Contra Dance" => {
            event.links.insert(
                0,
                "https://capitalcitygrange.org/dancing/contradancing/".to_string(),
            );
        }
        "Monday Cincinnati Contra Dance" => {
            event.price = Some("$2-$5".to_string());
            if event.state.is_none() {
                event.state = Some("OH".to_string());
                event.country = "USA".to_string();
            }
            event.links.insert(
                0,
                "https://www.cincinnaticontradance.org/schedule.htm".to_string(),
            );
        }
        "Monday Contra Dance" if event.city == "Nelson" => {
            event.links.insert(
                0,
                "https://www.monadnockfolk.org/contra-dances/the-nelson-monday-night-dance/"
                    .to_string(),
            );
        }
        "Monday Evening English Country Dance in Baltimore" => {
            event.name = "Monday Evening English Country Dance".to_string();
            event
                .links
                .insert(0, "https://www.bfms.org/mondayDance.php".to_string());
        }
        "Monday Night English Country Dance" if &event.city == "Baltimore" => {
            event
                .links
                .insert(0, "https://www.bfms.org/mondayDance.php".to_string());
        }
        "Monday Night Contra Dance at the Laurel Theater" => {
            event.name = "Contra Dance at the Laurel Theater".to_string();
            event
                .links
                .insert(0, "https://www.knoxvillecontra.org/schedule".to_string());
        }
        "Monday Night Dance" if &event.city == "Knoxville" => {
            event
                .links
                .insert(0, "https://www.knoxvillecontra.org/schedule".to_string());
        }
        "Mystic Contra Dance" | "Mystic Pie Dance" | "Mystic Pie Dance!" => {
            event
                .links
                .insert(0, "https://www.mysticpiedance.org/".to_string());
            if event.price.as_deref() == Some("$3-$10") {
                event.price = Some("$7-$10".to_string());
            }
        }
        "Nashville English Country Dance" => {
            event.links.insert(
                0,
                "https://www.nashvillecountrydancers.org/english-country-dances".to_string(),
            );
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
        "Parish Center for the Arts Barn Dance" => {
            event.links.insert(
                0,
                "https://parisharts.square.site/barn-contra-dance".to_string(),
            );
            event.city = "Westford".to_string();
            event.state = Some("MA".to_string());
        }
        "Phoenix 4th Friday Contra Dance" => {
            event.links.insert(0, "https://phxtmd.org/".to_string());
        }
        "Pittsburgh Contra Dance" => {
            event
                .links
                .insert(0, "https://pittsburghcontra.org/".to_string());
        }
        "Princeton Country Dancers' Combination Contra and English Country Dance" => {
            event
                .links
                .insert(0, "https://www.princetoncountrydancers.org/".to_string());
            event.workshop = true;
        }
        "Princeton Country Dancers’ Contra Dance" => {
            event
                .links
                .insert(0, "https://www.princetoncountrydancers.org/".to_string());
        }
        "Princeton Country Dancers' English Country Dance" => {
            event.name = "Princeton English Country Dance".to_string();
            event.workshop = true;
            event
                .links
                .insert(0, "https://www.princetoncountrydancers.org/".to_string());
        }
        "Quiet Corner Contra Dance" | "Quiet Corner Contra Dance!" => {
            event.name = "Quiet Corner Contra Dance".to_string();
            event
                .links
                .insert(0, "http://www.hcdance.org/quiet-corner-contra/".to_string());
        }
        "Richmond English Country Dance" | "Richmond Wednesday English Country Dance" => {
            event.links.insert(
                0,
                "https://colonialdanceclubofrichmond.com/english-dance-calendar".to_string(),
            );
        }
        "Richmond Saturday Night Contra Dance" => {
            event.name = "Saturday Night Contra Dance".to_string();
        }
        "Roseville CA First Sunday English Country Dance" => {
            event.name = "Roseville English Country Dance".to_string();
            event
                .links
                .insert(0, "https://sactocds.wordpress.com/".to_string());
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
        "Saint Louis Queer Contra" => {
            event
                .links
                .insert(0, "https://www.shedances.org/".to_string());
        }
        "San Antonio Contra Dance" => {
            event
                .links
                .insert(0, "https://www.satxcontra.org/".to_string());
        }
        "San Francisco Contra Dance" => {
            event.links.insert(
                0,
                "https://bacds.org/series/contra/san_francisco_wed/".to_string(),
            );
        }
        "San Luis Obispo Monthly Contra Dance" => {
            event
                .links
                .insert(0, "https://www.cccds.org/schedule/".to_string());
        }
        "Scissortail Contra Dance in Oklahoma City"
        | "Scissortail Contra Dance in Norman, OK"
        | "Scissortail Contra Dance in Tulsa, OK" => {
            event.name = "Scissortail Contra Dance".to_string();
            event
                .links
                .insert(0, "https://scissortail.org/calendar/".to_string());
        }
        "Scissortail English Country Dance in Oklahoma City" => {
            event.name = "Scissortail English Country Dance".to_string();
            event
                .links
                .insert(0, "https://scissortail.org/calendar/".to_string());
        }
        "Sebastopol 1st and 3rd Sunday English Dance"
        | "Sebastopol 5th Sunday Advanced English Dance" => {
            event
                .links
                .insert(0, "https://nbcds.org/english-country-dance/".to_string());
        }
        "Second Saturday Contra" if event.city == "Mesa" => {
            event.links.insert(0, "https://phxtmd.org/".to_string());
        }
        "Second Saturday BFMS Contra Dance"
        | "Second Saturday Baltimore Folk Music Society Contra Dance" => {
            event.name = "Second Saturday BFMS Contra Dance".to_string();
            event
                .links
                .insert(0, "https://www.bfms.org/saturdayDance.php".to_string());
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
        "South Florida Contradance" => {
            event
                .links
                .insert(0, "https://southfloridacontradance.com/".to_string());
        }
        "Space Coast Contra Dance" => {
            event.links.insert(
                0,
                "https://spacecoastcontra.org/calendar-upcoming-contra-dances/".to_string(),
            );
        }
        "Sunday Afternoon Dancing Planet Contra Dance" => {
            event.name = "Dancing Planet Contra Dance".to_string();
            event.links.insert(
                0,
                "https://dancingplanetproductions.com/contra/".to_string(),
            );
        }
        "Swingin' Tern Contra Dance" => {
            event
                .links
                .insert(0, "https://folkproject.org/swingin-tern/".to_string());
        }
        "Syracuse English Country Dance" => {
            event.links.insert(
                0,
                "https://syracusecountrydancers.org/upcoming-dances/".to_string(),
            );
        }
        "Tallahassee Contra Dance" => {
            event
                .links
                .insert(0, "http://www.tallydancer.com/".to_string());
        }
        "TECDA Friday Evening Dance" | "TECDA Tuesday Evening English Country Dance" => {
            event
                .links
                .insert(0, "https://www.tecda.ca/weekly_dances.html".to_string());
        }
        "The Asheville Monday Night Contra Dance" => {
            event.name = "Asheville Monday Night Contra Dance".to_string();
            event.links.insert(
                0,
                "https://themondaynightdance.wixsite.com/home/about".to_string(),
            );
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
        "Tucson Contra Dance" => {
            event
                .links
                .insert(0, "https://tucsoncontradancers.org/".to_string());
            event.price = Some("$5-$10".to_string());
        }
        "Wednesday Night Contra Dance" | "Wednesday Night Contra Dance BFMS"
            if event.city == "Baltimore" =>
        {
            event
                .links
                .insert(0, "https://www.bfms.org/squarecontra.php".to_string());
        }
        "Williamsburg Tuesday Night English Dance" => {
            event
                .links
                .insert(0, "https://williamsburgheritagedancers.org/".to_string());
        }
        "Wooster Contra Dance" => {
            event
                .links
                .insert(0, "https://www.woosterdance.com/contra".to_string());
        }
        "Worcester Contra Dance" | "Worcester Contra Dance!" => {
            event.name = "Worcester Contra Dance".to_string();
            event
                .links
                .insert(0, "https://www.worcesterdance.org/".to_string());
        }
        "Valley Contra Dance" => {
            event
                .links
                .insert(0, "https://valleycontradance.org/".to_string());
        }
        "Valparaiso Old Time Dance Society Contra Dance" => {
            event
                .links
                .insert(0, "https://valpo.chicagobarndance.com/".to_string());
        }
        "Verona, VA Monday Night Contra Dance" => {
            event.name = "Verona Monday Night Contra Dance".to_string();
            event.links.insert(
                0,
                "https://shenandoahvalleycontradance.weebly.com/monday-night-contra.html"
                    .to_string(),
            );
        }
        "Wasatch Contras Third Saturday Monthly Contra Dance" => {
            event.name = "Wasatch Contra Dance".to_string();
            event
                .links
                .insert(0, "https://wasatchcontras.org/".to_string());
        }
        "York Region English Country Dancers" => {
            event.links.insert(0, "http://www.yrecd.ca/".to_string());
        }
        _ => {}
    }

    match (event.city.as_str(), event.state.as_deref()) {
        ("401 Chapman St", Some("Greenfield")) => {
            event.city = "Greenfield".to_string();
            event.state = Some("MA".to_string());
        }
        ("1101 Poyntz Ave.", Some("Manhattan")) => {
            event.city = "Manhattan".to_string();
            event.state = Some("KS".to_string());
        }
        ("9 Genevieve Circle", Some("Asheville")) => {
            event.city = "Asheville".to_string();
            event.state = Some("NC".to_string());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_location() {
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![]),
                ..Default::default()
            })
            .unwrap(),
            Some(("USA".to_string(), None, "".to_string()))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec!["City".to_string()]),
                ..Default::default()
            })
            .unwrap(),
            Some(("USA".to_string(), None, "City".to_string()))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec!["City".to_string(), "CA".to_string()]),
                ..Default::default()
            })
            .unwrap(),
            Some((
                "USA".to_string(),
                Some("CA".to_string()),
                "City".to_string()
            ))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![
                    "123 Some Street".to_string(),
                    "Hayward".to_string(),
                    "CA".to_string(),
                    "94541".to_string(),
                    "USA".to_string(),
                ]),
                ..Default::default()
            })
            .unwrap(),
            Some((
                "USA".to_string(),
                Some("CA".to_string()),
                "Hayward".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![
                    "Pittsburgh".to_string(),
                    "PA".to_string(),
                    "1234".to_string(),
                    "USA".to_string(),
                ]),
                ..Default::default()
            })
            .unwrap(),
            Some((
                "USA".to_string(),
                Some("PA".to_string()),
                "Pittsburgh".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![
                    "Toronto".to_string(),
                    "Ontario".to_string(),
                    "1234".to_string(),
                    "Canada".to_string(),
                ]),
                ..Default::default()
            })
            .unwrap(),
            Some((
                "Canada".to_string(),
                Some("Ontario".to_string()),
                "Toronto".to_string(),
            ))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![
                    "London".to_string(),
                    "N10AB".to_string(),
                    "United Kingdom".to_string()
                ]),
                ..Default::default()
            })
            .unwrap(),
            Some(("UK".to_string(), None, "London".to_string()))
        );
        assert_eq!(
            Cdss::location(&EventParts {
                location_parts: Some(vec![
                    "Venue Name".to_string(),
                    "Address".to_string(),
                    "London".to_string(),
                    "N10AB".to_string(),
                    "United Kingdom".to_string()
                ]),
                ..Default::default()
            })
            .unwrap(),
            Some(("UK".to_string(), None, "London".to_string()))
        );
    }
}
