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

pub mod folkbalbende;
pub mod icalendar;
pub mod plugevents;
pub mod trycontra;
pub mod webfeet;

use std::{
    collections::HashMap,
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
};

use crate::{github::to_safe_filename, model::events::Events};
use eyre::Report;
use log::info;

/// Adds any old events older than the oldest new event, and returns the combination.
///
/// This is useful to preserve past events for importers for sources which don't include events in the past.
fn combine_events(old_events: Events, new_events: Events) -> Events {
    let Some(earliest_finish) = new_events
        .events
        .iter()
        .map(|e| e.time.end_time_sort_key())
        .min()
    else {
        // If there are no new events then keep all the old events.
        return old_events;
    };

    let mut events = new_events;
    events.events.extend(
        old_events
            .events
            .into_iter()
            .filter(|event| event.time.end_time_sort_key() < earliest_finish),
    );
    events.sort();
    events.events.dedup();
    events
}

/// Given a set of events, splits them by country then writes one file for each country.
///
/// If the file already exists for that country then applies the logic from [`combine_events`] to
/// preserve old events in it.
pub fn write_by_country(events: Events, filename: &Path) -> Result<(), Report> {
    let mut events_by_country: HashMap<String, Events> = HashMap::new();
    for event in events.events {
        events_by_country
            .entry(event.country.clone())
            .or_default()
            .events
            .push(event);
    }
    for (country, mut country_events) in events_by_country {
        let mut country_filename = PathBuf::new();
        country_filename.push("events");
        country_filename.push(to_safe_filename(&country));
        country_filename.push(filename);
        info!(
            "Writing {} events to {:?}",
            country_events.events.len(),
            country_filename
        );
        if country_filename.exists() {
            // Load without validating, as imports may be invalid.
            let old_events = Events::load_file_without_validation(&country_filename)?;
            country_events = combine_events(old_events, country_events);
        } else {
            create_dir_all(country_filename.parent().unwrap())?;
        }
        write(country_filename, country_events.to_yaml_string()?)?;
    }
    Ok(())
}

const BANDS: [&str; 245] = [
    "112 and Then Some",
    "A Fine Kettle of Fish",
    "Achterband",
    "AdHoc Orkest",
    "Aérokorda",
    "Air de Famille",
    "Airboxes",
    "AlleMonOh Stringband",
    "Andrea Capezzuoli",
    "Androneda",
    "Antanjo",
    "Appalachian Roots",
    "Artisjok",
    "Atlantic Crossing",
    "Aubergine",
    "Aurélien Claranbaux",
    "Back Row Band",
    "Ball Noir",
    "Ballo Allegro",
    "Bamako Express",
    "Bare Necessities",
    "Bart Praet",
    "Beat Bouet Trio",
    "Bellamira",
    "Ben Bolker and Susanne Maziarz",
    "Berkenwerk",
    "Big Fun",
    "Biskaya",
    "Blind Squirrel",
    "BmB",
    "Bougnat Sound",
    "Bourrée Party Crackers",
    "Bouton",
    "Box and String Trio",
    "Brazenkeys",
    "Broes",
    "Brook Farm String Band",
    "Bunny Bread Bandits",
    "Calico",
    "Cardboard Cabin",
    "Carin Greve",
    "Cecilia",
    "Chablis",
    "Chimney Swift",
    "Ciac Boum",
    "Cojiro",
    "Contra Banditos",
    "Contra Intuitive",
    "ContraForce",
    "Contrary Faeries",
    "Contraverts",
    "Danzvogel",
    "David Cornelissen",
    "De Houtzagerij",
    "De Trekvogels",
    "Dead Sea Squirrels",
    "Devilish Mary",
    "DJ TacoShel",
    "Dogtown",
    "Dragon Fire",
    "Dreamy Folk Flow",
    "Drehwurm",
    "Drive Train",
    "Drøn",
    "Duo Absynthe",
    "Duo Baftig",
    "Duo Bottasso",
    "Duo Clercx",
    "Duo Gielen-Buscan",
    "Duo l'Hêtre Heureux",
    "Duo Mackie/Hendrix",
    "Duo Pacher-Roblin",
    "Duo Roblin-Thebaut",
    "Duo Torv",
    "Duo Wolff-Moschcau",
    "Elixir",
    "Eloise & Co.",
    "Emelie Waldken",
    "Emily & The Simons",
    "Engine Room",
    "Erik en Martijn",
    "Exqueezit",
    "Fahrenheit",
    "Feather & Fox",
    "First Time Stringband",
    "Flying Romanos",
    "Folie du Nord",
    "Folkinger",
    "Fourpence",
    "Fyndus",
    "George Paul",
    "Geronimo",
    "Gisbert",
    "Good Intentions",
    "Gott Folk!",
    "Grand Picnic",
    "GrayScale",
    "Guillaume Sparrow-Pepin & Rachel Bell",
    "Hartwin Dhoore Trio",
    "Hartwin Dhoore",
    "Headwaters",
    "Hijinks",
    "Hoggetowne Fancy",
    "Holiday Ball Orchestra",
    "Hot Coffee Breakdown",
    "Hot Griselda",
    "Hot Toddy",
    "I Pizzicati",
    "Javallon",
    "Jeroen Laureyssens",
    "Joachim Montbord",
    "Jormsons Kapell",
    "Joyance",
    "Jumping Sharks",
    "Kelten zonder Grenzen",
    "Kikker & Findus",
    "Kingfisher",
    "Kördeböf",
    "KV Express",
    "L'air Inconnu",
    "La Belle Ivresse",
    "La Réveilleuse",
    "La Sauterelle",
    "Lackawanna Longnecks",
    "Lagomorph Trio",
    "Lake Effect",
    "Laouen",
    "Larks in the Attic",
    "Laurie Fisher & Nik Coker",
    "Les Bottines Artistiques",
    "Les Kickeuses",
    "Les Zéoles",
    "Liberty String Band",
    "Lizzy's Cocktail",
    "Lone Star Pirates",
    "Long Forgotten String Band",
    "Madlot",
    "Magistal",
    "Mara Menzel",
    "Maracu",
    "Marbelous Daves",
    "Marie Paulette",
    "Martina & Gisbert & Rainer",
    "Merriment",
    "Mevilish Merry",
    "Midnight on the Water",
    "Mieneke",
    "Momiro",
    "Mook",
    "Morceau de Breizh",
    "Mr Folxlide",
    "Musac",
    "Nachtmuziek",
    "Naragonia Quartet",
    "Naragonia",
    "Nebel",
    "Noiranomis",
    "Northern Aire",
    "Nova",
    "Nozzy",
    "Nubia",
    "Paracetamol",
    "PFM!",
    "Pimento Mori",
    "Playing with Fyre",
    "Pont Ondulé",
    "Portland Megaband",
    "QuiVive",
    "Ragged Robin",
    "Red Case Band",
    "Red Dog Riley",
    "Reelplay",
    "Rémi Geffroy",
    "River Music",
    "River Road",
    "Rokkende Vrouwen",
    "Round Hill Ramblers",
    "Sail Away Ladies",
    "Serendipity",
    "Simone Bottasso",
    "Sister Haggis",
    "Smith, Campeau & Nelson",
    "Snaarmaarwaar",
    "Snappin' Bug Stringband",
    "Soldo",
    "Sparv",
    "Spintuition",
    "SpringTide",
    "Starling",
    "Stomp Rocket",
    "Stringrays",
    "Supertrad",
    "Swinco",
    "Swingology",
    "Take a Dance",
    "Thalas",
    "The Atchisons",
    "The Black Cat Quadrille",
    "The Boom Chicks",
    "The Campeau Creek Boys",
    "The Dam Beavers",
    "The English Muffins",
    "The Fiddling Thomsons",
    "The Flying Elbows",
    "The Free Raisins",
    "The French Connection",
    "The Gaslight Tinkers",
    "The Ice Cream Truckers",
    "The Little Big Band",
    "The Moving Violations",
    "The Rafter Ringers",
    "The String Bean Serenaders",
    "The Syncopaths",
    "The Turning Stile",
    "Toss the Possum",
    "Tref",
    "Tribal Jaze",
    "Trillium",
    "Trio Baftig",
    "Trio Loubelya",
    "TriOblique",
    "Trip to Norwich",
    "Triple-X",
    "Two Catch a Raindrop",
    "Two Hats",
    "Unbowed",
    "Vandiekomsa",
    "Viola Voilá",
    "Wabi Sabi",
    "Wakarusa Roundabouts",
    "Warleggan Village Band",
    "Waterbound String Band",
    "Wee Merry Banshees",
    "Wergleyberg",
    "Westside Warblers",
    "Wheels of the World",
    "Wild Asparagus",
    "Wild Wombats of the Chesapeake",
    "Wilma",
    "Wim te Groen",
    "Wings & Tales",
    "Woody & the Westside Warblers",
    "Wouter en de Draak",
    "Wouter Kuyper",
    "Yanyk",
];
const CALLERS: [&str; 131] = [
    "Adina Gordon",
    "Alan Rosenthal",
    "Alex Deis-Lauby",
    "Alice Raybourn",
    "Andrew Swaine",
    "Ann Fallon",
    "Annie Kidwell",
    "Barbara Finney",
    "Barrett Grimm",
    "Ben Sachs-Hamilton",
    "Ben Sela",
    "Bev Birnbaum",
    "Billy Fischer",
    "Bob Fabinski",
    "Bob Frederking",
    "Bob Green",
    "Bob Isaacs",
    "Brad Foster",
    "Brian Hamshar",
    "Bridget Whitehead",
    "Bronwyn Chelette",
    "Brooke Friendly",
    "Bruce Hamilton",
    "Carol Kopp",
    "Caroline Barnes",
    "Cathy Campbell",
    "Charley Harvey",
    "Chrissy Fowler",
    "Christine Merryman",
    "Cindy Harris",
    "Cis Hinkle",
    "Claire Takemori",
    "Dan Blim",
    "Darlene Underwood",
    "Dave Berman",
    "Dave Smukler",
    "David Eisenstadter",
    "David Macemon",
    "Deanna Palumbo",
    "Dereck Kalish",
    "Devin Pohly",
    "Diane Silver",
    "Don Heinold",
    "Don Veino",
    "Dorothy Cummings",
    "Earl McGill",
    "Emily Addison",
    "Emily Rush",
    "Erik Hoffman",
    "Gaye Fifer",
    "Gene Murrow",
    "George Marshall",
    "George Thompson",
    "Greg Frock",
    "Gretchen Caldwell",
    "Harris Lapiroff",
    "Janet Shepherd",
    "Janine Smith",
    "Jen Jasenski",
    "Jill Allen",
    "Joanna Reiner Wilkinson",
    "Joe Harrington",
    "John Krumm",
    "John Notgrass",
    "Jordan Kammeyer",
    "Judi Rivkin",
    "Kalia Kliban",
    "Kappy Laning",
    "Karen Andrews",
    "Katie Zanders",
    "Katy Heine",
    "Kelsey Hartman",
    "Ken Gall",
    "Laura Beraha",
    "Laura Hudlow",
    "Laurel Thomas",
    "Lindsey Dono",
    "Lisa Greenleaf",
    "Lisa Harris-Frydman",
    "Lisa Heywood",
    "Lisa Newcomb",
    "Liz Nelson",
    "Louise Siddons",
    "Luke Donforth",
    "Mae Wilson",
    "Maeve Devlin",
    "Marc Airhart",
    "Marlin Whitaker",
    "Martha Kent",
    "Mary Wesley",
    "Melissa Chatham",
    "Michael Karchar",
    "Myra Hirschberg",
    "Nils Fredland",
    "Olivia Barry",
    "Orly Krasner",
    "Paul Ross",
    "Paul Wilde",
    "Penelope Weinberger",
    "Rebecca Anger",
    "Rhodri Davies",
    "Rich Goss",
    "Rich MacMath",
    "Rich Sbardella",
    "Rick Szumski",
    "River Abel",
    "River Rainbowface",
    "Ron Buchanan",
    "Seth Tepfer",
    "Steph West",
    "Steve Gester",
    "Steve Zakon-Anderson",
    "Sue Gola",
    "Susan English",
    "Susan Kevra",
    "Susan Michaels",
    "Susie Kendig",
    "Tara Bolker",
    "Ted Hodapp",
    "Terry Doyle",
    "Timothy Klein",
    "Tod Whittemore",
    "Tom Callwell",
    "Tom Greene",
    "Val Medve",
    "Vicki Morrison",
    "Walter Zagorski",
    "Wendy Graham",
    "Will Mentor",
    "William Watson",
    "Zach Kaplan",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        dancestyle::DanceStyle,
        event::{Event, EventTime},
    };
    use chrono::NaiveDate;

    fn make_event(name: &str, time: EventTime) -> Event {
        Event {
            name: name.to_string(),
            time,
            details: None,
            links: vec![],
            country: "Test".to_string(),
            state: None,
            city: "Test".to_string(),
            styles: vec![DanceStyle::EnglishCountryDance],
            workshop: true,
            social: false,
            bands: vec![],
            callers: vec![],
            price: None,
            organisation: None,
            cancelled: false,
            source: None,
        }
    }

    #[test]
    fn combine_no_old() {
        let old_events = Events::default();
        let new_events = Events {
            events: vec![
                make_event(
                    "New 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "New 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };

        let combined = combine_events(old_events, new_events.clone());
        assert_eq!(combined, new_events);
    }

    #[test]
    fn combine_no_new() {
        let old_events = Events {
            events: vec![
                make_event(
                    "Old 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "Old 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };
        let new_events = Events::default();

        let combined = combine_events(old_events.clone(), new_events);
        assert_eq!(combined, old_events);
    }

    #[test]
    fn combine_same() {
        let events = Events {
            events: vec![
                make_event(
                    "Old 1",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                    },
                ),
                make_event(
                    "Old 2",
                    EventTime::DateOnly {
                        start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                        end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                    },
                ),
            ],
        };

        let combined = combine_events(events.clone(), events.clone());
        assert_eq!(combined, events);
    }

    #[test]
    fn combine_overlap() {
        let old1 = make_event(
            "Old 1",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            },
        );
        let old3 = make_event(
            "Old 3",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 3).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 3).unwrap(),
            },
        );
        let old_events = Events {
            events: vec![old1.clone(), old3.clone()],
        };
        let new2 = make_event(
            "New 2",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 2).unwrap(),
            },
        );
        let new4 = make_event(
            "New 4",
            EventTime::DateOnly {
                start_date: NaiveDate::from_ymd_opt(1000, 1, 4).unwrap(),
                end_date: NaiveDate::from_ymd_opt(1000, 1, 4).unwrap(),
            },
        );
        let new_events = Events {
            events: vec![new2.clone(), new4.clone()],
        };

        let combined = combine_events(old_events, new_events);
        assert_eq!(combined.events, vec![old1, new2, new4]);
    }
}
