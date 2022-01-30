use crate::{
    errors::InternalError,
    model::{
        event::{DanceStyle, Event},
        events::Events,
    },
};
use askama::Template;
use axum::{
    extract::{Extension, Query},
    response::Html,
};
use chrono::{naive, Datelike, NaiveDate};
use serde::Deserialize;

pub async fn index(
    Extension(events): Extension<Events>,
    Query(filters): Query<Filters>,
) -> Result<Html<String>, InternalError> {
    let events = events
        .future()
        .into_iter()
        .filter(|event| filters.matches(event))
        .collect();
    let months = sort_and_group_by_month(events);
    let template = IndexTemplate { filters, months };
    Ok(Html(template.render()?))
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct Filters {
    country: Option<String>,
    city: Option<String>,
    style: Option<DanceStyle>,
    multiday: Option<bool>,
    workshop: Option<bool>,
    social: Option<bool>,
}

impl Filters {
    fn has_some(&self) -> bool {
        self.country.is_some()
            || self.city.is_some()
            || self.style.is_some()
            || self.multiday.is_some()
            || self.workshop.is_some()
            || self.social.is_some()
    }

    fn matches(&self, event: &Event) -> bool {
        if let Some(country) = &self.country {
            if &event.country != country {
                return false;
            }
        }
        if let Some(city) = &self.city {
            if &event.city != city {
                return false;
            }
        }
        if let Some(style) = &self.style {
            if !event.styles.contains(style) {
                return false;
            }
        }
        if let Some(multiday) = self.multiday {
            if event.multiday() != multiday {
                return false;
            }
        }
        if let Some(workshop) = self.workshop {
            if event.workshop != workshop {
                return false;
            }
        }
        if let Some(social) = self.social {
            if event.social != social {
                return false;
            }
        }

        true
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    filters: Filters,
    months: Vec<Month>,
}

struct Month {
    /// The first day of the month.
    start: NaiveDate,
    events: Vec<Event>,
}

impl Month {
    pub fn name(&self) -> String {
        self.start.format("%B %Y").to_string()
    }
}

/// Given a list of events in arbitrary order, sort them in ascending order of start date, then group them by starting month.
fn sort_and_group_by_month(mut events: Vec<&Event>) -> Vec<Month> {
    events.sort_by_key(|event| event.start_date);

    let mut months = vec![];
    let mut month = Month {
        start: naive::MIN_DATE,
        events: vec![],
    };
    for event in events {
        if event.start_date.year() == month.start.year()
            && event.start_date.month() == month.start.month()
        {
            month.events.push(event.to_owned());
        } else {
            if !month.events.is_empty() {
                months.push(month);
            }
            month = Month {
                start: NaiveDate::from_ymd(event.start_date.year(), event.start_date.month(), 1),
                events: vec![event.to_owned()],
            };
        }
    }
    if !month.events.is_empty() {
        months.push(month);
    }

    months
}
