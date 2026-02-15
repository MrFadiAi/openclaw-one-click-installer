use crate::models::{AITestResult, ChannelTestResult, DiagnosticResult, SystemInfo};
use crate::utils::{platform, shell};
use tauri::command;
use log::{info, warn, debug};

/// Strip ANSI escape sequences (color codes, etc.)
fn strip_ansi_codes(input: &str) -> String {
    // Match ANSI escape sequences: ESC[ ... m or ESC[ ... other control characters
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ESC[...m sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // Skip '['
                // Skip until alphabetic character
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Extract JSON content from mixed output
fn extract_json_from_output(output: &str) -> Option<String> {
    // First strip ANSI color codes
    let clean_output = strip_ansi_codes(output);

    // Find JSON start position line by line
    let lines: Vec<&str> = clean_output.lines().collect();
    let mut json_start_line = None;
    let mut json_end_line = None;

    // Find JSON start line:
    // - Starts with { (JSON object)
    // - Or starts with [" or [digit (real JSON array, not text like [plugins])
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            json_start_line = Some(i);
            break;
        }
        // Check if it's a real JSON array (starts with [" or [digit or [{)
        if trimmed.starts_with('[') && trimmed.len() > 1 {
            let second_char = trimmed.chars().nth(1).unwrap_or(' ');
            if second_char == '"' || second_char == '{' || second_char == '[' || second_char.is_ascii_digit() {
                json_start_line = Some(i);
                break;
            }
        }
    }
    
    // Find JSON end line (line ending with } or ], search from the end)
    for (i, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if trimmed == "}" || trimmed == "}," || trimmed.ends_with('}') {
            json_end_line = Some(i);
            break;
        }
        if trimmed == "]" || trimmed == "]," {
            json_end_line = Some(i);
            break;
        }
    }
    
    match (json_start_line, json_end_line) {
        (Some(start), Some(end)) if start <= end => {
            let json_lines: Vec<&str> = lines[start..=end].to_vec();
            let json_str = json_lines.join("\n");
            Some(json_str)
        }
        _ => None,
    }
}

/// Run diagnostics
#[command]
pub async fn run_doctor() -> Result<Vec<DiagnosticResult>, String> {
    info!("[Diagnostics] Starting system diagnostics...");
    let mut results = Vec::new();

    // Check if OpenClaw is installed
    info!("[Diagnostics] Checking OpenClaw installation status...");
    let openclaw_installed = shell::get_openclaw_path().is_some();
    info!("[Diagnostics] OpenClaw installed: {}", if openclaw_installed { "✓" } else { "✗" });
    results.push(DiagnosticResult {
        name: "OpenClaw Installation".to_string(),
        passed: openclaw_installed,
        message: if openclaw_installed {
            "OpenClaw is installed".to_string()
        } else {
            "OpenClaw is not installed".to_string()
        },
        suggestion: if openclaw_installed {
            None
        } else {
            Some("Run: npm install -g openclaw".to_string())
        },
    });

    // Check Node.js
    let node_check = shell::run_command_output("node", &["--version"]);
    results.push(DiagnosticResult {
        name: "Node.js".to_string(),
        passed: node_check.is_ok(),
        message: node_check
            .clone()
            .unwrap_or_else(|_| "Not installed".to_string()),
        suggestion: if node_check.is_err() {
            Some("Please install Node.js 22+".to_string())
        } else {
            None
        },
    });

    // Check config file
    let config_path = platform::get_config_file_path();
    let config_exists = std::path::Path::new(&config_path).exists();
    results.push(DiagnosticResult {
        name: "Config File".to_string(),
        passed: config_exists,
        message: if config_exists {
            format!("Config file exists: {}", config_path)
        } else {
            "Config file does not exist".to_string()
        },
        suggestion: if config_exists {
            None
        } else {
            Some("Run openclaw to initialize config".to_string())
        },
    });

    // Check environment variables file
    let env_path = platform::get_env_file_path();
    let env_exists = std::path::Path::new(&env_path).exists();
    results.push(DiagnosticResult {
        name: "Environment Variables".to_string(),
        passed: env_exists,
        message: if env_exists {
            format!("Environment file exists: {}", env_path)
        } else {
            "Environment file does not exist".to_string()
        },
        suggestion: if env_exists {
            None
        } else {
            Some("Please configure AI API Key".to_string())
        },
    });

    // Run openclaw doctor
    if openclaw_installed {
        let doctor_result = shell::run_openclaw(&["doctor"]);
        results.push(DiagnosticResult {
            name: "OpenClaw Doctor".to_string(),
            passed: doctor_result.is_ok() && !doctor_result.as_ref().unwrap().contains("invalid"),
            message: doctor_result.unwrap_or_else(|e| e),
            suggestion: None,
        });
    }
    
    Ok(results)
}

/// Test AI connection
#[command]
pub async fn test_ai_connection() -> Result<AITestResult, String> {
    info!("[AI Test] Starting AI connection test...");

    // Get current configured provider
    let start = std::time::Instant::now();

    // Use openclaw command to test connection
    info!("[AI Test] Executing: openclaw agent --local --to +1234567890 --message \"Reply OK\"");
    let result = shell::run_openclaw(&["agent", "--local", "--to", "+1234567890", "--message", "Reply OK"]);

    let latency = start.elapsed().as_millis() as u64;
    info!("[AI Test] Command execution completed, latency: {}ms", latency);

    match result {
        Ok(output) => {
            debug!("[AI Test] Raw output: {}", output);
            // Filter out warning messages
            let filtered: String = output
                .lines()
                .filter(|l: &&str| !l.contains("ExperimentalWarning"))
                .collect::<Vec<&str>>()
                .join("\n");
            
            let success = !filtered.to_lowercase().contains("error")
                && !filtered.contains("401")
                && !filtered.contains("403");
            
            if success {
                info!("[AI Test] ✓ AI connection test successful");
            } else {
                warn!("[AI Test] ✗ AI connection test failed: {}", filtered);
            }
            
            Ok(AITestResult {
                success,
                provider: "current".to_string(),
                model: "default".to_string(),
                response: if success { Some(filtered.clone()) } else { None },
                error: if success { None } else { Some(filtered) },
                latency_ms: Some(latency),
            })
        }
        Err(e) => Ok(AITestResult {
            success: false,
            provider: "current".to_string(),
            model: "default".to_string(),
            response: None,
            error: Some(e),
            latency_ms: Some(latency),
        }),
    }
}

/// Get channel test target
fn get_channel_test_target(channel_type: &str) -> Option<String> {
    let env_path = platform::get_env_file_path();

    // Get environment variable for test target based on channel type
    let env_key = match channel_type.to_lowercase().as_str() {
        "telegram" => "OPENCLAW_TELEGRAM_USERID",
        "discord" => "OPENCLAW_DISCORD_TESTCHANNELID",
        "slack" => "OPENCLAW_SLACK_TESTCHANNELID",
        "feishu" => "OPENCLAW_FEISHU_TESTCHATID",
        // WhatsApp uses QR code login, no test target needed to send messages
        "whatsapp" => return None,
        // iMessage also doesn't need test target
        "imessage" => return None,
        _ => return None,
    };

    crate::utils::file::read_env_value(&env_path, env_key)
}

/// Check if channel needs to send test message
fn channel_needs_send_test(channel_type: &str) -> bool {
    match channel_type.to_lowercase().as_str() {
        // These channels need to send test messages for verification
        "telegram" | "discord" | "slack" | "feishu" => true,
        // WhatsApp and iMessage only check status, don't send test messages
        "whatsapp" | "imessage" => false,
        _ => false,
    }
}

/// Parse channel status from text output
/// Format: "- Telegram default: enabled, configured, mode:polling, token:config"
fn parse_channel_status_text(output: &str, channel_type: &str) -> Option<(bool, bool, bool, String)> {
    let channel_lower = channel_type.to_lowercase();

    for line in output.lines() {
        let line = line.trim();
        // Match "- Telegram default: ..." format
        if line.starts_with("- ") && line.to_lowercase().contains(&channel_lower) {
            // Parse status
            let enabled = line.contains("enabled");
            let configured = line.contains("configured") && !line.contains("not configured");
            let linked = line.contains("linked");

            // Extract status description (part after colon)
            let status_part = line.split(':').skip(1).collect::<Vec<&str>>().join(":");
            let status_msg = status_part.trim().to_string();

            return Some((enabled, configured, linked, status_msg));
        }
    }
    None
}

/// Test channel connection (check status and send test message)
#[command]
pub async fn test_channel(channel_type: String) -> Result<ChannelTestResult, String> {
    info!("[Channel Test] Testing channel: {}", channel_type);
    let channel_lower = channel_type.to_lowercase();

    // Use openclaw channels status to check channel status (no --json as it may not be supported)
    info!("[Channel Test] Step 1: Checking channel status...");
    let status_result = shell::run_openclaw(&["channels", "status"]);

    let mut channel_ok = false;
    let mut status_message = String::new();
    let mut debug_info = String::new();

    match &status_result {
        Ok(output) => {
            info!("[Channel Test] status command executed successfully");

            // Try to parse status from text output
            if let Some((enabled, configured, linked, status_msg)) = parse_channel_status_text(output, &channel_type) {
                debug_info = format!("enabled={}, configured={}, linked={}", enabled, configured, linked);
                info!("[Channel Test] {} status: {}", channel_type, debug_info);

                if !configured {
                    info!("[Channel Test] {} not configured", channel_type);
                    return Ok(ChannelTestResult {
                        success: false,
                        channel: channel_type.clone(),
                        message: format!("{} not configured", channel_type),
                        error: Some(format!("Please run: openclaw channels add --channel {}", channel_lower)),
                    });
                }

                // If configured, consider status OK (Gateway may not be running, but config exists)
                channel_ok = configured;
                status_message = if linked {
                    "Linked".to_string()
                } else if !status_msg.is_empty() {
                    status_msg
                } else {
                    "Configured".to_string()
                };
            } else {
                // Try JSON parsing (as fallback)
                if let Some(json_str) = extract_json_from_output(output) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(channels) = json.get("channels").and_then(|c| c.as_object()) {
                            if let Some(ch) = channels.get(&channel_lower) {
                                let configured = ch.get("configured").and_then(|v| v.as_bool()).unwrap_or(false);
                                let linked = ch.get("linked").and_then(|v| v.as_bool()).unwrap_or(false);
                                channel_ok = configured;
                                status_message = if linked { "Linked".to_string() } else { "Configured".to_string() };
                            }
                        }
                    }
                }

                if !channel_ok {
                    debug_info = format!("Unable to parse {} status", channel_type);
                    info!("[Channel Test] {}", debug_info);
                }
            }
        }
        Err(e) => {
            debug_info = format!("Command execution failed: {}", e);
            info!("[Channel Test] {}", debug_info);
        }
    }

    // If channel status is not OK, return failure directly
    if !channel_ok {
        info!("[Channel Test] {} status check failed, not sending test message", channel_type);
        let error_msg = if debug_info.is_empty() {
            "Channel not running or not configured".to_string()
        } else {
            debug_info
        };
        return Ok(ChannelTestResult {
            success: false,
            channel: channel_type.clone(),
            message: format!("{} not connected", channel_type),
            error: Some(error_msg),
        });
    }

    info!("[Channel Test] {} status OK ({})", channel_type, status_message);

    // For WhatsApp and iMessage, only return status check result, don't send test message
    if !channel_needs_send_test(&channel_type) {
        info!("[Channel Test] {} doesn't need test message (status check only)", channel_type);
        return Ok(ChannelTestResult {
            success: true,
            channel: channel_type.clone(),
            message: format!("{} status OK ({})", channel_type, status_message),
            error: None,
        });
    }

    // Try to send test message
    info!("[Channel Test] Step 2: Getting test target...");
    let test_target = get_channel_test_target(&channel_type);

    if let Some(target) = test_target {
        info!("[Channel Test] Step 3: Sending test message to {}...", target);
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message = format!("🤖 OpenClaw Test Message\n\n✅ Connection successful!\n⏰ {}", timestamp);

        // Use openclaw message send to send test message
        info!("[Channel Test] Executing: openclaw message send --channel {} --target {} ...", channel_lower, target);
        let send_result = shell::run_openclaw(&[
            "message", "send",
            "--channel", &channel_lower,
            "--target", &target,
            "--message", &message,
            "--json"
        ]);

        match send_result {
            Ok(output) => {
                info!("[Channel Test] Send command output length: {}", output.len());

                // Check if send was successful
                let send_ok = if let Some(json_str) = extract_json_from_output(&output) {
                    info!("[Channel Test] Extracted JSON: {}", json_str);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        // Check various success indicators
                        let has_ok = json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                        let has_success = json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                        let has_message_id = json.get("messageId").is_some();
                        let has_payload_ok = json.get("payload").and_then(|p| p.get("ok")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let has_payload_message_id = json.get("payload").and_then(|p| p.get("messageId")).is_some();
                        let has_payload_result_message_id = json.get("payload")
                            .and_then(|p| p.get("result"))
                            .and_then(|r| r.get("messageId"))
                            .is_some();
                        
                        info!("[Channel Test] Condition check: ok={}, success={}, messageId={}, payload.ok={}, payload.messageId={}, payload.result.messageId={}",
                            has_ok, has_success, has_message_id, has_payload_ok, has_payload_message_id, has_payload_result_message_id);

                        has_ok || has_success || has_message_id || has_payload_ok || has_payload_message_id || has_payload_result_message_id
                    } else {
                        info!("[Channel Test] JSON parsing failed");
                        false
                    }
                } else {
                    info!("[Channel Test] No JSON extracted, checking keywords");
                    // If no JSON, check for error keywords
                    !output.to_lowercase().contains("error") && !output.to_lowercase().contains("failed")
                };

                if send_ok {
                    info!("[Channel Test] ✓ {} test message sent successfully", channel_type);
                    Ok(ChannelTestResult {
                        success: true,
                        channel: channel_type.clone(),
                        message: format!("{} test message sent ({})", channel_type, status_message),
                        error: None,
                    })
                } else {
                    info!("[Channel Test] ✗ {} test message send failed", channel_type);
                    Ok(ChannelTestResult {
                        success: false,
                        channel: channel_type.clone(),
                        message: format!("{} message send failed", channel_type),
                        error: Some(output),
                    })
                }
            }
            Err(e) => {
                info!("[Channel Test] ✗ {} send command execution failed: {}", channel_type, e);
                Ok(ChannelTestResult {
                    success: false,
                    channel: channel_type.clone(),
                    message: format!("{} message send failed", channel_type),
                    error: Some(e),
                })
            }
        }
    } else {
        // No test target configured, return status but hint that test target needs to be configured
        let hint = match channel_lower.as_str() {
            "telegram" => "Please configure OPENCLAW_TELEGRAM_USERID",
            "discord" => "Please configure OPENCLAW_DISCORD_TESTCHANNELID",
            "slack" => "Please configure OPENCLAW_SLACK_TESTCHANNELID",
            "feishu" => "Please configure OPENCLAW_FEISHU_TESTCHATID",
            _ => "Please configure test target",
        };

        info!("[Channel Test] {} test target not configured, skipping message send ({})", channel_type, hint);
        Ok(ChannelTestResult {
            success: true,
            channel: channel_type.clone(),
            message: format!("{} status OK ({}) - {}", channel_type, status_message, hint),
            error: None,
        })
    }
}

/// Send test message to channel
#[command]
pub async fn send_test_message(channel_type: String, target: String) -> Result<ChannelTestResult, String> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let message = format!("🤖 OpenClaw Test Message\n\n✅ Connection successful!\n⏰ {}", timestamp);

    // Use openclaw message send command to send test message
    let send_result = shell::run_openclaw(&[
        "message", "send",
        "--channel", &channel_type,
        "--target", &target,
        "--message", &message,
        "--json"
    ]);

    match send_result {
        Ok(output) => {
            // Try to extract and parse JSON result from mixed output
            let success = if let Some(json_str) = extract_json_from_output(&output) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    json.get("success").and_then(|v| v.as_bool()).unwrap_or(false)
                        || json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false)
                        || json.get("messageId").is_some()
                } else {
                    false
                }
            } else {
                // Non-JSON output, check for error keywords
                !output.to_lowercase().contains("error") && !output.to_lowercase().contains("failed")
            };

            Ok(ChannelTestResult {
                success,
                channel: channel_type,
                message: if success { "Message sent".to_string() } else { "Message send failed".to_string() },
                error: if success { None } else { Some(output) },
            })
        }
        Err(e) => Ok(ChannelTestResult {
            success: false,
            channel: channel_type,
            message: "Send failed".to_string(),
            error: Some(e),
        }),
    }
}

/// Get system information
#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    info!("[System Info] Getting system information...");
    let os = platform::get_os();
    let arch = platform::get_arch();
    info!("[System Info] OS: {}, Arch: {}", os, arch);

    // Get OS version
    let os_version = if platform::is_macos() {
        shell::run_command_output("sw_vers", &["-productVersion"])
            .unwrap_or_else(|_| "unknown".to_string())
    } else if platform::is_linux() {
        shell::run_bash_output("cat /etc/os-release | grep VERSION_ID | cut -d'=' -f2 | tr -d '\"'")
            .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    };
    
    let openclaw_installed = shell::get_openclaw_path().is_some();
    let openclaw_version = if openclaw_installed {
        shell::run_openclaw(&["--version"]).ok()
    } else {
        None
    };
    
    let node_version = shell::run_command_output("node", &["--version"]).ok();
    
    Ok(SystemInfo {
        os,
        os_version,
        arch,
        openclaw_installed,
        openclaw_version,
        node_version,
        config_dir: platform::get_config_dir(),
    })
}

/// Start channel login (e.g., WhatsApp QR code scan)
#[command]
pub async fn start_channel_login(channel_type: String) -> Result<String, String> {
    info!("[Channel Login] Starting channel login flow: {}", channel_type);

    match channel_type.as_str() {
        "whatsapp" => {
            info!("[Channel Login] WhatsApp login flow...");
            // First enable plugin in background
            info!("[Channel Login] Enabling whatsapp plugin...");
            let _ = shell::run_openclaw(&["plugins", "enable", "whatsapp"]);

            #[cfg(target_os = "macos")]
            {
                let env_path = platform::get_env_file_path();
                // Create a temporary script file
                // Flow: 1. Enable plugin 2. Restart Gateway 3. Login
                let script_content = format!(
                    r#"#!/bin/bash
source {} 2>/dev/null
clear
echo "╔════════════════════════════════════════════════════════╗"
echo "║           📱 WhatsApp Login Wizard                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "Step 1/3: Enabling WhatsApp plugin..."
openclaw plugins enable whatsapp 2>/dev/null || true

# Ensure whatsapp is in plugins.allow array
python3 << 'PYEOF'
import json
import os

config_path = os.path.expanduser("~/.openclaw/openclaw.json")
plugin_id = "whatsapp"

try:
    with open(config_path, 'r') as f:
        config = json.load(f)

    # Set plugins.allow and plugins.entries
    if 'plugins' not in config:
        config['plugins'] = {{'allow': [], 'entries': {{}}}}
    if 'allow' not in config['plugins']:
        config['plugins']['allow'] = []
    if 'entries' not in config['plugins']:
        config['plugins']['entries'] = {{}}

    if plugin_id not in config['plugins']['allow']:
        config['plugins']['allow'].append(plugin_id)

    config['plugins']['entries'][plugin_id] = {{'enabled': True}}

    # Ensure channels.whatsapp exists (but don't set enabled, WhatsApp doesn't support this key)
    if 'channels' not in config:
        config['channels'] = {{}}
    if plugin_id not in config['channels']:
        config['channels'][plugin_id] = {{'dmPolicy': 'pairing', 'groupPolicy': 'allowlist'}}

    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2, ensure_ascii=False)
    print("Config updated")
except Exception as e:
    print(f"Warning: {{e}}")
PYEOF

echo "✅ Plugin enabled"
echo ""

echo "Step 2/3: Restarting Gateway to apply plugin..."
# Use openclaw command to stop and start gateway
openclaw gateway stop 2>/dev/null || true
sleep 2
# Start gateway service
openclaw gateway start 2>/dev/null || openclaw gateway --port 18789 &
sleep 3
echo "✅ Gateway restarted"
echo ""

echo "Step 3/3: Starting WhatsApp login..."
echo "Please scan the QR code below using WhatsApp mobile app"
echo ""
openclaw channels login --channel whatsapp --verbose
echo ""
echo "════════════════════════════════════════════════════════"
echo "Login complete!"
echo ""
read -p "Press Enter to close this window..."
"#,
                    env_path
                );

                let script_path = "/tmp/openclaw_whatsapp_login.command";
                std::fs::write(script_path, script_content)
                    .map_err(|e| format!("Failed to create script: {}", e))?;

                // Set executable permission
                std::process::Command::new("chmod")
                    .args(["+x", script_path])
                    .output()
                    .map_err(|e| format!("Failed to set permission: {}", e))?;

                // Use open command to open .command file (will automatically execute in new terminal window)
                std::process::Command::new("open")
                    .arg(script_path)
                    .spawn()
                    .map_err(|e| format!("Failed to launch terminal: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            {
                let env_path = platform::get_env_file_path();
                // Create script
                let script_content = format!(
                    r#"#!/bin/bash
source {} 2>/dev/null
clear
echo "📱 WhatsApp Login Wizard"
echo ""
openclaw channels login --channel whatsapp --verbose
echo ""
read -p "Press Enter to close..."
"#,
                    env_path
                );

                let script_path = "/tmp/openclaw_whatsapp_login.sh";
                std::fs::write(script_path, &script_content)
                    .map_err(|e| format!("Failed to create script: {}", e))?;

                std::process::Command::new("chmod")
                    .args(["+x", script_path])
                    .output()
                    .map_err(|e| format!("Failed to set permission: {}", e))?;

                // Try different terminal emulators
                let terminals = ["gnome-terminal", "xfce4-terminal", "konsole", "xterm"];
                let mut launched = false;

                for term in terminals {
                    let result = std::process::Command::new(term)
                        .args(["--", script_path])
                        .spawn();

                    if result.is_ok() {
                        launched = true;
                        break;
                    }
                }

                if !launched {
                    return Err("Unable to launch terminal, please run manually: openclaw channels login --channel whatsapp".to_string());
                }
            }

            #[cfg(target_os = "windows")]
            {
                return Err("Windows does not support automatic terminal launch, please run manually: openclaw channels login --channel whatsapp".to_string());
            }

            #[cfg(not(target_os = "windows"))]
            Ok("WhatsApp login started in new terminal window, please check the popup terminal window and scan the QR code".to_string())
        }
        _ => Err(format!("Login wizard not supported for {}", channel_type)),
    }
}
