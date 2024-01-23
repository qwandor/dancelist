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
pub fn print_diff(events_a: Vec<Event>, events_b: Vec<Event>) {
    let diff = find_diff(events_a, events_b);

    for (event, added) in &diff.different {
        if *added {
            println!("Added: {:?}", event);
        } else {
            println!("Removed: {:?}", event);
        }
    }
    println!("{} events the same.", diff.same);
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiffResult {
    /// The boolean is false if the event is only in the first list, true if it is only in the
    /// second.
    different: Vec<(Event, bool)>,
    /// The number of events exactly the same in both lists.
    same: usize,
}

fn find_diff(mut events_a: Vec<Event>, mut events_b: Vec<Event>) -> DiffResult {
    // Sort both by date then location, for a consistent comparison.
    events_a.sort_by_key(Event::date_location_sort_key);
    events_b.sort_by_key(Event::date_location_sort_key);

    let mut different = Vec::new();
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
            different.push((event_b.unwrap().to_owned(), true));
            b += 1;
        } else if event_b.is_none() {
            different.push((event_a.unwrap().to_owned(), false));
            a += 1;
        } else if event_a.unwrap().date_location_sort_key()
            < event_b.unwrap().date_location_sort_key()
        {
            different.push((event_a.unwrap().to_owned(), false));
            a += 1;
        } else {
            different.push((event_b.unwrap().to_owned(), true));
            b += 1;
        }
    }

    DiffResult { different, same }
}
