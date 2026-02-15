use serde::Serialize;

use crate::utils::run_command_string;

#[derive(Serialize)]
pub struct PciBusInfo {
    slot: String,
    class_name: String,
    class_code: String,
    device_description: String,
    vendor_id: String,
    device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subsystem_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subsystem_vendor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subsystem_device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_driver_in_use: Option<String>,
    kernel_modules: Vec<String>,
}

fn parse_named_ids(value: &str) -> (String, Option<String>, Option<String>) {
    let mut name = value.trim().to_string();
    let mut vendor_id = None;
    let mut device_id = None;
    if let Some(lb) = value.rfind('[') {
        if let Some(rb) = value[lb + 1..].find(']') {
            let ids = &value[lb + 1..lb + 1 + rb];
            if let Some((v, d)) = ids.split_once(':') {
                vendor_id = Some(v.to_string());
                device_id = Some(d.to_string());
            }
            name = value[..lb].trim_end().to_string();
        }
    }
    (name, vendor_id, device_id)
}

fn parse_pci_header(line: &str) -> PciBusInfo {
    let (slot, rest) = line
        .split_once(' ')
        .map(|(s, r)| (s, r))
        .unwrap_or((line, ""));
    let (class_part, after_class) = if let Some(pos) = rest.find("]: ") {
        (&rest[..pos + 1], &rest[pos + 3..])
    } else {
        (rest, "")
    };
    let (class_name, class_code) = if let Some(lb) = class_part.rfind('[') {
        let class_name = class_part[..lb].trim_end().to_string();
        let class_code = class_part[lb + 1..class_part.len() - 1].to_string();
        (class_name, class_code)
    } else {
        (class_part.trim().to_string(), String::new())
    };

    let mut after = after_class.trim().to_string();
    let mut revision = None;
    if let Some(rev_start) = after.rfind("(rev ") {
        if after.ends_with(')') {
            let rev = after[rev_start + 5..after.len() - 1].trim().to_string();
            revision = Some(rev);
            after = after[..rev_start].trim_end().to_string();
        }
    }

    let mut vendor_id = String::new();
    let mut device_id = String::new();
    let mut device_description = after.trim().to_string();
    if let Some(lb) = after.rfind('[') {
        if let Some(rb) = after[lb + 1..].find(']') {
            let ids = &after[lb + 1..lb + 1 + rb];
            if let Some((v, d)) = ids.split_once(':') {
                vendor_id = v.to_string();
                device_id = d.to_string();
            }
            device_description = after[..lb].trim_end().to_string();
        }
    }

    PciBusInfo {
        slot: slot.to_string(),
        class_name,
        class_code,
        device_description,
        vendor_id,
        device_id,
        revision,
        subsystem_name: None,
        subsystem_vendor_id: None,
        subsystem_device_id: None,
        kernel_driver_in_use: None,
        kernel_modules: Vec::new(),
    }
}

fn is_pci_header_line(line: &str) -> bool {
    let mut parts = line.split_whitespace();
    let slot = match parts.next() {
        Some(s) => s,
        None => return false,
    };
    if !slot.contains('.') || !slot.contains(':') {
        return false;
    }
    let mut iter = slot.split(':');
    let bus = match iter.next() {
        Some(b) => b,
        None => return false,
    };
    let dev_fn = match iter.next() {
        Some(d) => d,
        None => return false,
    };
    if iter.next().is_some() || bus.len() != 2 {
        return false;
    }
    let mut dev_fn_iter = dev_fn.split('.');
    let dev = match dev_fn_iter.next() {
        Some(d) => d,
        None => return false,
    };
    let func = match dev_fn_iter.next() {
        Some(f) => f,
        None => return false,
    };
    dev_fn_iter.next().is_none()
        && dev.len() == 2
        && func.len() == 1
        && bus.chars().all(|c| c.is_ascii_hexdigit())
        && dev.chars().all(|c| c.is_ascii_hexdigit())
        && func.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn pci_info() -> Vec<PciBusInfo> {
    let output = run_command_string("lspci", &["-nnk"]);
    let mut devices = Vec::new();
    let mut current: Option<PciBusInfo> = None;
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if is_pci_header_line(line) {
            if let Some(info) = current.take() {
                devices.push(info);
            }
            current = Some(parse_pci_header(line));
            continue;
        }

        let trimmed = line.trim_start();
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if let Some(ref mut info) = current {
                match key {
                    "Subsystem" => {
                        let (name, vendor_id, device_id) = parse_named_ids(value);
                        info.subsystem_name = Some(name);
                        info.subsystem_vendor_id = vendor_id;
                        info.subsystem_device_id = device_id;
                    }
                    "Kernel driver in use" => {
                        info.kernel_driver_in_use = Some(value.to_string());
                    }
                    "Kernel modules" => {
                        info.kernel_modules = value
                            .split(',')
                            .map(|m| m.trim())
                            .filter(|m| !m.is_empty())
                            .map(String::from)
                            .collect();
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(info) = current.take() {
        devices.push(info);
    }
    devices
}
