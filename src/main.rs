use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use std::env;
use std::process::Command;

fn run_command_string(cmd: &str, args: &[&str]) -> String {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .expect("failed to run command");
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

fn command_colon_field(cmd: &str, args: &[&str], label: &str) -> String {
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

#[derive(Serialize)]
struct UnameInfo {
    kernel_release: String,
    machine: String,
}

fn uname_info() -> UnameInfo {
    let kernel_release = || run_command_string("uname", &["-r"]);
    let machine = || run_command_string("uname", &["-m"]);
    UnameInfo {
        kernel_release: kernel_release(),
        machine: machine(),
    }
}

#[derive(Serialize)]
struct DmiInfo {
    bios_date: String,
    bios_vendor: String,
    bios_version: String,
    board_name: String,
    board_vendor: String,
    board_version: String,
    product_name: String,
    product_sku: String,
    product_version: String,
    sys_vendor: String,
}

fn dmi_info() -> DmiInfo {
    let dmi_path = |name: &str| format!("/sys/class/dmi/id/{name}");
    let cat_dmi = |name: &str| run_command_string("cat", &[&dmi_path(name)]);
    DmiInfo {
        bios_date: cat_dmi("bios_date"),
        bios_vendor: cat_dmi("bios_vendor"),
        bios_version: cat_dmi("bios_version"),
        board_name: cat_dmi("board_name"),
        board_vendor: cat_dmi("board_vendor"),
        board_version: cat_dmi("board_version"),
        product_name: cat_dmi("product_name"),
        product_sku: cat_dmi("product_sku"),
        product_version: cat_dmi("product_version"),
        sys_vendor: cat_dmi("sys_vendor"),
    }
}

#[derive(Serialize)]
struct CpuInfo {
    architecture: String,
    vendor_id: String,
    model_name: String,
    cpus: String,
    cores_per_socket: String,
    threads_per_core: String,
    sockets: String,
    cpu_max_mhz: String,
    cpu_min_mhz: String,
    virtualization: String,
    l1d_cache: String,
    l1i_cache: String,
    l2_cache: String,
    l3_cache: String,
}

fn cpu_info() -> CpuInfo {
    CpuInfo {
        architecture: command_colon_field("lscpu", &[], "Architecture"),
        vendor_id: command_colon_field("lscpu", &[], "Vendor ID"),
        model_name: command_colon_field("lscpu", &[], "Model name"),
        cpus: command_colon_field("lscpu", &[], "CPU(s)"),
        cores_per_socket: command_colon_field("lscpu", &[], "Core(s) per socket"),
        threads_per_core: command_colon_field("lscpu", &[], "Thread(s) per core"),
        sockets: command_colon_field("lscpu", &[], "Socket(s)"),
        cpu_max_mhz: command_colon_field("lscpu", &[], "CPU max MHz"),
        cpu_min_mhz: command_colon_field("lscpu", &[], "CPU min MHz"),
        virtualization: command_colon_field("lscpu", &[], "Virtualization"),
        l1d_cache: command_colon_field("lscpu", &[], "L1d cache"),
        l1i_cache: command_colon_field("lscpu", &[], "L1i cache"),
        l2_cache: command_colon_field("lscpu", &[], "L2 cache"),
        l3_cache: command_colon_field("lscpu", &[], "L3 cache"),
    }
}

#[derive(Serialize)]
struct PciBusInfo {
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

fn pci_info() -> Vec<PciBusInfo> {
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

#[derive(Serialize)]
struct SystemInfo {
    uname: UnameInfo,
    dmi: DmiInfo,
    cpu: CpuInfo,
    lspci: Vec<PciBusInfo>,
}

fn main() {
    let mut pretty = false;
    let mut indent = 4usize;
    for arg in env::args().skip(1) {
        if arg == "--pretty" {
            pretty = true;
        } else if let Some(value) = arg.strip_prefix("--indent=") {
            if let Ok(parsed) = value.parse::<usize>() {
                indent = parsed;
            }
            pretty = true;
        }
    }

    let info = SystemInfo {
        uname: uname_info(),
        dmi: dmi_info(),
        cpu: cpu_info(),
        lspci: pci_info(),
    };
    if pretty {
        let mut out = Vec::new();
        let indent_bytes = vec![b' '; indent];
        let formatter = PrettyFormatter::with_indent(&indent_bytes);
        let mut serializer = Serializer::with_formatter(&mut out, formatter);
        info.serialize(&mut serializer)
            .expect("failed to serialize JSON");
        let json = String::from_utf8(out).expect("non-utf8 JSON output");
        println!("{json}");
    } else {
        let json = serde_json::to_string(&info).expect("failed to serialize JSON");
        println!("{json}");
    }
}
