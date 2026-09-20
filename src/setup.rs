use std::collections::HashMap;

use dirs;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};


// read this file at compile time and turn its contents into string
const DEFAULT_CONFIG: &str = include_str!("config/config.json");
const DEFAULT_INSTRUCTIONS: &str = include_str!("config/instructions.md");

pub fn is_ollama_installed() -> bool {
    Command::new("ollama")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_ollama_running(endpoint: &str) -> bool {
    reqwest::blocking::Client::new()
        .get(format!("{}/api/tags", endpoint.trim_end_matches('/')))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

pub fn get_ollama_models(endpoint: &str) -> Vec<String> {
    // Try HTTP API first
    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    if let Ok(resp) = reqwest::blocking::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
    {
        if let Ok(v) = resp.json::<serde_json::Value>() {
            let mut names = Vec::new();
            if let Some(models) = v.get("models").and_then(|m| m.as_array()) {
                for m in models {
                    if let Some(name) = m.get("name").and_then(|s| s.as_str()) {
                        names.push(name.to_string());
                    } else if let Some(model) = m.get("model").and_then(|s| s.as_str()) {
                        names.push(model.to_string());
                    }
                }
            }
            if !names.is_empty() {
                return names;
            }
        }
    }
    // Fallback to `ollama list` command
    if let Ok(out) = Command::new("ollama").arg("list").output() {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout);
            let mut names = Vec::new();
            for line in txt.lines().skip(1) {
                if let Some(name) = line.split_whitespace().next() {
                    if !name.is_empty() && name != "NAME" {
                        names.push(name.to_string());
                    }
                }
            }
            if !names.is_empty() {
                return names;
            }
        }
    }
    Vec::new()
}

pub fn ollama_pull_model(model: &str) -> Result<(), String> {
    println!("Pulling Ollama model '{}' ... (this may take a while)", model);
    let status = Command::new("ollama")
        .args(["pull", model])
        .status()
        .map_err(|e| format!("Failed to run ollama pull: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ollama pull failed with status {:?}", status))
    }
}

pub fn setup_ollama() {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub providers: HashMap<String, Provider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Provider {
    pub endpoint: String,
    pub api_key: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
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
    let path: PathBuf = config_file_path();

    let contents = fs::read_to_string(path).expect("Unable to read config file");
    serde_json::from_str(&contents).expect("Invalid config file")
}

pub fn save_config(config: &Config) {
    let path = config_file_path();
    let contents = serde_json::to_string_pretty(config)
        .expect("Unable to serialize config");
    fs::write(&path, contents).expect("Unable to write config file");
}

pub fn instructions_file_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("could not find config directory")
        .join("bcommit");
    fs::create_dir_all(&config_dir).expect("failed to create config directory");
    config_dir.join("instructions.md")
}

pub fn setup_instructions() {
    let path = instructions_file_path();
    if !path.exists() {
        fs::create_dir_all(path.parent().unwrap()).expect("Unable to create config directory");
        fs::write(&path, DEFAULT_INSTRUCTIONS).expect("Unable to write default instructions");
    }
}

pub fn read_instructions() -> String {
    let path = instructions_file_path();
    fs::read_to_string(&path).unwrap_or_else(|_| DEFAULT_INSTRUCTIONS.to_string())
}

pub fn reset_instructions() {
    let path = instructions_file_path();
    fs::create_dir_all(path.parent().unwrap()).expect("Unable to create config directory");
    fs::write(&path, DEFAULT_INSTRUCTIONS).expect("Unable to reset instructions");
}
