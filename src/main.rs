mod model;

use crate::model::events::Events;
use eyre::Report;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let events = Events::load()?;

    for event in &events.events {
        println!("{:?}", event);
    }

    println!("{}", toml::to_string(&events)?);

    Ok(())
}
