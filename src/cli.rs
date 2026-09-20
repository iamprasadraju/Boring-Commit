use crate::llm::auth_provider;
use crate::setup::{
    get_ollama_models, instructions_file_path, is_ollama_installed, is_ollama_running,
    ollama_pull_model, parse_config_file, read_instructions,
    reset_instructions as setup_reset_instructions, save_config, setup_instructions, setup_ollama,
};
use std::process::Command;

use inquire::{Confirm, Password, PasswordDisplayMode, Select, Text, set_global_render_config};
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet};

pub fn config_provider(){
    let render_config = RenderConfig::default_colored()
        .with_prompt_prefix(inquire::ui::Styled::new("?").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_answered_prompt_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_highlighted_option_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
        .with_text_input(StyleSheet::new().with_fg(Color::LightBlue))
        .with_default_value(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_help_message(StyleSheet::new().with_fg(Color::LightCyan))
        .with_answer(StyleSheet::new().with_fg(Color::LightCyan))
        .with_option(StyleSheet::new().with_fg(Color::White))
        .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD)));

    set_global_render_config(render_config);
    
    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║      BoringCommit Config             ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let mut config = parse_config_file();

    let providers: Vec<String> =
        config.providers.keys().cloned().collect();

    let provider = Select::new("Select a Provider:", providers)
        .with_help_message("Choose your LLM provider")
        .prompt()
        .unwrap();

    let provider_info = config
        .providers
        .get(&provider)
        .expect("Provider not found")
        .clone();

    let api_key: String;

    if provider == "ollama" {
        // Local provider — check installation, skip API key
        if !is_ollama_installed() {
            println!("Ollama not found on system.");
            let do_install = Confirm::new("Install Ollama now?")
                .with_default(true)
                .with_help_message("Runs setup_ollama (curl | sh)")
                .prompt()
                .unwrap();
            if do_install {
                setup_ollama();
                if !is_ollama_installed() {
                    eprintln!("Ollama installation failed");
                    std::process::exit(1);
                }
                println!("Ollama installed successfully");
            } else {
                eprintln!("Ollama is required for local provider");
                std::process::exit(1);
            }
        } else {
            println!("Ollama is installed");
        }

        if !is_ollama_running(&provider_info.endpoint) {
            eprintln!("Warning: Ollama is not running at {} — run `ollama serve`", provider_info.endpoint);
            // continue anyway, model check will fail if not running
        }

        api_key = String::new();
        config.providers.get_mut(&provider).unwrap().api_key = None;

        // verify Ollama connectivity (no API key)
        match auth_provider(&provider, &provider_info, "", "") {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        let has_existing = provider_info
            .api_key
            .as_ref()
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false);
        let help_msg = if has_existing {
            "Current: ***** (leave empty to keep, or enter new)"
        } else {
            "Enter your API key"
        };
        let input = Password::new("API Key:")
            .with_display_mode(PasswordDisplayMode::Masked)
            .without_confirmation()
            .with_help_message(help_msg)
            .prompt()
            .unwrap();
        let key = if input.trim().is_empty() && has_existing {
            provider_info.api_key.clone().unwrap()
        } else {
            input
        };
        api_key = key;
        config.providers.get_mut(&provider).unwrap().api_key = Some(api_key.clone());

        match auth_provider(&provider, &provider_info, &api_key, "") {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    let model: String = if provider == "ollama" {
        // Ollama recommended models with pulling
        let installed = get_ollama_models(&provider_info.endpoint);
        let rec1 = "qwen2.5-coder:3b";
        let rec2 = "phi4-mini:3.8b";
        let rec_display1 = format!("{} (Recommended by creator)", rec1);
        let rec_display2 = format!("{} (Recommended by creator)", rec2);

        let mut options: Vec<String> = Vec::new();
        options.push(rec_display1.clone());
        options.push(rec_display2.clone());
        for m in &installed {
            if m != rec1 && m != rec2 {
                options.push(m.clone());
            }
        }
        options.push("Custom / Enter manually".to_string());

        let selected = Select::new("Select Ollama Model:", options)
            .with_help_message("Recommended models are marked — will auto-pull if not installed")
            .prompt()
            .unwrap();

        let chosen = if selected == rec_display1 {
            rec1.to_string()
        } else if selected == rec_display2 {
            rec2.to_string()
        } else if selected == "Custom / Enter manually" {
            Text::new("Add Model:")
                .with_help_message("Enter Ollama model (e.g. qwen2.5-coder:3b)")
                .prompt()
                .unwrap()
        } else {
            selected
        };

        if chosen.trim().is_empty() {
            eprintln!("Model is required");
            std::process::exit(1);
        }

        // prompt to pull if not installed (rather than auto-save)
        if !installed.contains(&chosen) {
            let do_pull = Confirm::new(&format!("Model '{}' not installed. Pull it now?", chosen))
                .with_default(true)
                .with_help_message("Runs `ollama pull <model>`")
                .prompt()
                .unwrap();
            if do_pull {
                if let Err(e) = ollama_pull_model(&chosen) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                println!("Pulled '{}'", chosen);
            } else {
                println!("Warning: Skipping pull — model will be pulled on first use");
            }
        } else {
            println!("Model '{}' already installed", chosen);
        }

        chosen
    } else {
        let m = Text::new("Add Model:")
            .with_help_message("Enter the model name to use (e.g. openai/gpt-4o, groq/llama-3.1-8b-instant)")
            .prompt()
            .unwrap();
        if m.trim().is_empty() {
            eprintln!("Model is required");
            std::process::exit(1);
        }
        m
    };

    // verify model exists for provider before saving (ollama handled via pull prompt)
    if provider != "ollama" {
        match auth_provider(&provider, &provider_info, &api_key, &model) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    if provider != "ollama" {
        let entry = config.providers.get_mut(&provider).unwrap();
        if !entry.models.contains(&model) {
            entry.models.push(model.clone());
        }
    }

    config.provider = provider.clone();
    config.model = model;

    save_config(&config);
    if provider == "ollama" {
        println!("Ollama model '{}' set as active (managed via `ollama list`)", config.model);
    } else {
        println!("Configuration complete!");
    }
}

pub fn choose_model() {
    let render_config = RenderConfig::default_colored()
        .with_prompt_prefix(inquire::ui::Styled::new("?").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_answered_prompt_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_highlighted_option_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
        .with_text_input(StyleSheet::new().with_fg(Color::LightBlue))
        .with_default_value(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_help_message(StyleSheet::new().with_fg(Color::LightCyan))
        .with_answer(StyleSheet::new().with_fg(Color::LightCyan))
        .with_option(StyleSheet::new().with_fg(Color::White))
        .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD)));

    set_global_render_config(render_config);

    let mut config = parse_config_file();

    // show active model
    if !config.provider.is_empty() && !config.model.is_empty() {
        println!("Active: {} / {}", config.provider, config.model);
    } else {
        println!("No active model configured");
    }
    println!();

    let providers: Vec<String> = config.providers.keys().cloned().collect();

    if providers.is_empty() {
        eprintln!("No providers configured");
        std::process::exit(1);
    }

    let provider_start = providers.iter().position(|p| p == &config.provider).unwrap_or(0);
    let provider = Select::new("Select a Provider:", providers)
        .with_help_message("Choose provider to pick a model from")
        .with_starting_cursor(provider_start)
        .prompt()
        .unwrap();

    let provider_info = config
        .providers
        .get(&provider)
        .expect("Provider not found")
        .clone();

    if provider == "ollama" && !is_ollama_installed() {
        println!("Ollama not found on system.");
        let do_install = Confirm::new("Install Ollama now?")
            .with_default(true)
            .prompt()
            .unwrap();
        if do_install {
            setup_ollama();
        } else {
            eprintln!("Ollama is required for local provider");
            std::process::exit(1);
        }
    }

    if provider == "ollama" {
        let ollama_models = get_ollama_models(&provider_info.endpoint);
        if ollama_models.is_empty() {
            eprintln!("No models found via `ollama list`. Pull one with `ollama pull <model>` or run `bcommit config`");
            std::process::exit(1);
        }
        let model_start = if config.provider == provider {
            ollama_models.iter().position(|m| m == &config.model).unwrap_or(0)
        } else {
            0
        };
        let model = Select::new("Select Ollama Model:", ollama_models.clone())
            .with_help_message("Models from `ollama list` — > to select")
            .with_starting_cursor(model_start)
            .prompt()
            .unwrap();
        config.provider = provider.clone();
        config.model = model.clone();
    } else {
        if provider_info.models.is_empty() {
            eprintln!("No saved models for '{}'. Run `bcommit config` to add one.", provider);
            std::process::exit(1);
        }

        let model_start = if config.provider == provider {
            provider_info.models.iter().position(|m| m == &config.model).unwrap_or(0)
        } else {
            0
        };
        let model = Select::new("Select Model:", provider_info.models.clone())
            .with_help_message("Choose a saved model for this provider")
            .with_starting_cursor(model_start)
            .prompt()
            .unwrap();

        config.provider = provider.clone();
        config.model = model.clone();
    }

    save_config(&config);
    println!("Active model: {} / {}", config.provider, config.model);
    println!("Active model set to '{}' for provider '{}'", config.model, provider);
}

pub fn remove_model() {
    let render_config = RenderConfig::default_colored()
        .with_prompt_prefix(inquire::ui::Styled::new("?").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_answered_prompt_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_highlighted_option_prefix(inquire::ui::Styled::new(">").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
        .with_text_input(StyleSheet::new().with_fg(Color::LightBlue))
        .with_default_value(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_help_message(StyleSheet::new().with_fg(Color::LightCyan))
        .with_answer(StyleSheet::new().with_fg(Color::LightCyan))
        .with_option(StyleSheet::new().with_fg(Color::White))
        .with_selected_option(Some(StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD)));

    set_global_render_config(render_config);

    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║      Remove Model                    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let mut config = parse_config_file();

    let providers: Vec<String> = config.providers.keys().cloned().collect();
    if providers.is_empty() {
        eprintln!("No providers configured");
        std::process::exit(1);
    }

    if !config.provider.is_empty() && !config.model.is_empty() {
        println!("Active: {} / {}", config.provider, config.model);
        println!();
    }

    let provider_start = providers.iter().position(|p| p == &config.provider).unwrap_or(0);
    let provider = Select::new("Select a Provider:", providers)
        .with_help_message("Choose provider to remove a model from")
        .with_starting_cursor(provider_start)
        .prompt()
        .unwrap();

    if provider == "ollama" {
        eprintln!("Remove disabled for ollama — models are managed via `ollama rm <model>` (use `ollama list` to view)");
        std::process::exit(1);
    }

    let provider_info = config
        .providers
        .get(&provider)
        .expect("Provider not found")
        .clone();

    if provider_info.models.is_empty() {
        eprintln!("No saved models for '{}'", provider);
        std::process::exit(1);
    }

    let model = Select::new("Select Model to Remove:", provider_info.models.clone())
        .with_help_message("> to select, Enter to confirm")
        .prompt()
        .unwrap();

    let confirm = Confirm::new(&format!("Remove '{}' from '{}'?", model, provider))
        .with_default(false)
        .prompt()
        .unwrap();

    if !confirm {
        println!("Cancelled");
        return;
    }

    {
        let entry = config.providers.get_mut(&provider).unwrap();
        entry.models.retain(|m| m != &model);
    }

    // clear active if it was the removed model
    if config.provider == provider && config.model == model {
        config.model = String::new();
        // keep provider selected but model cleared; user can pick another via `bcommit model`
        println!("Warning: Active model was removed — active model cleared");
    }

    save_config(&config);
    println!("Removed '{}' from '{}'", model, provider);
}

pub fn show_instructions() {
    setup_instructions();
    let path = instructions_file_path();
    let content = read_instructions();
    println!();
    println!("Instructions file: {}", path.display());
    println!("────────────────────────────────────────");
    println!("{}", content);
    println!("────────────────────────────────────────");
    println!("Edit with: bcommit sysprompt edit  |  Reset with: bcommit sysprompt reset");
}

pub fn edit_instructions() {
    setup_instructions();
    let path = instructions_file_path();
    println!("Opening instructions file: {}", path.display());
    println!("Edit and save to customize commit generation. Reset with: bcommit sysprompt reset");
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });
    let status = Command::new(&editor).arg(&path).status();
    match status {
        Ok(s) if s.success() => {
            println!("Instructions saved to {}", path.display());
            let content = read_instructions();
            if content.trim().is_empty() {
                eprintln!("Warning: instructions file is empty — LLM will use empty system prompt");
            }
        }
        Ok(s) => eprintln!("Editor exited with status {:?}", s),
        Err(e) => eprintln!("Failed to launch editor '{}': {}. Edit manually: {}", editor, e, path.display()),
    }
}

pub fn reset_instructions() {
    let confirm = Confirm::new("Reset instructions to default?")
        .with_default(false)
        .with_help_message("Overwrites current instructions.md")
        .prompt()
        .unwrap();
    if !confirm {
        println!("Cancelled");
        return;
    }
    setup_reset_instructions();
    println!("Instructions reset to default at {}", instructions_file_path().display());
    let content = read_instructions();
    println!("────────────────────────────────────────");
    println!("{}", content);
    println!("────────────────────────────────────────");
}

pub fn instructions_path() {
    setup_instructions();
    println!("{}", instructions_file_path().display());
}
