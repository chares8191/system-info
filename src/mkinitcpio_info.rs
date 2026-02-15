use serde::Serialize;

use crate::utils::run_command_optional;

#[derive(Serialize)]
pub struct MkinitcpioInfo {
    modules: Vec<String>,
    hooks: Vec<String>,
}

fn parse_mkinitcpio_list(line: &str) -> Vec<String> {
    let start = match line.find('(') {
        Some(pos) => pos + 1,
        None => return Vec::new(),
    };
    let end = match line.rfind(')') {
        Some(pos) if pos > start => pos,
        _ => return Vec::new(),
    };
    line[start..end]
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

pub fn mkinitcpio_info() -> MkinitcpioInfo {
    let modules_line = run_command_optional("grep", &["-E", "^MODULES=", "/etc/mkinitcpio.conf"])
        .unwrap_or_default();
    let hooks_line = run_command_optional("grep", &["-E", "^HOOKS=", "/etc/mkinitcpio.conf"])
        .unwrap_or_default();
    MkinitcpioInfo {
        modules: parse_mkinitcpio_list(&modules_line),
        hooks: parse_mkinitcpio_list(&hooks_line),
    }
}
