use serde::Serialize;

use crate::utils::run_command_optional;

#[derive(Serialize)]
pub struct PacmanInfo {
    explicit: Vec<String>,
}

pub fn pacman_info() -> PacmanInfo {
    let output = run_command_optional("pacman", &["-Qe"]).unwrap_or_default();
    let explicit = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    PacmanInfo { explicit }
}
