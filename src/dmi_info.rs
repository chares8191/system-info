use serde::Serialize;

use crate::utils::run_command_string;

#[derive(Serialize)]
pub struct DmiInfo {
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

pub fn dmi_info() -> DmiInfo {
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
