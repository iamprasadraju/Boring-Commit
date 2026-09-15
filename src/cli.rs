use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bcommit",
    visible_alias = "boringcommit",
    version,
    about = "BoringCommit is a tool that uses LLMs to generate Git commit messages from staged changes. Written in Rust."
)]

pub struct Cli{
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Setup,
}


