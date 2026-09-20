pub fn success(msg: &str) {
    println!("\x1b[32m\x1b[1m✔ {}\x1b[0m", msg);
}

pub fn error(msg: &str) {
    eprintln!("\x1b[31m\x1b[1m✖ {}\x1b[0m", msg);
}

pub fn warn(msg: &str) {
    eprintln!("\x1b[33m⚠ {}\x1b[0m", msg);
}
