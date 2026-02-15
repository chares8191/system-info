use serde::Serialize;

use crate::utils::run_command_string;

#[derive(Serialize)]
pub struct UnameInfo {
    kernel_release: String,
    machine: String,
}

pub fn uname_info() -> UnameInfo {
    let kernel_release = || run_command_string("uname", &["-r"]);
    let machine = || run_command_string("uname", &["-m"]);
    UnameInfo {
        kernel_release: kernel_release(),
        machine: machine(),
    }
}
