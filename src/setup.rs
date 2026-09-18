use std::process::Command;
use std::path::PathBuf;
use std::env;

/*pub fn setup(){
    
        
}
*/

pub fn ollama(){
    println!("Boring Commit setup => ");
        if cfg!(target_os = "windows"){
            Command::new("powershell")
                .args(["-Command", "irm https://ollama.com/install.ps1 | iex"])
                .status()
                .expect("failed to install Ollama");
        }
        else {
            Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://ollama.com/install.sh | sh")
                .status()
                .expect("failed to install Ollama");
        }
}

pub fn config_file(){
    
    // windows: C:\Users\<YourUsername>\AppData\Roaming\bcommit\config.toml
    // Mac/Linux: ~/.config/bcommit/config.toml

    let home_dir = env::home_dir()
        .expect("could not find home directory");

    println!("{}", windows_home_dir);

fn main(){
    config_file();
}
