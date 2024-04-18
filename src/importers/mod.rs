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
pub mod trycontra;
pub mod webfeet;

use crate::model::events::Events;

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

const BANDS: [&str; 194] = [
    "112 and Then Some",
    "A Fine Kettle of Fish",
    "Achterband",
    "AdHoc Orkest",
    "Aérokorda",
    "Airboxes",
    "AlleMonOh Stringband",
    "Androneda",
    "Antanjo",
    "Artisjok",
    "Atlantic Crossing",
    "Aubergine",
    "Aurélien Claranbaux",
    "Back Row Band",
    "Ball Noir",
    "Bamako Express",
    "Bare Necessities",
    "Bart Praet",
    "Beat Bouet Trio",
    "Bellamira",
    "Ben Bolker and Susanne Maziarz",
    "Berkenwerk",
    "Big Fun",
    "BmB",
    "Bougnat Sound",
    "Bourrée Party Crackers",
    "Bouton",
    "Broes",
    "Brook Farm String Band",
    "Bunny Bread Bandits",
    "Calico",
    "Carin Greve",
    "Cardboard Cabin",
    "Cecilia",
    "Chablis",
    "Chimney Swift",
    "Ciac Boum",
    "Cojiro",
    "Contra Banditos",
    "Contrary Faeries",
    "Contraverts",
    "De Houtzagerij",
    "De Trekvogels",
    "Dead Sea Squirrels",
    "Devilish Mary",
    "Dogtown",
    "Drehwurm",
    "Drive Train",
    "Drøn",
    "Duo Absynthe",
    "Duo Baftig",
    "Duo Bottasso",
    "Duo Clercx",
    "Duo Gielen-Buscan",
    "Duo Mackie/Hendrix",
    "Duo Pacher-Roblin",
    "Duo Roblin-Thebaut",
    "Duo Torv",
    "Duo l'Hêtre Heureux",
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
    "Good Intentions",
    "Gott Folk!",
    "GrayScale",
    "Hartwin Dhoore Trio",
    "Hartwin Dhoore",
    "Headwaters",
    "Hijinks",
    "Hoggetowne Fancy",
    "Holiday Ball Orchestra",
    "Hot Griselda",
    "Hot Toddy",
    "I Pizzicati",
    "Javallon",
    "Jormsons Kapell",
    "Joyance",
    "Kelten zonder Grenzen",
    "Kikker & Findus",
    "Kingfisher",
    "Kördeböf",
    "KV Express",
    "L'air Inconnu",
    "La Réveilleuse",
    "La Sauterelle",
    "Lackawanna Longnecks",
    "Lake Effect",
    "Laouen",
    "Larks in the Attic",
    "Les Bottines Artistiques",
    "Les Kickeuses",
    "Les Zéoles",
    "Liberty String Band",
    "Lizzy's Cocktail",
    "Lone Star Pirates",
    "Long Forgotten String Band",
    "Madlot",
    "Mara Menzel",
    "Maracu",
    "Marbelous Daves",
    "Martina & Gisbert & Rainer",
    "Mevilish Merry",
    "Mieneke",
    "Momiro",
    "Mook",
    "Morceau de Breizh",
    "Musac",
    "Nachtmuziek",
    "Naragonia",
    "Naragonia Quartet",
    "Nebel",
    "Noiranomis",
    "Northern Aire",
    "Nova",
    "Nubia",
    "Paracetamol",
    "PFM!",
    "Pimento Mori",
    "Playing with Fyre",
    "Pont Ondulé",
    "Portland Megaband",
    "QuiVive",
    "Red Case Band",
    "Rémi Geffroy",
    "River Music",
    "River Road",
    "Rokkende Vrouwen",
    "Serendipity",
    "Simone Bottasso",
    "Smith, Campeau & Nelson",
    "Snappin' Bug Stringband",
    "Sparv",
    "Spintuition",
    "SpringTide",
    "Starling",
    "Stomp Rocket",
    "Supertrad",
    "Swinco",
    "Swingology",
    "Take a Dance",
    "The Atchisons",
    "The Boom Chicks",
    "The Black Cat Quadrille",
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
    "The String Bean Serenaders",
    "The Syncopaths",
    "The Turning Stile",
    "Tref",
    "Trillium",
    "TriOblique",
    "Trio Baftig",
    "Trio Loubelya",
    "Trip to Norwich",
    "Triple-X",
    "Two Hats",
    "Unbowed",
    "Vandiekomsa",
    "Wabi Sabi",
    "Warleggan Village Band",
    "Wee Merry Banshees",
    "Wergleyberg",
    "Westside Warblers",
    "Wheels of the World",
    "Wild Asparagus",
    "Wilma",
    "Wim te Groen",
    "Wings & Tales",
    "Woody & the Westside Warblers",
    "Wouter en de Draak",
    "Wouter Kuyper",
];
const CALLERS: [&str; 93] = [
    "Adina Gordon",
    "Alan Rosenthal",
    "Alex Deis-Lauby",
    "Alice Raybourn",
    "Andrew Swaine",
    "Barbara Finney",
    "Barrett Grimm",
    "Ben Sachs-Hamilton",
    "Bev Birnbaum",
    "Bob Frederking",
    "Billy Fischer",
    "Bob Isaacs",
    "Brad Foster",
    "Bridget Whitehead",
    "Bronwyn Chelette",
    "Brooke Friendly",
    "Cathy Campbell",
    "Christine Merryman",
    "Cindy Harris",
    "Claire Takemori",
    "Dan Blim",
    "Darlene Underwood",
    "Dave Berman",
    "Dave Smukler",
    "David Macemon",
    "Dereck Kalish",
    "Diane Silver",
    "Don Heinold",
    "Don Veino",
    "Dorothy Cummings",
    "Earl McGill",
    "Emily Rush",
    "Erik Hoffman",
    "Gaye Fifer",
    "George Marshall",
    "George Thompson",
    "Greg Frock",
    "Janine Smith",
    "Jen Jasenski",
    "Jill Allen",
    "Joanna Reiner Wilkinson",
    "John Krumm",
    "Jordan Kammeyer",
    "Judi Rivkin",
    "Kalia Kliban",
    "Katie Zanders",
    "Katy Heine",
    "Kelsey Hartman",
    "Ken Gall",
    "Laura Beraha",
    "Laurel Thomas",
    "Lindsey Dono",
    "Lisa Greenleaf",
    "Lisa Harris-Frydman",
    "Lisa Heywood",
    "Liz Nelson",
    "Luke Donforth",
    "Mae Wilson",
    "Maeve Devlin",
    "Marc Airhart",
    "Marlin Whitaker",
    "Martha Kent",
    "Mary Wesley",
    "Michael Karchar",
    "Nils Fredland",
    "Orly Krasner",
    "Paul Ross",
    "Paul Wilde",
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
    "Steve Zakon-Anderson",
    "Susan English",
    "Susan Kevra",
    "Susan Michaels",
    "Susie Kendig",
    "Tara Bolker",
    "Ted Hodapp",
    "Timothy Klein",
    "Tod Whittemore",
    "Tom Greene",
    "Val Medve",
    "Vicki Morrison",
    "Walter Zagorski",
    "Will Mentor",
    "William Watson",
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
