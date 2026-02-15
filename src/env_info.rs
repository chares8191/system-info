use serde::Serialize;

use crate::utils::env_var_opt;

#[derive(Serialize)]
pub struct EnvInfo {
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

pub fn env_info() -> EnvInfo {
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
