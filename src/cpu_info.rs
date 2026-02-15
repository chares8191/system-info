use serde::Serialize;

use crate::utils::command_colon_field;

#[derive(Serialize)]
pub struct CpuInfo {
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

pub fn cpu_info() -> CpuInfo {
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
