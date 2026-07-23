//! RocketChat → RuckChat migration binary.

use std::process;

use clap::Parser;

use rocketchat2ruckchat::config::{Cli, resolve};
use rocketchat2ruckchat::error::Error;
use rocketchat2ruckchat::pipeline;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = resolve(&cli)?;
    let report_path = pipeline::run(&config).await?;

    println!("Migration report written to {}", report_path.display());
    Ok(())
}
