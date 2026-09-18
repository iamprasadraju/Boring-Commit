use std::process::Command;
use std::env;
use std::fs;
use std::path::PathBuf;

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

pub fn config_file_path() -> PathBuf {
    
    // macOS/linux: ~/.config/bcommit/config.toml
    // Windows: C:\Users\<YourUsername>\AppData\Roaming\bcommit\config.toml
    
    let home_dir = env::home_dir()
    .expect("could not find home directory");

    let config_dir = if cfg!(target_os = "windows") {
        home_dir
            .join("AppData")
            .join("Roaming")
            .join("bcommit")   
    }else {
        home_dir
            .join(".config")
            .join("bcommit")
    };

    fs::create_dir_all(&config_dir)
        .expect("failed to create config dir");
    
    config_dir.join("config.toml")    
}

pub fn config_file_create(){
    let boring_line = "# Boring Commit Config";
    fs::write(&config_file_path(), boring_line)
            .expect("failed to create config file");
}

pub fn config_file_read() -> String{
    fs::read_to_string(&config_file_path())
        .expect("unable to read the file")
}

fn main(){
    // let path = config_file_path();
    // config_file_create();
    
    let con = config_file_read();
    
    println!("{:?}", con);
}
