use std::process::Command;

pub fn setup(){
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