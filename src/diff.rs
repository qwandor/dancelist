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

use crate::model::event::Event;

/// Prints out a diff between the two sets of events.
pub fn print_diff(mut events_a: Vec<Event>, mut events_b: Vec<Event>) {
    // Sort both by date then location, for a consistent comparison.
    events_a.sort_by_key(Event::date_location_sort_key);
    events_b.sort_by_key(Event::date_location_sort_key);

    let mut same = 0;
    let mut a = 0;
    let mut b = 0;
    while a < events_a.len() || b < events_b.len() {
        let event_a = events_a.get(a);
        let event_b = events_b.get(b);
        if event_a == event_b {
            a += 1;
            b += 1;
            same += 1;
        } else if event_a.is_none() {
            println!("Added: {:?}", event_b);
            b += 1;
        } else if event_b.is_none() {
            println!("Removed: {:?}", event_a);
            a += 1;
        } else if event_a.unwrap().date_location_sort_key()
            < event_b.unwrap().date_location_sort_key()
        {
            println!("Removed: {:?}", event_a);
            a += 1;
        } else {
            println!("Added: {:?}", event_b);
            b += 1;
        }
    }
    println!("{} events the same.", same);
}
