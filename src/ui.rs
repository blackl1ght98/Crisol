//! Salida de consola con un formato consistente. Sin dependencias externas.

pub fn step(current: usize, total: usize, msg: &str) {
    println!("\x1b[1;36m[{current}/{total}]\x1b[0m {msg}");
}

pub fn info(msg: &str) {
    println!("\x1b[0;34m  ▸\x1b[0m {msg}");
}

pub fn detail(msg: &str) {
    println!("\x1b[2m      {msg}\x1b[0m");
}

pub fn ok(msg: &str) {
    println!("\x1b[1;32m  ✓\x1b[0m {msg}");
}

pub fn warn(msg: &str) {
    println!("\x1b[1;33m  !\x1b[0m {msg}");
}

pub fn err(msg: &str) {
    eprintln!("\x1b[1;31m  ✗\x1b[0m {msg}");
}

pub fn header(msg: &str) {
    println!("\n\x1b[1;35m◆ {msg}\x1b[0m");
}
