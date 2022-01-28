mod model;

use crate::model::{
    event::{DanceStyle, Event},
    events::Events,
};
use eyre::Report;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let events = Events {
        events: vec![
            Event {
                name: "London Barndance with Mark Elvins and English Contra Dance Band".to_string(),
                details: Some(
                    "Contra dance with Mark Elvins calling and the English Contra Dance Band playing.".to_string(),
                ),
                links: vec![
                    "https://www.barndance.org/programme.html".to_string(),
                    "https://www.facebook.com/events/243417821143957/".to_string(),
                ],
                country: "UK".to_string(),
                city: "London".to_string(),
                styles: vec![DanceStyle::Contra],
                workshop: false,
                social: true,
                bands: vec!["English Contra Dance Band".to_string()],
                callers: vec!["Mark Elvins".to_string()],
                price: Some("£5-£14".to_string()),
                organisation: Some("London Barndance Company".to_string()),
            },
            Event {
                name: "Minimal".to_string(),
                details: None,
                links: vec![],
                country: "UK".to_string(),
                city: "London".to_string(),
                styles: vec![],
                workshop: false,
                social: false,
                bands: vec![],
                callers: vec![],
                price: None,
                organisation: None,
            },
        ],
    };

    for event in &events.events {
        println!("{:?}", event);
    }

    println!("{}", toml::to_string(&events)?);

    Ok(())
}
