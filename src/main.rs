mod git;
mod cli;
mod setup;


use clap::Parser;
use git::staged_changes;
use cli::{Cli, Commands};
use setup::setup;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => setup(),
        None => {
            println!("Use `lcommit setup`");
        }
    }

    let output = staged_changes();
    println!("{}", output);
}