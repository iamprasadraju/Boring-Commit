use std::process::Command;

pub fn staged_changes() -> String{
    let output = Command::new("git")
        .args(["diff", "--staged", "--unified=0"])
        .output()
        .expect("Git command failed");

    // println!("{:?}", output);
    // Output { status: ExitStatus(unix_wait_status(0)), stdout: ""}

    if output.stdout.is_empty(){
        println!("No staged changes.");
        std::process::exit(0); 
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}