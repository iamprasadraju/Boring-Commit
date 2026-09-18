use dirs;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/*pub fn setup(){


}
*/

// read this file at compile time and turn its contents into string
const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

pub fn ollama() {
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

pub fn config_file_path() -> PathBuf {
    //Linux: ~/.config/bcommit/config.toml
    // macOS: ~/Library/Application Support/bcommit/config.toml
    // Windows: %APPDATA%\bcommit\config.toml

    let config_dir = dirs::config_dir()
        .expect("could not find config directory")
        .join("bcommit");

    fs::create_dir_all(&config_dir).expect("failed to create config directory");

    config_dir.join("config.toml")
}

pub fn config_file_create() {
    let path = config_file_path();

    if path.exists() {
        return;
    }

    fs::write(path, DEFAULT_CONFIG).expect("failed to create config file")
}

pub fn config_file_read() -> String {
    let path = config_file_path();

    fs::read_to_string(path).expect("unable to read the file")
}

fn main() {
    // let path = config_file_path();
    config_file_create();

    let con = config_file_read();
    println!("{}", con);
}
