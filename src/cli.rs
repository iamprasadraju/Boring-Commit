use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lcommit",
    version,
    about = "Generate Git commit messages from staged changes with a local LLM, written in Rust"
)]


pub struct Cli{
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Setup,
}


