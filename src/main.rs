mod cli;
mod git;
mod setup;

// use clap::Parser;
// use cli::{Cli, Commands};
// use git::staged_changes;

use setup::config_file_path;
use setup::parse_config_file;
use setup::setup_config;

fn main() {
    let path = config_file_path();
    setup_config();

    let content = parse_config_file();
    println!("{}", content.provider);
}
