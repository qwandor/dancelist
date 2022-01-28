use serde::{Deserialize, Serialize};

/// The prefix which Facebook event URLs start with.
const FACEBOOK_EVENT_PREFIX: &str = "https://www.facebook.com/events/";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event {
    /// The name of the event.
    pub name: String,
    /// More details describing the event.
    pub details: Option<String>,
    /// URLs with more information about the event, including the Facebook event page if any.
    pub links: Vec<String>,
    // TODO: Should start and end require time or just date? What about timezone?
    pub country: String,
    pub city: String,
    // TODO: What about full address?
    /// The dance styles included in the event.
    pub styles: Vec<DanceStyle>,
    /// The event includes one or more workshops or lessons.
    pub workshop: bool,
    /// The event includes one or more social dances.
    pub social: bool,
    /// The names of the bands playing at the event.
    pub bands: Vec<String>,
    /// The names of the callers calling at the event, if applicable.
    pub callers: Vec<String>,
    /// The price or price range of the event, if available.
    pub price: Option<String>,
    // TODO: Should free events be distinguished from events with unknown price?
    /// The organisation who run the event.
    pub organisation: Option<String>,
}

impl Event {
    /// Check that the event information is valid. Returns an empty list if it is, or a list of
    /// problems if not.
    pub fn validate(&self) -> Vec<&'static str> {
        let mut problems = vec![];

        if !self.workshop && !self.social {
            problems.push("Must have at least a workshop or a social.")
        }

        problems
    }

    /// Get the URL of the event's Facebook event, if any.
    pub fn facebook_event(&self) -> Option<&String> {
        self.links
            .iter()
            .find(|link| link.starts_with(FACEBOOK_EVENT_PREFIX))
    }

    /// Get the event's first link.
    pub fn main_link(&self) -> Option<&String> {
        self.links.first()
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DanceStyle {
    Balfolk,
    Contra,
    EnglishCeilidh,
    Playford,
    Reeling,
    ScottishCeilidh,
    ScottishCountryDance,
    Scandinavian,
}
