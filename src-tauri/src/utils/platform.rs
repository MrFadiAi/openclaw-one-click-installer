use std::env;

/// 获取操作系统类型
pub fn get_os() -> String {
    env::consts::OS.to_string()
}

/// 获取系统架构
pub fn get_arch() -> String {
    env::consts::ARCH.to_string()
}

/// 获取配置目录路径
pub fn get_config_dir() -> String {
    if let Some(home) = dirs::home_dir() {
        if is_windows() {
            format!("{}\\.openclaw", home.display())
        } else {
            format!("{}/.openclaw", home.display())
        }
    } else {
        String::from("~/.openclaw")
    }
}

/// 获取环境变量文件路径
pub fn get_env_file_path() -> String {
    if is_windows() {
        format!("{}\\env", get_config_dir())
    } else {
        format!("{}/env", get_config_dir())
    }
}

/// 获取 openclaw.json 配置文件路径
pub fn get_config_file_path() -> String {
    if is_windows() {
        format!("{}\\openclaw.json", get_config_dir())
    } else {
        format!("{}/openclaw.json", get_config_dir())
    }
}

/// Get manager configuration file path (manager.json)
pub fn get_manager_config_file_path() -> String {
    if is_windows() {
        format!("{}\\manager.json", get_config_dir())
    } else {
        format!("{}/manager.json", get_config_dir())
    }
}

/// 获取日志文件路径
pub fn get_log_file_path() -> String {
    if is_windows() {
        format!("{}\\openclaw-gateway.log", get_config_dir())
    } else {
        String::from("/tmp/openclaw-gateway.log")
    }
}

/// Get MCP servers install directory
pub fn get_mcp_install_dir() -> String {
    if is_windows() {
        format!("{}\\mcps", get_config_dir())
    } else {
        format!("{}/mcps", get_config_dir())
    }
}

/// Get MCP configuration file path (separate from openclaw.json)
pub fn get_mcp_config_file_path() -> String {
    if is_windows() {
        format!("{}\\mcps.json", get_config_dir())
    } else {
        format!("{}/mcps.json", get_config_dir())
    }
}

/// Get mcporter configuration file path (~/.mcporter/mcporter.json)
pub fn get_mcporter_config_file_path() -> String {
    if let Some(home) = dirs::home_dir() {
        if is_windows() {
            format!("{}\\.mcporter\\mcporter.json", home.display())
        } else {
            format!("{}/.mcporter/mcporter.json", home.display())
        }
    } else {
        String::from("~/.mcporter/mcporter.json")
    }
}

/// 检测当前平台是否为 macOS
pub fn is_macos() -> bool {
    env::consts::OS == "macos"
}

/// 检测当前平台是否为 Windows
pub fn is_windows() -> bool {
    env::consts::OS == "windows"
}

/// 检测当前平台是否为 Linux
pub fn is_linux() -> bool {
    env::consts::OS == "linux"
}
