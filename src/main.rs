mod ui;
mod cli;
mod git;
mod llm;
mod setup;

use clap::{Parser, Subcommand};
use setup::{parse_config_file, setup_config, setup_instructions};
use cli::{
    choose_model, config_provider, edit_instructions, instructions_path, remove_model,
    reset_instructions, show_instructions,
};
use inquire::{set_global_render_config, ui::{Attributes, Color, RenderConfig, StyleSheet}};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use std::io::Write;

const BRAND_BEFORE_HELP: &str = concat!(
    "\x1b[97m╔╗ ╔═╗╦═╗╦╔╗╔╔═╗  ╔═╗╔═╗╔╦╗╔╦╗╦╔╦╗\x1b[0m\n",
    "\x1b[97m╠╩╗║ ║╠╦╝║║║║║ ╦  ║  ║ ║║║║║║║║ ║ \x1b[0m\n",
    "\x1b[97m╚═╝╚═╝╩╚═╩╝╚╝╚═╝  ╚═╝╚═╝╩ ╩╩ ╩╩ ╩\x1b[0m\n",
    "\x1b[90m────────────────────────────────────────\x1b[0m"
);

#[derive(Parser)]
#[command(
    name = "bcommit",
    visible_alias = "boringcommit",
    version,
    about = "BoringCommit — generate commit messages from staged changes using LLMs",
    long_about = "BoringCommit — generate commit messages from staged changes using LLMs\n\nCommands:\n  bcommit                      Generate commit message from staged changes (default)\n  bcommit config               Configure provider and model\n  bcommit model                Choose active model\n  bcommit model remove         Remove a model\n  bcommit sysprompt            Show system prompt (editable)\n  bcommit sysprompt edit       Edit prompt in $EDITOR\n  bcommit sysprompt reset      Reset prompt to default\n  bcommit sysprompt path       Show prompt file path",
    disable_help_subcommand = true,
    before_help = BRAND_BEFORE_HELP,
    help_template = "{before-help}\n{about}\n\n{usage-heading} {usage}\n\n{all-args}\n\n{after-help}"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Skip confirmation and auto-commit
    #[arg(long, short)]
    yes: bool,
}

#[derive(Subcommand)]
enum ModelAction {
    /// Remove a saved model from a provider
    Remove,
}

#[derive(Subcommand)]
enum SyspromptAction {
    /// Show current system prompt
    Show,
    /// Edit system prompt in $EDITOR
    Edit,
    /// Reset system prompt to default
    Reset,
    /// Show path to prompt file
    Path,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure provider and model
    Config,
    /// Choose active model
    Model {
        #[command(subcommand)]
        action: Option<ModelAction>,
    },
    /// Manage system prompt for commit generation (editable)
    Sysprompt {
        #[command(subcommand)]
        action: Option<SyspromptAction>,
    },
}

fn with_braille_spinner<T, F: FnOnce() -> T + Send + 'static>(msg: &str, f: F) -> T
where
    T: Send + 'static,
{
    // High-Quality Unicode Braille Dots — True Spinner
    let braille = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let cyan = "\x1b[36m";
    let reset = "\x1b[0m";
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let msg_owned = msg.to_string();

    let handle = std::thread::spawn(move || {
        let mut i = 0usize;
        while running_clone.load(Ordering::Relaxed) {
            print!("\r{}{}{} {}", cyan, braille[i % braille.len()], reset, msg_owned);
            let _ = std::io::stdout().flush();
            std::thread::sleep(Duration::from_millis(80));
            i += 1;
        }
    });

    let result = f();

    running.store(false, Ordering::Relaxed);
    let _ = handle.join();
    // clear spinner line
    print!("\r{}\r", " ".repeat(msg.len() + 4));
    let _ = std::io::stdout().flush();
    result
}

fn run_generate(auto_yes: bool) {
    let config = parse_config_file();

    if config.provider.is_empty() || config.model.is_empty() {
        ui::error("No provider/model configured. Run `bcommit config` first.");
        std::process::exit(1);
    }

    let provider_info = match config.providers.get(&config.provider) {
        Some(p) => p.clone(),
        None => {
            ui::error(&format!("Provider '{}' not found in config", config.provider));
            std::process::exit(1);
        }
    };

    // ensure staged changes exist
    let diff = git::staged_changes();

    // GitHub-style stats with colors
    let stats = git::staged_stats();
    git::print_staged_summary(&stats);

    let spinner_msg = format!("Generating with {} / {} ...", config.provider, config.model);
    let msg = with_braille_spinner(&spinner_msg, move || {
        match llm::generate_commit_message(&diff, &config.provider, &provider_info, &config.model) {
            Ok(m) => m,
            Err(e) => {
                // need to clear spinner before eprintln
                ui::error(&format!("\n{}", e));
                std::process::exit(1);
            }
        }
    });

    println!("\n{}\n", msg);

    if auto_yes {
        match git::commit_with_message(&msg) {
            Ok(_) => ui::success("Committed"),
            Err(e) => ui::error(&e),
        }
        return;
    }

    // interactive confirm / edit — Edit opens $EDITOR with multiline support
    let render_config = RenderConfig::default_colored()
        .with_prompt_prefix(inquire::ui::Styled::new("?").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_answered_prompt_prefix(inquire::ui::Styled::new("❯").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_highlighted_option_prefix(inquire::ui::Styled::new("❯").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
        .with_text_input(StyleSheet::new().with_fg(Color::LightBlue))
        .with_default_value(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_help_message(StyleSheet::new().with_fg(Color::LightCyan))
        .with_answer(StyleSheet::new().with_fg(Color::LightCyan))
        .with_option(StyleSheet::new().with_fg(Color::White))
        .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD)));
    set_global_render_config(render_config);

    let choice = inquire::Select::new(
        "What to do?",
        vec!["Commit".to_string(), "Edit".to_string(), "Regenerate".to_string(), "Cancel".to_string()],
    )
    .with_help_message("(e) to open vim, (enter) to submit — Edit opens $EDITOR")
    .prompt()
    .unwrap_or_else(|_| "Cancel".to_string());

    match choice.as_ref() {
        "Commit" => {
            if let Err(e) = git::commit_with_message(&msg) {
                ui::error(&e);
            } else {
                ui::success("Committed");
            }
        }
        "Edit" => {
            let edited = match inquire::Editor::new("Edit commit message:")
                .with_predefined_text(&msg)
                .with_help_message("(e) to open vim, (enter) to submit — Save and close editor ($EDITOR), you’ll be asked to confirm")
                .with_file_extension(".md")
                .prompt()
            {
                Ok(s) => s,
                Err(inquire::error::InquireError::OperationCanceled) => {
                    println!("Edit cancelled");
                    return;
                }
                Err(inquire::error::InquireError::OperationInterrupted) => {
                    println!("Edit interrupted — cancelled");
                    return;
                }
                Err(e) => {
                    ui::error(&format!("Editor error: {}", e));
                    return;
                }
            };
            let edited = edited.trim().to_string();
            if edited.is_empty() {
                ui::error("Commit message empty — cancelled");
                return;
            }
            // Don't commit directly after save — ask for confirmation
            println!("\nEdited message:\n{}\n", edited);
            let confirm = inquire::Select::new(
                "Commit this edited message?",
                vec!["Commit".to_string(), "Edit again".to_string(), "Cancel".to_string()],
            )
            .with_help_message("(enter) to submit")
            .prompt()
            .unwrap_or_else(|_| "Cancel".to_string());
            match confirm.as_str() {
                "Commit" => {
                    if let Err(e) = git::commit_with_message(&edited) {
                        ui::error(&e);
                    } else {
                        ui::success("Committed");
                    }
                }
                "Edit again" => {
                    // Re-enter edit flow with the edited text as base
                    let re_edited = match inquire::Editor::new("Edit commit message:")
                        .with_predefined_text(&edited)
                        .with_help_message("(e) to open vim, (enter) to submit — Save and close editor ($EDITOR), you’ll be asked to confirm")
                        .with_file_extension(".md")
                        .prompt()
                    {
                        Ok(s) => s.trim().to_string(),
                        Err(inquire::error::InquireError::OperationCanceled) => {
                            println!("Edit cancelled");
                            return;
                        }
                        Err(inquire::error::InquireError::OperationInterrupted) => {
                            println!("Edit interrupted — cancelled");
                            return;
                        }
                        Err(e) => {
                            ui::error(&format!("Editor error: {}", e));
                            return;
                        }
                    };
                    if re_edited.is_empty() {
                        ui::error("Commit message empty — cancelled");
                        return;
                    }
                    println!("\nEdited message:\n{}\n", re_edited);
                    let final_confirm = inquire::Confirm::new("Commit this edited message?")
                        .with_default(true)
                        .prompt()
                        .unwrap_or(false);
                    if final_confirm {
                        if let Err(e) = git::commit_with_message(&re_edited) {
                            ui::error(&e);
                        } else {
                            ui::success("Committed");
                        }
                    } else {
                        println!("Cancelled. Edited message was:\n{}", re_edited);
                    }
                }
                _ => {
                    println!("Cancelled. Edited message was:\n{}", edited);
                }
            }
        }
        "Regenerate" => {
            // simple recursion
            run_generate(false);
        }
        _ => {
            println!("Cancelled. Message was:\n{}", msg);
        }
    }
}

fn main() {
    setup_config();
    setup_instructions();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config) => config_provider(),
        Some(Commands::Model { action: Some(ModelAction::Remove) }) => remove_model(),
        Some(Commands::Model { action: None }) => choose_model(),
        Some(Commands::Sysprompt { action: Some(SyspromptAction::Show) }) => show_instructions(),
        Some(Commands::Sysprompt { action: Some(SyspromptAction::Edit) }) => edit_instructions(),
        Some(Commands::Sysprompt { action: Some(SyspromptAction::Reset) }) => reset_instructions(),
        Some(Commands::Sysprompt { action: Some(SyspromptAction::Path) }) => instructions_path(),
        Some(Commands::Sysprompt { action: None }) => show_instructions(),
        None => run_generate(cli.yes),
    }
}
