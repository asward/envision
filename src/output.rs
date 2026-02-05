use std::io::{self, Write};

pub struct Output {
    color: bool,
}

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

impl Output {
    pub fn new() -> Self {
        let no_color_flag = std::env::args().any(|a| a == "--no-color");
        let color = !no_color_flag && std::env::var("NO_COLOR").is_err();
        Self { color }
    }

    fn styled(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("{code}{text}{RESET}")
        } else {
            text.to_string()
        }
    }

    pub fn success(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "{}", self.styled(GREEN, msg));
    }

    pub fn info(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "{}", msg);
    }

    pub fn warn(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "{}", self.styled(YELLOW, msg));
    }

    pub fn error(&self, msg: &str) {
        let _ = writeln!(io::stderr(), "{}", self.styled(RED, msg));
    }

    pub fn bold(&self, msg: &str) -> String {
        self.styled(BOLD, msg)
    }

    pub fn dim(&self, msg: &str) -> String {
        self.styled(DIM, msg)
    }

    pub fn key_value(&self, key: &str, value: &str) {
        let _ = writeln!(io::stderr(), "  {}: {}", self.bold(key), value);
    }
}
