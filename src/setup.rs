use std::collections::HashMap;

use dirs;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;


// read this file at compile time and turn its contents into string
const DEFAULT_CONFIG: &str = include_str!("config/config.json");

pub fn setup_ollama() {
    println!("Boring Commit setup => ");
    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", "irm https://ollama.com/install.ps1 | iex"])
            .status()
            .expect("failed to install Ollama");
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://ollama.com/install.sh | sh")
            .status()
            .expect("failed to install Ollama");
    }
}

/*  =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
    setup and parsing config.json file
    -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-= */

#[derive(Debug, Deserialize)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub providers: HashMap<String, Provider>,
}

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub endpoint: String,
    pub api_key: Option<String>,
}

pub fn config_file_path() -> PathBuf {
    //Linux: ~/.config/bcommit/config.toml
    // macOS: ~/Library/Application Support/bcommit/config.toml
    // Windows: %APPDATA%\bcommit\config.toml

    let config_dir = dirs::config_dir()
        .expect("could not find config directory")
        .join("bcommit");

    fs::create_dir_all(&config_dir).expect("failed to create config directory");

    config_dir.join("config.json")
}

pub fn setup_config() {
    let path = config_file_path();

    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap()).expect("Unable to create config directory");

        fs::write(&path, DEFAULT_CONFIG).expect("Unable to write default config");
    }
}

pub fn parse_config_file() -> Config {
    let path = config_file_path();

    let contents = fs::read_to_string(path).expect("Unable to read config file");
    serde_json::from_str(&contents).expect("Invalid config file")
}
