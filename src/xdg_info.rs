use serde::Serialize;

use crate::utils::env_var_opt;

#[derive(Serialize)]
pub struct XdgInfo {
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

pub fn xdg_info() -> XdgInfo {
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
