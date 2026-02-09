use serde::Serialize;
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

fn kernel_release() -> String {
    run_command_string("uname", &["-r"])
}

fn machine_arch() -> String {
    run_command_string("uname", &["-m"])
}

#[derive(Serialize)]
struct SystemInfo {
    kernel_release: String,
    machine_arch: String,
}

fn main() {
    let info = SystemInfo {
        kernel_release: kernel_release(),
        machine_arch: machine_arch(),
    };
    let json = serde_json::to_string(&info).expect("failed to serialize JSON");
    println!("{json}");
}
