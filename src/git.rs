use std::process::Command;

pub fn staged_changes() -> String{
    let output = Command::new("git")
        .args(["diff", "--staged", "--unified=0"])
        .output()
        .expect("Git command failed");

    if output.stdout.is_empty(){
        println!("No staged changes. Stage files with `git add` first.");
        std::process::exit(0); 
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

pub fn staged_stats() -> DiffStats {
    let output = Command::new("git")
        .args(["diff", "--staged", "--numstat"])
        .output()
        .expect("Git command failed");

    let text = String::from_utf8_lossy(&output.stdout);
    let mut files_changed = 0usize;
    let mut insertions = 0usize;
    let mut deletions = 0usize;

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // numstat: <ins>\t<del>\t<file>
        let mut parts = line.split('\t');
        let ins = parts.next().unwrap_or("0");
        let del = parts.next().unwrap_or("0");
        files_changed += 1;
        if ins != "-" {
            if let Ok(n) = ins.parse::<usize>() {
                insertions += n;
            }
        }
        if del != "-" {
            if let Ok(n) = del.parse::<usize>() {
                deletions += n;
            }
        }
    }

    DiffStats { files_changed, insertions, deletions }
}

pub fn print_staged_summary(stats: &DiffStats) {
    let reset = "\x1b[0m";
    let green = "\x1b[32m";
    let red = "\x1b[31m";
    let bold = "\x1b[1m";
    println!(
        " {}{} files changed{}, {}{} insertions(+){}, {}{} deletions(-){}",
        bold, stats.files_changed, reset,
        green, stats.insertions, reset,
        red, stats.deletions, reset
    );
}

pub fn commit_with_message(msg: &str) -> Result<(), String> {
    let status = Command::new("git")
        .args(["commit", "-m", msg])
        .status()
        .map_err(|e| format!("Failed to run git commit: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("git commit failed with status {:?}", status))
    }
}
