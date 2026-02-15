use serde::Serialize;

use crate::utils::{current_username, run_command_string};

#[derive(Serialize)]
pub struct UserPasswdInfo {
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

pub fn user_passwd_info() -> UserPasswdInfo {
    let user = current_username();
    let line = run_command_string("getent", &["passwd", &user]);
    parse_passwd_line(&line)
}
