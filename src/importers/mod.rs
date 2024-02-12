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

const BANDS: [&str; 129] = [
    "Achterband",
    "AdHoc Orkest",
    "Aérokorda",
    "Airboxes",
    "AlleMonOh Stringband",
    "Androneda",
    "Artisjok",
    "Aubergine",
    "Aurélien Claranbaux",
    "Ball Noir",
    "Bamako Express",
    "Bare Necessities",
    "Bart Praet",
    "Beat Bouet Trio",
    "Ben Bolker and Susanne Maziarz",
    "Berkenwerk",
    "Big Fun",
    "BmB",
    "Bouton",
    "Brook Farm String Band",
    "Bunny Bread Bandits",
    "Calico",
    "Carin Greve",
    "Cecilia",
    "Chimney Swift",
    "Cojiro",
    "Contraverts",
    "De Houtzagerij",
    "De Trekvogels",
    "Dead Sea Squirrels",
    "Devilish Mary",
    "Dogtown",
    "Duo Absynthe",
    "Duo Baftig",
    "Duo Bottasso",
    "Duo Clercx",
    "Duo Gielen-Buscan",
    "Duo Mackie/Hendrix",
    "Duo Roblin-Thebaut",
    "Duo Torv",
    "Duo l'Hêtre Heureux",
    "Elixir",
    "Eloise & Co.",
    "Emelie Waldken",
    "Emily & The Simons",
    "Exqueezit",
    "Fahrenheit",
    "First Time Stringband",
    "Folie du Nord",
    "Fyndus",
    "Geronimo",
    "Good Intentions",
    "Gott Folk!",
    "GrayScale",
    "Hartwin Dhoore Trio",
    "Hartwin Dhoore",
    "Headwaters",
    "Joyance",
    "Kelten zonder Grenzen",
    "Kikker & Findus",
    "Kingfisher",
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
    "Lone Star Pirates",
    "Long Forgotten String Band",
    "Madlot",
    "Mevilish Merry",
    "Mieneke",
    "Momiro",
    "Mook",
    "Musac",
    "Naragonia",
    "Nebel",
    "Noiranomis",
    "Nova",
    "Nubia",
    "Paracetamol",
    "PFM!",
    "Playing with Fyre",
    "Pont Ondulé",
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
    "The Dam Beavers",
    "The Fiddling Thomsons",
    "The Flying Elbows",
    "The Free Raisins",
    "The French Connection",
    "The Gaslight Tinkers",
    "The Turning Stile",
    "Tref",
    "Trio Loubelya",
    "Triple-X",
    "Two Hats",
    "Unbowed",
    "Warleggan Village Band",
    "Wee Merry Banshees",
    "Wheels of the World",
    "Wild Asparagus",
    "Wilma",
    "Wim te Groen",
    "Wouter en de Draak",
    "Wouter Kuyper",
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
