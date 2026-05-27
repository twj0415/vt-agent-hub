use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub const STORAGE_ROOT_OVERRIDE_ENV: &str = "VT_HUB_MANAGER_STORAGE_ROOT";
pub const CODEX_ROOT_OVERRIDE_ENV: &str = "VT_HUB_MANAGER_CODEX_ROOT";
pub const CLAUDE_ROOT_OVERRIDE_ENV: &str = "VT_HUB_MANAGER_CLAUDE_ROOT";
pub const CURSOR_ROOT_OVERRIDE_ENV: &str = "VT_HUB_MANAGER_CURSOR_ROOT";

pub fn app_storage_root(data_dir_name: &str) -> Result<PathBuf, String> {
    if let Some(root) = env_path(STORAGE_ROOT_OVERRIDE_ENV) {
        return Ok(root);
    }

    Ok(user_home_dir()?.join(data_dir_name))
}

pub fn codex_root() -> PathBuf {
    tool_root(CODEX_ROOT_OVERRIDE_ENV, ".codex")
}

pub fn claude_root() -> PathBuf {
    tool_root(CLAUDE_ROOT_OVERRIDE_ENV, ".claude")
}

pub fn cursor_root() -> PathBuf {
    tool_root(CURSOR_ROOT_OVERRIDE_ENV, ".cursor")
}

pub fn user_home_dir() -> Result<PathBuf, String> {
    if let Some(home) = env_path("USERPROFILE") {
        return Ok(home);
    }

    if cfg!(windows) {
        if let (Some(drive), Some(path)) = (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
            let mut home = OsString::from(drive);
            home.push(path);
            if !home.as_os_str().is_empty() {
                return Ok(PathBuf::from(home));
            }
        }
    }

    if let Some(home) = env_path("HOME") {
        return Ok(home);
    }

    Err("Unable to resolve the current user home directory from USERPROFILE, HOMEDRIVE/HOMEPATH, or HOME.".to_string())
}

fn tool_root(override_env: &str, default_dir: &str) -> PathBuf {
    env_path(override_env).unwrap_or_else(|| user_home_dir_or_current().join(default_dir))
}

fn user_home_dir_or_current() -> PathBuf {
    user_home_dir()
        .or_else(|_| env::current_dir().map_err(|error| error.to_string()))
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.as_os_str().is_empty())
        .map(PathBuf::from)
}
