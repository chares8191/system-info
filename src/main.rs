use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use std::env;
use std::process::Command;

fn run_command_string(cmd: &str, args: &[&str]) -> String {
    let output = match Command::new(cmd).args(args).output() {
        Ok(output) => output,
        Err(_) => return String::new(),
    };
    String::from_utf8_lossy(&output.stdout).trim_end().to_string()
}

fn run_command_optional(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
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

fn trim_tree_prefix(value: &str) -> &str {
    value.trim_start_matches(|c: char| {
        c == '⎡' || c == '⎜' || c == '⎣' || c == '↳' || c.is_whitespace()
    })
}

fn current_username() -> String {
    env::var("USER").unwrap_or_else(|_| run_command_string("id", &["-un"]))
}

fn env_var_opt(name: &str) -> Option<String> {
    env::var(name).ok()
}

#[derive(Serialize)]
struct UserPasswdInfo {
    username: String,
    uid: String,
    gid: String,
    home_directory: String,
    login_shell: String,
}

fn parse_passwd_line(line: &str) -> UserPasswdInfo {
    let mut parts = line.split(':');
    let username = parts.next().unwrap_or_default().to_string();
    let _password = parts.next().unwrap_or_default();
    let uid = parts.next().unwrap_or_default().to_string();
    let gid = parts.next().unwrap_or_default().to_string();
    let _gecos = parts.next().unwrap_or_default();
    let home_directory = parts.next().unwrap_or_default().to_string();
    let login_shell = parts.next().unwrap_or_default().to_string();
    UserPasswdInfo {
        username,
        uid,
        gid,
        home_directory,
        login_shell,
    }
}

fn user_passwd_info() -> UserPasswdInfo {
    let user = current_username();
    let line = run_command_string("getent", &["passwd", &user]);
    parse_passwd_line(&line)
}

#[derive(Serialize)]
struct XdgInfo {
    xdg_cache_home: Option<String>,
    xdg_config_home: Option<String>,
    xdg_data_home: Option<String>,
    xdg_runtime_dir: Option<String>,
    xdg_seat: Option<String>,
    xdg_session_class: Option<String>,
    xdg_session_id: Option<String>,
    xdg_session_type: Option<String>,
    xdg_state_home: Option<String>,
    xdg_vtnr: Option<String>,
}

fn xdg_info() -> XdgInfo {
    XdgInfo {
        xdg_cache_home: env_var_opt("XDG_CACHE_HOME"),
        xdg_config_home: env_var_opt("XDG_CONFIG_HOME"),
        xdg_data_home: env_var_opt("XDG_DATA_HOME"),
        xdg_runtime_dir: env_var_opt("XDG_RUNTIME_DIR"),
        xdg_seat: env_var_opt("XDG_SEAT"),
        xdg_session_class: env_var_opt("XDG_SESSION_CLASS"),
        xdg_session_id: env_var_opt("XDG_SESSION_ID"),
        xdg_session_type: env_var_opt("XDG_SESSION_TYPE"),
        xdg_state_home: env_var_opt("XDG_STATE_HOME"),
        xdg_vtnr: env_var_opt("XDG_VTNR"),
    }
}

#[derive(Serialize)]
struct EnvInfo {
    user: Option<String>,
    logname: Option<String>,
    home: Option<String>,
    shell: Option<String>,
    path: Option<String>,
    lang: Option<String>,
    lc_all: Option<String>,
    lc_ctype: Option<String>,
    term: Option<String>,
}

fn env_info() -> EnvInfo {
    EnvInfo {
        user: env_var_opt("USER"),
        logname: env_var_opt("LOGNAME"),
        home: env_var_opt("HOME"),
        shell: env_var_opt("SHELL"),
        path: env_var_opt("PATH"),
        lang: env_var_opt("LANG"),
        lc_all: env_var_opt("LC_ALL"),
        lc_ctype: env_var_opt("LC_CTYPE"),
        term: env_var_opt("TERM"),
    }
}

#[derive(Serialize)]
struct XinputDeviceInfo {
    name: String,
    id: String,
    role: Option<String>,
    device_type: Option<String>,
    attached_to: Option<String>,
}

fn parse_xinput_bracket(value: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut role = None;
    let mut device_type = None;
    let mut attached_to = None;

    let mut content = value.trim();
    if let Some(lb) = content.rfind('(') {
        if let Some(rb) = content[lb + 1..].find(')') {
            attached_to = Some(content[lb + 1..lb + 1 + rb].trim().to_string());
            content = content[..lb].trim_end();
        }
    }

    let mut parts = content.split_whitespace();
    if let Some(first) = parts.next() {
        role = Some(first.to_string());
    }
    if let Some(second) = parts.next() {
        device_type = Some(second.to_string());
    }

    (role, device_type, attached_to)
}

fn parse_xinput_list(output: &str) -> Vec<XinputDeviceInfo> {
    let mut devices = Vec::new();
    for line in output.lines() {
        if !line.contains("id=") {
            continue;
        }
        let (left, right) = match line.split_once("id=") {
            Some(parts) => parts,
            None => continue,
        };
        let name = trim_tree_prefix(left).trim().to_string();
        let mut right_parts = right.split_whitespace();
        let id = match right_parts.next() {
            Some(value) => value.to_string(),
            None => continue,
        };

        let mut role = None;
        let mut device_type = None;
        let mut attached_to = None;
        if let Some(lb) = line.find('[') {
            if let Some(rb) = line[lb + 1..].find(']') {
                let bracket = &line[lb + 1..lb + 1 + rb];
                let parsed = parse_xinput_bracket(bracket);
                role = parsed.0;
                device_type = parsed.1;
                attached_to = parsed.2;
            }
        }

        devices.push(XinputDeviceInfo {
            name,
            id,
            role,
            device_type,
            attached_to,
        });
    }
    devices
}

#[derive(Serialize)]
struct XinputInfo {
    devices: Option<Vec<XinputDeviceInfo>>,
}

fn xinput_info() -> XinputInfo {
    let devices = run_command_optional("xinput", &["list"]).map(|out| parse_xinput_list(&out));
    XinputInfo { devices }
}

#[derive(Serialize)]
struct XrandrMonitorInfo {
    index: u32,
    name: String,
    geometry: String,
}

#[derive(Serialize)]
struct XrandrInfo {
    monitors: Option<Vec<XrandrMonitorInfo>>,
}

fn parse_xrandr_listmonitors(output: &str) -> Vec<XrandrMonitorInfo> {
    let mut monitors = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Monitors:") {
            continue;
        }
        let (index_part, rest) = match line.split_once(':') {
            Some(parts) => parts,
            None => continue,
        };
        let index = match index_part.trim().parse::<u32>() {
            Ok(value) => value,
            Err(_) => continue,
        };
        let mut parts: Vec<&str> = rest.trim().split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts.pop().unwrap().to_string();
        let geometry = parts.join(" ");
        monitors.push(XrandrMonitorInfo {
            index,
            name,
            geometry,
        });
    }
    monitors
}

fn xrandr_info() -> XrandrInfo {
    let monitors = run_command_optional("xrandr", &["--listmonitors"])
        .map(|out| parse_xrandr_listmonitors(&out));
    XrandrInfo { monitors }
}

#[derive(Serialize)]
struct X11Info {
    xinput: XinputInfo,
    xrandr: XrandrInfo,
}

fn x11_info() -> X11Info {
    X11Info {
        xinput: xinput_info(),
        xrandr: xrandr_info(),
    }
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
struct ProcInfo {
    cmdline: String,
    meminfo: ProcMemInfo,
    partitions: Vec<ProcPartitionInfo>,
    version: String,
}

#[derive(Serialize)]
struct ProcMemInfo {
    mem_total: String,
}

#[derive(Serialize)]
struct ProcPartitionInfo {
    major: String,
    minor: String,
    blocks: String,
    name: String,
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

fn parse_proc_partitions(value: &str) -> Vec<ProcPartitionInfo> {
    let mut partitions = Vec::new();
    for line in value.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("major") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (major, minor, blocks, name) =
            match (parts.next(), parts.next(), parts.next(), parts.next()) {
                (Some(major), Some(minor), Some(blocks), Some(name)) => {
                    (major, minor, blocks, name)
                }
                _ => continue,
            };
        partitions.push(ProcPartitionInfo {
            major: major.to_string(),
            minor: minor.to_string(),
            blocks: blocks.to_string(),
            name: name.to_string(),
        });
    }
    partitions
}

fn proc_info() -> ProcInfo {
    let proc_path = |name: &str| format!("/proc/{name}");
    let cat_proc = |name: &str| run_command_string("cat", &[&proc_path(name)]);
    ProcInfo {
        cmdline: cat_proc("cmdline"),
        meminfo: parse_proc_meminfo(&cat_proc("meminfo")),
        partitions: parse_proc_partitions(&cat_proc("partitions")),
        version: cat_proc("version"),
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
    user: UserPasswdInfo,
    xdg: XdgInfo,
    env: EnvInfo,
    x11: X11Info,
    uname: UnameInfo,
    dmi: DmiInfo,
    proc: ProcInfo,
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
        user: user_passwd_info(),
        xdg: xdg_info(),
        env: env_info(),
        x11: x11_info(),
        uname: uname_info(),
        dmi: dmi_info(),
        proc: proc_info(),
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
