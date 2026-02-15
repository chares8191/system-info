use std::env;
use std::process::Command;

pub fn run_command_string(cmd: &str, args: &[&str]) -> String {
    let output = match Command::new(cmd).args(args).output() {
        Ok(output) => output,
        Err(_) => return String::new(),
    };
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

pub fn run_command_optional(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn command_colon_field(cmd: &str, args: &[&str], label: &str) -> String {
    let output = run_command_string(cmd, args);
    let prefix = if label.ends_with(':') {
        label.to_string()
    } else {
        format!("{label}:")
    };
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().to_string();
        }
    }
    String::new()
}

pub fn trim_tree_prefix(value: &str) -> &str {
    value.trim_start_matches(|c: char| {
        c == '⎡' || c == '⎜' || c == '⎣' || c == '↳' || c.is_whitespace()
    })
}

pub fn current_username() -> String {
    env::var("USER").unwrap_or_else(|_| run_command_string("id", &["-un"]))
}

pub fn env_var_opt(name: &str) -> Option<String> {
    env::var(name).ok()
}
