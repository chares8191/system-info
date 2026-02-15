use serde::Serialize;
use serde_json::ser::{PrettyFormatter, Serializer};
use std::env;

mod cpu_info;
mod dmi_info;
mod env_info;
mod lsblk_info;
mod lsmod_info;
mod mkinitcpio_info;
mod pacman_info;
mod pci_info;
mod proc_info;
mod uname_info;
mod user_passwd_info;
mod utils;
mod x11_info;
mod xdg_info;

use crate::cpu_info::{cpu_info, CpuInfo};
use crate::dmi_info::{dmi_info, DmiInfo};
use crate::env_info::{env_info, EnvInfo};
use crate::lsblk_info::{lsblk_info, BlockDeviceInfo};
use crate::lsmod_info::{lsmod_info, KernelModuleInfo};
use crate::mkinitcpio_info::{mkinitcpio_info, MkinitcpioInfo};
use crate::pacman_info::{pacman_info, PacmanInfo};
use crate::pci_info::{pci_info, PciBusInfo};
use crate::proc_info::{proc_info, ProcInfo};
use crate::uname_info::{uname_info, UnameInfo};
use crate::user_passwd_info::{user_passwd_info, UserPasswdInfo};
use crate::x11_info::{x11_info, X11Info};
use crate::xdg_info::{xdg_info, XdgInfo};

#[derive(Serialize)]
struct SystemInfo {
    uname: UnameInfo,
    user: UserPasswdInfo,
    env: EnvInfo,
    dmi: DmiInfo,
    xdg: XdgInfo,
    cpu: CpuInfo,
    proc: ProcInfo,
    mkinitcpio: MkinitcpioInfo,
    x11: X11Info,
    pacman: PacmanInfo,
    lsblk: Vec<BlockDeviceInfo>,
    lspci: Vec<PciBusInfo>,
    lsmod: Vec<KernelModuleInfo>,
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
        user: user_passwd_info(),
        env: env_info(),
        dmi: dmi_info(),
        xdg: xdg_info(),
        cpu: cpu_info(),
        proc: proc_info(),
        mkinitcpio: mkinitcpio_info(),
        x11: x11_info(),
        pacman: pacman_info(),
        lsblk: lsblk_info(),
        lspci: pci_info(),
        lsmod: lsmod_info(),
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
