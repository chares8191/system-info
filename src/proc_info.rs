use serde::Serialize;

use crate::utils::run_command_string;

#[derive(Serialize)]
pub struct ProcInfo {
    cmdline: String,
    meminfo: ProcMemInfo,
    version: String,
}

#[derive(Serialize)]
pub struct ProcMemInfo {
    mem_total: String,
}

fn parse_proc_meminfo(value: &str) -> ProcMemInfo {
    let mut items = std::collections::HashMap::new();
    for line in value.lines() {
        if let Some((key, rest)) = line.split_once(':') {
            items.insert(key.trim(), rest.trim().to_string());
        }
    }
    let get = |key: &str| items.get(key).cloned().unwrap_or_default();
    ProcMemInfo {
        mem_total: get("MemTotal"),
    }
}

pub fn proc_info() -> ProcInfo {
    let proc_path = |name: &str| format!("/proc/{name}");
    let cat_proc = |name: &str| run_command_string("cat", &[&proc_path(name)]);
    ProcInfo {
        cmdline: cat_proc("cmdline"),
        meminfo: parse_proc_meminfo(&cat_proc("meminfo")),
        version: cat_proc("version"),
    }
}
