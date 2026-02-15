use serde::Serialize;

use crate::utils::run_command_string;

#[derive(Serialize)]
pub struct KernelModuleInfo {
    module: String,
    size: String,
    used_by_count: String,
    used_by: Vec<String>,
}

fn parse_lsmod_line(line: &str) -> Option<KernelModuleInfo> {
    let mut parts = line.split_whitespace();
    let module = parts.next()?.to_string();
    let size = parts.next()?.to_string();
    let used_by_count = parts.next()?.to_string();
    let used_by_raw = parts.collect::<Vec<&str>>().join(" ");
    let used_by = used_by_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    Some(KernelModuleInfo {
        module,
        size,
        used_by_count,
        used_by,
    })
}

pub fn lsmod_info() -> Vec<KernelModuleInfo> {
    let output = run_command_string("lsmod", &[]);
    output
        .lines()
        .skip(1)
        .filter_map(parse_lsmod_line)
        .collect()
}
