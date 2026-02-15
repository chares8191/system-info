use serde::Serialize;

use crate::utils::{run_command_optional, trim_tree_prefix};

#[derive(Serialize)]
pub struct XinputDeviceInfo {
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
pub struct XinputInfo {
    devices: Option<Vec<XinputDeviceInfo>>,
}

fn xinput_info() -> XinputInfo {
    let devices = run_command_optional("xinput", &["list"]).map(|out| parse_xinput_list(&out));
    XinputInfo { devices }
}

#[derive(Serialize)]
pub struct XrandrMonitorInfo {
    index: u32,
    name: String,
    geometry: String,
}

#[derive(Serialize)]
pub struct XrandrInfo {
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
pub struct XdpyInfo {
    dimensions: Option<String>,
    resolution: Option<String>,
}

fn xdpy_info() -> XdpyInfo {
    let output = run_command_optional("xdpyinfo", &[]).unwrap_or_default();
    let mut dimensions = None;
    let mut resolution = None;
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("dimensions:") {
            let value = rest.trim();
            if !value.is_empty() {
                dimensions = Some(value.to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("resolution:") {
            let value = rest.trim();
            if !value.is_empty() {
                resolution = Some(value.to_string());
            }
        }
    }
    XdpyInfo {
        dimensions,
        resolution,
    }
}

#[derive(Serialize)]
pub struct XrdbInfo {
    resources: Vec<String>,
}

fn xrdb_info() -> XrdbInfo {
    let resources = run_command_optional("xrdb", &["-query"])
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    XrdbInfo { resources }
}

#[derive(Serialize)]
pub struct X11Info {
    xinput: XinputInfo,
    xrandr: XrandrInfo,
    xrdb: XrdbInfo,
    xdpyinfo: XdpyInfo,
}

pub fn x11_info() -> X11Info {
    X11Info {
        xinput: xinput_info(),
        xrandr: xrandr_info(),
        xrdb: xrdb_info(),
        xdpyinfo: xdpy_info(),
    }
}
