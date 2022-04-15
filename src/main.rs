use eyre::Report;
use std::{env, process::exit};

use dancelist::*;

#[tokio::main]
async fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        serve().await
    } else if args.len() == 2 && args[1] == "schema" {
        // Output JSON schema for events.
        print!("{}", event_schema()?);
        Ok(())
    } else if args.len() >= 2 && args.len() <= 3 && args[1] == "validate" {
        validate(args.get(2).map(String::as_str)).await
    } else if args.len() >= 2 && args.len() <= 3 && args[1] == "cat" {
        concatenate(args.get(2).map(String::as_str)).await
    } else if args.len() == 2 && args[1] == "balbende" {
        import_balbende().await
    } else if args.len() == 2 && args[1] == "webfeet" {
        import_webfeet().await
    } else if args.len() == 2 && args[1] == "balfolknl" {
        import_balfolknl().await
    } else {
        eprintln!("Invalid command.");
        exit(1);
    }
}
