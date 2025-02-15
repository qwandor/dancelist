// Copyright 2024 the dancelist authors.
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

pub const CALLERS: [&str; 234] = [
    "Adam Hughes",
    "Adina Gordon",
    "Alan Rosenthal",
    "Alan Winston",
    "Alex Deis-Lauby",
    "Alice Raybourn",
    "Alice Smith-Goeke",
    "Amy Letson",
    "Andrea Nettleton",
    "Andrew Swaine",
    "Angela DeCarlis",
    "Ann Carter",
    "Ann Fallon",
    "Anna Rain",
    "Annie Fain Barralon",
    "Annie Kidwell",
    "April Blum",
    "Barbara Finney",
    "Barrett Grimm",
    "Beau Farmer",
    "Ben Allbrandt",
    "Ben Sachs-Hamilton",
    "Ben Sela",
    "Beth Molaro",
    "Bev Birnbaum",
    "Beverly Francis",
    "Billy Fischer",
    "Bob Devaty",
    "Bob Fabinski",
    "Bob Frederking",
    "Bob Green",
    "Bob Isaacs",
    "Bob Morgan",
    "Brad Foster",
    "Brian Hamshar",
    "Bridget Whitehead",
    "Bronwyn Chelette",
    "Brooke Friendly",
    "Bruce Hamilton",
    "Cara King",
    "Carl Friedman",
    "Carl Levine",
    "Carol Kopp",
    "Carol Ormand",
    "Caroline Barnes",
    "Carrie Dayton-Madsen",
    "Cathy Campbell",
    "Cathy Hollister",
    "Charley Harvey",
    "Charlie Turner",
    "Charlotte Crittenden",
    "Charlotte Rich-Griffin",
    "Charmaine Slaven",
    "Chet Gray",
    "Chris Hernandez",
    "Chrissy Fowler",
    "Christine Merryman",
    "Cindy Harris",
    "Cis Hinkle",
    "Claire Takemori",
    "Colette Mrozek",
    "Courtney Cartwright",
    "Daisy Black",
    "Dan Blim",
    "Dan Kappus",
    "Dan Seppeler",
    "Darlene Hamilton",
    "Darlene Underwood",
    "Dave Bateman",
    "Dave Berman",
    "Dave Rupp",
    "Dave Smukler",
    "David Eisenstadter",
    "David Kirchner",
    "David Macemon",
    "David Millstone",
    "David Newitt",
    "David Smukler",
    "Deanna Palumbo",
    "Dereck Kalish",
    "Devin Pohly",
    "Diane Silver",
    "Dilip Sequeira",
    "Don Heinold",
    "Don Veino",
    "Donna Hunt",
    "Dorothy Cummings",
    "Drew Delaware",
    "Earl McGill",
    "Elizabeth Estep",
    "Emily Addison",
    "Emily Rush",
    "Erik Hoffman",
    "Gaye Fifer",
    "Gene Murrow",
    "George Marshall",
    "George Thompson",
    "Graham Christian",
    "Greg Frock",
    "Gretchen Caldwell",
    "Harris Laperoff",
    "Harris Lapiroff",
    "Jack Kanutin",
    "Jacqui Grennan",
    "Jake Wood",
    "James Hutson",
    "Janet Shepherd",
    "Janine Smith",
    "Jen Jasenski",
    "Jen Morgan",
    "Jenna Simpson",
    "Jennifer Staples",
    "Jenny Fraser",
    "Jeremy Child",
    "Jeremy Korr",
    "Jesse Partridge",
    "Jill Allen",
    "Jill Delaney",
    "Joanna Reiner Wilkinson",
    "Joe Harrington",
    "John Krumm",
    "John Notgrass",
    "Jordan Kammeyer",
    "Joseph Pimentel",
    "Judi Rivkin",
    "Julian Blechner",
    "Juliette Webb",
    "Kalia Kliban",
    "Kappy Laning",
    "Karen Andrews",
    "Karen Jackson",
    "Karen Justin",
    "Katie Zanders",
    "Katy Heine",
    "Kelsey Hartman",
    "Ken Gall",
    "Kenny Greer",
    "Kim Forry",
    "Koren Wake",
    "Kris Rosar",
    "Laura Beraha",
    "Laura Hudlow",
    "Laurel Thomas",
    "Lauren Catlin",
    "Lauren Wilson",
    "Lindsay Verbil",
    "Lindsey Dono",
    "Lisa Greenleaf",
    "Lisa Harris-Frydman",
    "Lisa Heywood",
    "Lisa Newcomb",
    "Lise Dyckman",
    "Liz Burkhart",
    "Liz Nelson",
    "Louise Siddons",
    "Luke Donforth",
    "Lydia Molineaux",
    "Mae Wilson",
    "Maeve Devlin",
    "Malcolm Jowett",
    "Marc Airhart",
    "Margaret Goodman",
    "Mark Elvins",
    "Marlin Whitaker",
    "Martha Kent",
    "Mary Wesley",
    "Melissa Chatham",
    "Michael Karchar",
    "Michael Karcher",
    "Myra Hirschberg",
    "Nicola Scott",
    "Nils Fredland",
    "Noah Grunzweig",
    "Olivia Barry",
    "Orly Krasner",
    "Paul Ross",
    "Paul Wilde",
    "Penelope Weinberger",
    "Peter Stix",
    "Peter Wollenberg",
    "Qwill Duvall",
    "Rachel Ameen",
    "Rachel Pusey",
    "Rebecca Anger",
    "Rhi Davies",
    "Rhodri Davies",
    "Rich Goss",
    "Rich MacMath",
    "Rich Sbardella",
    "Rick Mohr",
    "Rick Szumski",
    "River Abel",
    "River Rainbowface",
    "Rob Humphrey",
    "Ron Buchanan",
    "Sally Vernon",
    "Sam Smith",
    "Sam Tetley Smith",
    "Sandy Lafleur",
    "Sarah Trop",
    "Scott Higgs",
    "Seth Tepfer",
    "Steph West",
    "Stephanie Marie",
    "Steve Gester",
    "Steve Holland",
    "Steve Otlowski",
    "Steve Pike",
    "Steve White",
    "Steve Zakon-Anderson",
    "Sue Gola",
    "Sue Rosen",
    "Susan English",
    "Susan Kevra",
    "Susan Michaels",
    "Susan Petrick",
    "Susan Taylor",
    "Susie Kendig",
    "Suzanne Farmer",
    "Tamara Loewenthal",
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
