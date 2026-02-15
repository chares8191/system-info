use serde::Serialize;
use serde_json::Value;

use crate::utils::run_command_optional;

#[derive(Serialize)]
pub struct BlockDeviceInfo {
    name: String,
    path: String,
    maj_min: String,
    rm: String,
    size: String,
    ro: String,
    dev_type: String,
    fsroots: String,
    fstype: String,
    fsver: String,
    label: String,
    uuid: String,
    fsused: String,
    fssize: String,
    pkname: String,
    partuuid: String,
    parttype: String,
    parttypename: String,
    pttype: String,
    ptuuid: String,
    mountpoints: String,
}

pub fn lsblk_info() -> Vec<BlockDeviceInfo> {
    let output = match run_command_optional(
        "lsblk",
        &[
            "--json",
            "--list",
            "-o",
            "NAME,PATH,MAJ:MIN,RM,SIZE,RO,TYPE,MOUNTPOINTS,FSROOTS,FSTYPE,FSVER,LABEL,UUID,FSUSED,FSSIZE,PKNAME,PARTUUID,PARTTYPE,PARTTYPENAME,PTTYPE,PTUUID",
        ],
    ) {
        Some(output) => output,
        None => return Vec::new(),
    };
    let parsed: Value = match serde_json::from_str(&output) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let devices = match parsed.get("blockdevices").and_then(|v| v.as_array()) {
        Some(list) => list,
        None => return Vec::new(),
    };
    let value_to_string = |value: Option<&Value>| -> String {
        match value {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Array(items)) => items
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            _ => String::new(),
        }
    };
    let mut out = Vec::new();
    for dev in devices {
        let obj = match dev.as_object() {
            Some(obj) => obj,
            None => continue,
        };
        out.push(BlockDeviceInfo {
            name: value_to_string(obj.get("name")),
            path: value_to_string(obj.get("path")),
            maj_min: value_to_string(obj.get("maj:min")),
            rm: value_to_string(obj.get("rm")),
            size: value_to_string(obj.get("size")),
            ro: value_to_string(obj.get("ro")),
            dev_type: value_to_string(obj.get("type")),
            fsroots: value_to_string(obj.get("fsroots")),
            fstype: value_to_string(obj.get("fstype")),
            fsver: value_to_string(obj.get("fsver")),
            label: value_to_string(obj.get("label")),
            uuid: value_to_string(obj.get("uuid")),
            fsused: value_to_string(obj.get("fsused")),
            fssize: value_to_string(obj.get("fssize")),
            pkname: value_to_string(obj.get("pkname")),
            partuuid: value_to_string(obj.get("partuuid")),
            parttype: value_to_string(obj.get("parttype")),
            parttypename: value_to_string(obj.get("parttypename")),
            pttype: value_to_string(obj.get("pttype")),
            ptuuid: value_to_string(obj.get("ptuuid")),
            mountpoints: value_to_string(obj.get("mountpoints")),
        });
    }
    out
}
