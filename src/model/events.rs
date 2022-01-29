use super::event::Event;
use chrono::Utc;
use eyre::{bail, Report, WrapErr};
use log::trace;
use serde::{Deserialize, Serialize};
use std::{
    fs::{read_dir, read_to_string},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Events {
    pub events: Vec<Event>,
}

impl Events {
    pub fn load(directory: &Path) -> Result<Self, Report> {
        let mut events = vec![];
        for entry in read_dir(directory)? {
            let filename = entry?.path();
            trace!("Reading events from {:?}", filename);
            let contents =
                read_to_string(&filename).wrap_err_with(|| format!("Reading {:?}", filename))?;
            let file_events = toml::from_str::<Events>(&contents)?.events;
            for event in &file_events {
                let problems = event.validate();
                if !problems.is_empty() {
                    bail!(
                        "Problems with event '{}' in {:?}: {:?}",
                        event.name,
                        filename,
                        problems
                    );
                }
            }
            events.extend(file_events);
        }
        Ok(Self { events })
    }

    /// Get all events finishing on or after the present day.
    pub fn future(&self) -> Vec<&Event> {
        let today = Utc::now().naive_utc().date();
        self.events
            .iter()
            .filter(|event| event.end_date >= today)
            .collect()
    }
}
