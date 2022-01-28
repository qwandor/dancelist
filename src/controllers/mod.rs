use crate::{
    errors::InternalError,
    model::{event::Event, events::Events},
};
use askama::Template;
use axum::{extract::Extension, response::Html};
use chrono::{naive, Datelike, NaiveDate};

pub async fn index(Extension(events): Extension<Events>) -> Result<Html<String>, InternalError> {
    let months = sort_and_group_by_month(events.events);
    let template = IndexTemplate { months };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
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
fn sort_and_group_by_month(mut events: Vec<Event>) -> Vec<Month> {
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
            month.events.push(event);
        } else {
            if !month.events.is_empty() {
                months.push(month);
            }
            month = Month {
                start: NaiveDate::from_ymd(event.start_date.year(), event.start_date.month(), 1),
                events: vec![event],
            };
        }
    }
    if !month.events.is_empty() {
        months.push(month);
    }

    months
}
