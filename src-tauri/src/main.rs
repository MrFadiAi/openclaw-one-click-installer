// Prevent Windows system from displaying console window
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod models;
mod utils;

use commands::{config, diagnostics, installer, process, service, skills};
use utils::log_sanitizer;
use std::io::Write;

fn main() {
    // Initialize logging - show info level logs by default
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format(|buf, record| {
        let sanitized = log_sanitizer::sanitize(&record.args().to_string());
        writeln!(buf, "{} [{}] {}", record.level(), record.target(), sanitized)
    })
    .init();
    
    log::info!("🦞 OpenClaw Manager started");

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            // Service management
            service::get_service_status,
            service::start_service,
            service::stop_service,
            service::restart_service,
            service::get_logs,
            service::kill_all_port_processes,
            // Process management
            process::check_openclaw_installed,
            process::get_openclaw_version,
            process::check_secure_version,
            process::check_port_in_use,
            // Configuration management
            config::get_config,
            config::save_config,
            config::get_env_value,
            config::save_env_value,
            config::get_ai_providers,
            config::get_channels_config,
            config::save_channel_config,
            config::clear_channel_config,
            // Gateway Token
            config::get_or_create_gateway_token,
            config::get_dashboard_url,
            config::repair_device_token,
            // AI configuration management
            config::get_official_providers,
            config::get_ai_config,
            config::save_provider,
            config::delete_provider,
            config::set_primary_model,
            config::add_available_model,
            config::remove_available_model,
            // Feishu plugin management
            config::check_feishu_plugin,
            config::install_feishu_plugin,
            // MCP management
            config::get_mcp_config,
            config::save_mcp_config,
            config::install_mcp_from_git,
            config::uninstall_mcp,
            config::check_mcporter_installed,
            config::install_mcporter,
            config::uninstall_mcporter,
            config::install_mcp_plugin,
            config::openclaw_config_set,
            config::test_mcp_server,
            // Diagnostic tests
            diagnostics::run_doctor,
            diagnostics::test_ai_connection,
            diagnostics::test_channel,
            diagnostics::get_system_info,
            diagnostics::start_channel_login,
            // Installer
            installer::check_environment,
            installer::install_nodejs,
            installer::install_openclaw,
            installer::init_openclaw_config,
            installer::open_install_terminal,
            installer::uninstall_openclaw,
            installer::install_gateway_service,
            // Version update
            installer::check_openclaw_update,
            installer::update_openclaw,
            // Skills management
            skills::get_skills,
            skills::check_clawhub_installed,
            skills::install_clawhub,
            skills::install_skill,
            skills::uninstall_skill,
            skills::uninstall_skill,
            skills::uninstall_clawhub,
            // Multi-Agent Routing
            config::get_openclaw_home_dir,
            config::get_agents_config,
            config::save_agent,
            config::save_subagent_defaults,
            config::delete_agent,
            config::save_agent_binding,
            config::delete_agent_binding,
            config::get_agent_system_prompt,
            config::save_agent_system_prompt,
            config::test_agent_routing,
            // Telegram Multi-Account
            config::get_telegram_accounts,
            config::save_telegram_account,
            config::delete_telegram_account,
            // Heartbeat & Compaction
            config::get_heartbeat_config,
            config::save_heartbeat_config,
            config::get_compaction_config,
            config::save_compaction_config,
            // Workspace & Personality
            config::get_workspace_config,
            config::save_workspace_config,
            config::get_personality_file,
            config::save_personality_file,
            // Browser Control
            config::get_browser_config,
            config::save_browser_config,
            // Web Search
            config::get_web_config,
            config::save_web_config,
            // Gateway Configuration
            config::get_gateway_config,
            config::save_gateway_config,
            // Configuration Management
            config::export_config,
            config::import_config,
        ])
        .run(tauri::generate_context!())
        .expect("Error occurred while running Tauri application");
}
