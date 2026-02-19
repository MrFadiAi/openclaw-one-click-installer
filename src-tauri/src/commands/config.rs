use crate::models::{
    AIConfigOverview, ChannelConfig, ConfiguredModel, ConfiguredProvider,
    MCPConfig, ModelConfig, OfficialProvider, SuggestedModel,
};
use crate::utils::{file, platform, shell, log_sanitizer};
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::command;

/// Load openclaw.json configuration
fn load_openclaw_config() -> Result<Value, String> {
    let config_path = platform::get_config_file_path();

    if !file::file_exists(&config_path) {
        return Ok(json!({}));
    }

    let content =
        file::read_file(&config_path).map_err(|e| format!("Failed to read configuration file: {}", e))?;

    // Strip UTF-8 BOM if present (Windows editors sometimes add this)
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    serde_json::from_str(content).map_err(|e| format!("Failed to parse configuration file: {}", e))
}

/// Save openclaw.json configuration
fn save_openclaw_config(config: &Value) -> Result<(), String> {
    let config_path = platform::get_config_file_path();

    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize configuration: {}", e))?;

    file::write_file(&config_path, &content).map_err(|e| format!("Failed to write configuration file: {}", e))
}

/// Load manager.json configuration (manager-specific settings)
fn load_manager_config() -> Result<Value, String> {
    let config_path = platform::get_manager_config_file_path();

    if !file::file_exists(&config_path) {
        return Ok(json!({}));
    }

    let content =
        file::read_file(&config_path).map_err(|e| format!("Failed to read manager configuration file: {}", e))?;

    // Strip UTF-8 BOM if present
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    serde_json::from_str(content).map_err(|e| format!("Failed to parse manager configuration file: {}", e))
}

/// Save manager.json configuration
fn save_manager_config(config: &Value) -> Result<(), String> {
    let config_path = platform::get_manager_config_file_path();

    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Failed to serialize manager configuration: {}", e))?;

    file::write_file(&config_path, &content).map_err(|e| format!("Failed to write manager configuration file: {}", e))
}

/// Get complete configuration
#[command]
pub async fn get_config() -> Result<Value, String> {
    info!("[Get Config] Reading openclaw.json configuration...");
    let result = load_openclaw_config();
    match &result {
        Ok(_) => info!("[Get Config] Configuration read successfully"),
        Err(e) => error!("[Get Config] Failed to read configuration: {}", e),
    }
    result
}

/// Save configuration
#[command]
pub async fn save_config(config: Value) -> Result<String, String> {
    info!("[Save Config] Saving openclaw.json configuration...");
    debug!(
        "[Save Config] Configuration content: {}",
        log_sanitizer::sanitize(&serde_json::to_string_pretty(&config).unwrap_or_default())
    );
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!("[Save Config] Configuration saved successfully");
            Ok("Configuration saved".to_string())
        }
        Err(e) => {
            error!("[Save Config] Failed to save configuration: {}", e);
            Err(e)
        }
    }
}

/// Get environment variable value
#[command]
pub async fn get_env_value(key: String) -> Result<Option<String>, String> {
    info!("[Get Env] Reading environment variable: {}", key);
    let env_path = platform::get_env_file_path();
    let value = file::read_env_value(&env_path, &key);
    match &value {
        Some(v) => debug!(
            "[Get Env] {}={} (masked)",
            key,
            if v.len() > 8 { "***" } else { v }
        ),
        None => debug!("[Get Env] {} does not exist", key),
    }
    Ok(value)
}

/// Save environment variable value
#[command]
pub async fn save_env_value(key: String, value: String) -> Result<String, String> {
    info!("[Save Env] Saving environment variable: {}", key);
    let env_path = platform::get_env_file_path();
    debug!("[Save Env] Environment file path: {}", env_path);

    match file::set_env_value(&env_path, &key, &value) {
        Ok(_) => {
            info!("[Save Env] Environment variable {} saved successfully", key);
            Ok("Environment variable saved".to_string())
        }
        Err(e) => {
            error!("[Save Env] Failed to save: {}", e);
            Err(format!("Failed to save environment variable: {}", e))
        }
    }
}

// ============ Gateway Token Commands ============

/// Generate random token
fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // Generate token using timestamp and random number
    let random_part: u64 = (timestamp as u64) ^ 0x5DEECE66Du64;
    format!("{:016x}{:016x}{:016x}",
        random_part,
        random_part.wrapping_mul(0x5DEECE66Du64),
        timestamp as u64
    )
}

/// Get or create Gateway Token
#[command]
pub async fn get_or_create_gateway_token() -> Result<String, String> {
    info!("[Gateway Token] Getting or creating Gateway Token...");

    let mut config = load_openclaw_config()?;

    // Check if token already exists
    if let Some(token) = config
        .pointer("/gateway/auth/token")
        .and_then(|v| v.as_str())
    {
        if !token.is_empty() {
            info!("[Gateway Token] Using existing Token");
            return Ok(token.to_string());
        }
    }

    // Generate new token
    let new_token = generate_token();
    info!("[Gateway Token] Generated new Token");

    // Ensure path exists
    if config.get("gateway").is_none() {
        config["gateway"] = json!({});
    }
    if config["gateway"].get("auth").is_none() {
        config["gateway"]["auth"] = json!({});
    }

    // Set token and mode
    config["gateway"]["auth"]["token"] = json!(new_token);
    config["gateway"]["auth"]["mode"] = json!("token");
    config["gateway"]["mode"] = json!("local");

    // Save configuration
    save_openclaw_config(&config)?;

    info!("[Gateway Token] Token saved to configuration");
    Ok(new_token)
}

/// Get Dashboard URL (with token)
#[command]
pub async fn get_dashboard_url() -> Result<String, String> {
    info!("[Dashboard URL] Getting Dashboard URL...");

    let token = get_or_create_gateway_token().await?;
    let url = format!("http://localhost:18789?token={}", token);

    info!("[Dashboard URL] URL generated");
    Ok(url)
}

/// Repair device token mismatch by deleting stale identity and paired device files.
/// After calling this, the gateway should be restarted to regenerate fresh device identity.
#[command]
pub async fn repair_device_token() -> Result<String, String> {
    info!("[Device Token Repair] Starting device token repair...");

    let config_dir = platform::get_config_dir();
    let identity_file = format!(
        "{}{}identity{}device.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );
    let paired_file = format!(
        "{}{}devices{}paired.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );

    let mut deleted = Vec::new();

    // Delete identity/device.json (stale device keypair)
    match std::fs::remove_file(&identity_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", identity_file);
            deleted.push("identity/device.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", identity_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", identity_file, e);
        }
    }

    // Delete devices/paired.json (stale paired device entries)
    match std::fs::remove_file(&paired_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", paired_file);
            deleted.push("devices/paired.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", paired_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", paired_file, e);
        }
    }

    // Delete identity/device-auth.json (stale device auth token)
    let device_auth_file = format!(
        "{}{}identity{}device-auth.json",
        config_dir,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    );
    match std::fs::remove_file(&device_auth_file) {
        Ok(_) => {
            info!("[Device Token Repair] Deleted: {}", device_auth_file);
            deleted.push("identity/device-auth.json".to_string());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            info!("[Device Token Repair] Not found (already clean): {}", device_auth_file);
        }
        Err(e) => {
            warn!("[Device Token Repair] Failed to delete {}: {}", device_auth_file, e);
        }
    }

    if deleted.is_empty() {
        info!("[Device Token Repair] No stale files found, identity was already clean");
        Ok("Device identity already clean. Please restart the service.".to_string())
    } else {
        info!("[Device Token Repair] Cleaned {} stale file(s): {:?}", deleted.len(), deleted);
        Ok(format!("Cleaned stale device files: {}. Please restart the service.", deleted.join(", ")))
    }
}

// ============ AI Configuration Commands ============

/// Get official Provider list (preset templates)
#[command]
pub async fn get_official_providers() -> Result<Vec<OfficialProvider>, String> {
    info!("[Official Provider] Getting official Provider preset list...");

    let providers = vec![
        OfficialProvider {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            icon: "🟣".to_string(),
            default_base_url: Some("https://api.anthropic.com".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/anthropic".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "claude-opus-4-5-20251101".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("Most powerful version, suitable for complex tasks".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "claude-sonnet-4-5-20250929".to_string(),
                    name: "Claude Sonnet 4.5".to_string(),
                    description: Some("Balanced version, high cost-performance ratio".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            icon: "🟢".to_string(),
            default_base_url: Some("https://api.openai.com/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/openai".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "gpt-4o".to_string(),
                    name: "GPT-4o".to_string(),
                    description: Some("Latest multimodal model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: true,
                },
                SuggestedModel {
                    id: "gpt-4o-mini".to_string(),
                    name: "GPT-4o Mini".to_string(),
                    description: Some("Fast and economical version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(4096),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "moonshot".to_string(),
            name: "Moonshot".to_string(),
            icon: "🌙".to_string(),
            default_base_url: Some("https://api.moonshot.cn/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/moonshot".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "kimi-k2.5".to_string(),
                    name: "Kimi K2.5".to_string(),
                    description: Some("Latest flagship model".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "moonshot-v1-128k".to_string(),
                    name: "Moonshot 128K".to_string(),
                    description: Some("Ultra-long context".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "qwen".to_string(),
            name: "Qwen (Tongyi Qianwen)".to_string(),
            icon: "🔮".to_string(),
            default_base_url: Some("https://dashscope.aliyuncs.com/compatible-mode/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/qwen".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "qwen-max".to_string(),
                    name: "Qwen Max".to_string(),
                    description: Some("Most powerful version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "qwen-plus".to_string(),
                    name: "Qwen Plus".to_string(),
                    description: Some("Balanced version".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            icon: "🔵".to_string(),
            default_base_url: Some("https://api.deepseek.com".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: None,
            suggested_models: vec![
                SuggestedModel {
                    id: "deepseek-chat".to_string(),
                    name: "DeepSeek V3".to_string(),
                    description: Some("Latest chat model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "deepseek-reasoner".to_string(),
                    name: "DeepSeek R1".to_string(),
                    description: Some("Reasoning-enhanced model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
        OfficialProvider {
            id: "glm".to_string(),
            name: "GLM (Zhipu)".to_string(),
            icon: "🔷".to_string(),
            default_base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/glm".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "glm-5".to_string(),
                    name: "GLM-5".to_string(),
                    description: Some("Latest flagship model".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            icon: "🟡".to_string(),
            default_base_url: Some("https://api.minimax.io/anthropic".to_string()),
            api_type: "anthropic-messages".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/minimax".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "minimax-m2.1".to_string(),
                    name: "MiniMax M2.1".to_string(),
                    description: Some("Latest model".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "venice".to_string(),
            name: "Venice AI".to_string(),
            icon: "🏛️".to_string(),
            default_base_url: Some("https://api.venice.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/venice".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "llama-3.3-70b".to_string(),
                    name: "Llama 3.3 70B".to_string(),
                    description: Some("Privacy-first inference".to_string()),
                    context_window: Some(128000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            icon: "🔄".to_string(),
            default_base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://docs.openclaw.ai/providers/openrouter".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "anthropic/claude-opus-4-5".to_string(),
                    name: "Claude Opus 4.5".to_string(),
                    description: Some("Access via OpenRouter".to_string()),
                    context_window: Some(200000),
                    max_tokens: Some(8192),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "ollama".to_string(),
            name: "Ollama (Local)".to_string(),
            icon: "🟠".to_string(),
            default_base_url: Some("http://localhost:11434".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: false,
            docs_url: Some("https://docs.openclaw.ai/providers/ollama".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "llama3".to_string(),
                    name: "Llama 3".to_string(),
                    description: Some("Run locally".to_string()),
                    context_window: Some(8192),
                    max_tokens: Some(4096),
                    recommended: true,
                },
            ],
        },
        OfficialProvider {
            id: "google".to_string(),
            name: "Google Gemini".to_string(),
            icon: "✨".to_string(),
            default_base_url: Some("https://generativelanguage.googleapis.com/v1beta/openai/".to_string()),
            api_type: "openai-completions".to_string(),
            requires_api_key: true,
            docs_url: Some("https://ai.google.dev/gemini-api/docs/openai".to_string()),
            suggested_models: vec![
                SuggestedModel {
                    id: "gemini-3-flash-preview".to_string(),
                    name: "Gemini 3 Flash".to_string(),
                    description: Some("Fast and efficient multimodal model (Preview)".to_string()),
                    context_window: Some(1048576),
                    max_tokens: Some(8192),
                    recommended: true,
                },
                SuggestedModel {
                    id: "gemini-3-pro-preview".to_string(),
                    name: "Gemini 3 Pro".to_string(),
                    description: Some("Complex reasoning tasks (Preview)".to_string()),
                    context_window: Some(1048576),
                    max_tokens: Some(8192),
                    recommended: false,
                },
            ],
        },
    ];

    info!(
        "[Official Provider] Returned {} official Provider presets",
        providers.len()
    );
    Ok(providers)
}

/// Get AI configuration overview
#[command]
pub async fn get_ai_config() -> Result<AIConfigOverview, String> {
    info!("[AI Config] Getting AI configuration overview...");

    let config_path = platform::get_config_file_path();
    info!("[AI Config] Configuration file path: {}", config_path);

    let config = load_openclaw_config()?;
    debug!("[AI Config] Configuration content: {}", serde_json::to_string_pretty(&config).unwrap_or_default());

    // Parse primary model
    let primary_model = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    info!("[AI Config] Primary model: {:?}", primary_model);

    // Parse available model list
    let available_models: Vec<String> = config
        .pointer("/agents/defaults/models")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();
    info!("[AI Config] Number of available models: {}", available_models.len());

    // Parse configured Providers
    let mut configured_providers: Vec<ConfiguredProvider> = Vec::new();

    let providers_value = config.pointer("/models/providers");
    info!("[AI Config] providers node exists: {}", providers_value.is_some());

    if let Some(providers) = providers_value.and_then(|v| v.as_object()) {
        info!("[AI Config] Found {} Providers", providers.len());

        for (provider_name, provider_config) in providers {
            info!("[AI Config] Parsing Provider: {}", provider_name);

            let base_url = provider_config
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let api_key = provider_config
                .get("apiKey")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let api_key_masked = api_key.as_ref().map(|key| {
                if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "****".to_string()
                }
            });

            // Parse model list
            let models_array = provider_config.get("models").and_then(|v| v.as_array());
            info!("[AI Config] Provider {} models array: {:?}", provider_name, models_array.map(|a| a.len()));

            let models: Vec<ConfiguredModel> = models_array
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();
                            let name = m
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            let full_id = format!("{}/{}", provider_name, id);
                            let is_primary = primary_model.as_ref() == Some(&full_id);

                            info!("[AI Config] Parsed model: {} (is_primary: {})", full_id, is_primary);

                            Some(ConfiguredModel {
                                full_id,
                                id,
                                name,
                                api_type: m.get("api").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                context_window: m
                                    .get("contextWindow")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                max_tokens: m
                                    .get("maxTokens")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                is_primary,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            info!("[AI Config] Provider {} parsing complete: {} models", provider_name, models.len());

            configured_providers.push(ConfiguredProvider {
                name: provider_name.clone(),
                base_url,
                api_key_masked,
                has_api_key: api_key.is_some(),
                models,
            });
        }
    } else {
        info!("[AI Config] providers configuration not found or incorrect format");
    }

    info!(
        "[AI Config] Final result - Primary model: {:?}, {} Providers, {} available models",
        primary_model,
        configured_providers.len(),
        available_models.len()
    );

    Ok(AIConfigOverview {
        primary_model,
        configured_providers,
        available_models,
    })
}

/// Add or update Provider
#[command]
pub async fn save_provider(
    provider_name: String,
    base_url: String,
    api_key: Option<String>,
    api_type: String,
    models: Vec<ModelConfig>,
) -> Result<String, String> {
    info!(
        "[Save Provider] Saving Provider: {} ({} models)",
        provider_name,
        models.len()
    );

    let mut config = load_openclaw_config()?;

    // Ensure paths exist
    if config.get("models").is_none() {
        config["models"] = json!({});
    }
    if config["models"].get("providers").is_none() {
        config["models"]["providers"] = json!({});
    }
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // Build model configuration
    let models_json: Vec<Value> = models
        .iter()
        .map(|m| {
            let mut model_obj = json!({
                "id": m.id,
                "name": m.name,
                "api": m.api.clone().unwrap_or(api_type.clone()),
                "input": if m.input.is_empty() { vec!["text".to_string()] } else { m.input.clone() },
            });

            if let Some(cw) = m.context_window {
                model_obj["contextWindow"] = json!(cw);
            }
            if let Some(mt) = m.max_tokens {
                model_obj["maxTokens"] = json!(mt);
            }
            if let Some(r) = m.reasoning {
                model_obj["reasoning"] = json!(r);
            }
            if let Some(cost) = &m.cost {
                model_obj["cost"] = json!({
                    "input": cost.input,
                    "output": cost.output,
                    "cacheRead": cost.cache_read,
                    "cacheWrite": cost.cache_write,
                });
            } else {
                model_obj["cost"] = json!({
                    "input": 0,
                    "output": 0,
                    "cacheRead": 0,
                    "cacheWrite": 0,
                });
            }

            model_obj
        })
        .collect();

    // Build Provider configuration
    let mut provider_config = json!({
        "baseUrl": base_url,
        "models": models_json,
    });

    // Handle API Key: if a new non-empty key is provided, use it; otherwise preserve the existing one
    if let Some(key) = api_key {
        if !key.is_empty() {
            // Use the newly provided API Key
            provider_config["apiKey"] = json!(key);
            info!("[Save Provider] Using new API Key");
        } else {
            // Empty string means no change, try to preserve the existing API Key
            if let Some(existing_key) = config
                .pointer(&format!("/models/providers/{}/apiKey", provider_name))
                .and_then(|v| v.as_str())
            {
                provider_config["apiKey"] = json!(existing_key);
                info!("[Save Provider] Preserving existing API Key");
            }
        }
    } else {
        // None means no change, try to preserve the existing API Key
        if let Some(existing_key) = config
            .pointer(&format!("/models/providers/{}/apiKey", provider_name))
            .and_then(|v| v.as_str())
        {
            provider_config["apiKey"] = json!(existing_key);
            info!("[Save Provider] Preserving existing API Key");
        }
    }

    // Save Provider configuration
    config["models"]["providers"][&provider_name] = provider_config;

    // Add models to agents.defaults.models
    for model in &models {
        let full_id = format!("{}/{}", provider_name, model.id);
        config["agents"]["defaults"]["models"][&full_id] = json!({});
    }

    // Update metadata
    let now = chrono::Utc::now().to_rfc3339();
    if config.get("meta").is_none() {
        config["meta"] = json!({});
    }
    config["meta"]["lastTouchedAt"] = json!(now);

    save_openclaw_config(&config)?;
    info!("[Save Provider] Provider {} saved successfully", provider_name);

    Ok(format!("Provider {} saved", provider_name))
}

/// Delete Provider
#[command]
pub async fn delete_provider(provider_name: String) -> Result<String, String> {
    info!("[Delete Provider] Deleting Provider: {}", provider_name);

    let mut config = load_openclaw_config()?;

    // Delete Provider configuration
    if let Some(providers) = config
        .pointer_mut("/models/providers")
        .and_then(|v| v.as_object_mut())
    {
        providers.remove(&provider_name);
    }

    // Delete related models
    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        let keys_to_remove: Vec<String> = models
            .keys()
            .filter(|k| k.starts_with(&format!("{}/", provider_name)))
            .cloned()
            .collect();

        for key in keys_to_remove {
            models.remove(&key);
        }
    }

    // If primary model belongs to this Provider, clear primary model
    if let Some(primary) = config
        .pointer("/agents/defaults/model/primary")
        .and_then(|v| v.as_str())
    {
        if primary.starts_with(&format!("{}/", provider_name)) {
            config["agents"]["defaults"]["model"]["primary"] = json!(null);
        }
    }

    save_openclaw_config(&config)?;
    info!("[Delete Provider] Provider {} deleted", provider_name);

    Ok(format!("Provider {} deleted", provider_name))
}

/// Set primary model
#[command]
pub async fn set_primary_model(model_id: String) -> Result<String, String> {
    info!("[Set Primary Model] Setting primary model: {}", model_id);

    let mut config = load_openclaw_config()?;

    // Ensure paths exist
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("model").is_none() {
        config["agents"]["defaults"]["model"] = json!({});
    }

    // Set primary model
    config["agents"]["defaults"]["model"]["primary"] = json!(model_id);

    save_openclaw_config(&config)?;
    info!("[Set Primary Model] Primary model set to: {}", model_id);

    Ok(format!("Primary model set to {}", model_id))
}

/// Add model to available list
#[command]
pub async fn add_available_model(model_id: String) -> Result<String, String> {
    info!("[Add Model] Adding model to available list: {}", model_id);

    let mut config = load_openclaw_config()?;

    // Ensure paths exist
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }
    if config["agents"]["defaults"].get("models").is_none() {
        config["agents"]["defaults"]["models"] = json!({});
    }

    // Add model
    config["agents"]["defaults"]["models"][&model_id] = json!({});

    save_openclaw_config(&config)?;
    info!("[Add Model] Model {} added", model_id);

    Ok(format!("Model {} added", model_id))
}

/// Remove model from available list
#[command]
pub async fn remove_available_model(model_id: String) -> Result<String, String> {
    info!("[Remove Model] Removing model from available list: {}", model_id);

    let mut config = load_openclaw_config()?;

    if let Some(models) = config
        .pointer_mut("/agents/defaults/models")
        .and_then(|v| v.as_object_mut())
    {
        models.remove(&model_id);
    }

    save_openclaw_config(&config)?;
    info!("[Remove Model] Model {} removed", model_id);

    Ok(format!("Model {} removed", model_id))
}

// ============ MCP Configuration Commands ============

/// Load MCP config from separate mcps.json file
fn load_mcp_config_file() -> Result<HashMap<String, MCPConfig>, String> {
    let config_path = platform::get_mcp_config_file_path();
    let path = std::path::Path::new(&config_path);
    
    if !path.exists() {
        return Ok(HashMap::new());
    }
    
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read mcps.json: {}", e))?;
    
    let configs: HashMap<String, MCPConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse mcps.json: {}", e))?;
    
    Ok(configs)
}

/// Save MCP config to separate mcps.json file AND sync to ~/.mcporter/mcporter.json
fn save_mcp_config_file(configs: &HashMap<String, MCPConfig>) -> Result<(), String> {
    // 1. Save to Manager's private config (mcps.json)
    let config_path = platform::get_mcp_config_file_path();
    let content = serde_json::to_string_pretty(configs)
        .map_err(|e| format!("Failed to serialize MCP config: {}", e))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write mcps.json: {}", e))?;
    
    // 2. Sync enabled servers to system mcporter config (~/.mcporter/mcporter.json)
    if let Err(e) = sync_to_mcporter(configs) {
        warn!("Failed to sync to mcporter: {}", e);
        // Don't fail the whole save operation if sync fails
    }
    
    Ok(())
}

fn sync_to_mcporter(configs: &HashMap<String, MCPConfig>) -> Result<(), String> {
    let mcporter_path = platform::get_mcporter_config_file_path();
    let path = std::path::Path::new(&mcporter_path);

    // Create ~/.mcporter directory if missing
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create mcporter config dir: {}", e))?;
        }
    }

    // Load existing mcporter config or create new
    let mut root_val: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read mcporter.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| serde_json::json!({ "mcpServers": {} }))
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    // Ensure mcpServers object exists
    if root_val.get("mcpServers").is_none() {
        root_val["mcpServers"] = serde_json::json!({});
    }

    let mcp_servers_obj = root_val["mcpServers"].as_object_mut().unwrap();

    // Sync: Add/Update enabled servers from Manager
    for (name, config) in configs {
        if config.enabled {
            // Convert MCPConfig to serde_json::Value
            // Note: We skip 'enabled' field as mcporter doesn't use it (presence = enabled)
            let mut server_val = serde_json::to_value(config)
                .map_err(|e| format!("Failed to serialize config for {}: {}", name, e))?;
            
            if let Some(obj) = server_val.as_object_mut() {
                obj.remove("enabled");
            }
            
            mcp_servers_obj.insert(name.clone(), server_val);
        } else {
            // Remove disabled servers if they were previously synced
            mcp_servers_obj.remove(name);
        }
    }
    
    // Important: We do NOT remove servers that are in mcporter but NOT in Manager,
    // to respect user's manual edits or other tools. We only manage the ones we know about.

    // Write back
    let new_content = serde_json::to_string_pretty(&root_val)
        .map_err(|e| format!("Failed to serialize mcporter config: {}", e))?;
    
    std::fs::write(path, new_content)
        .map_err(|e| format!("Failed to write mcporter.json: {}", e))?;

    Ok(())
}

/// Get MCP configuration
#[command]
pub async fn get_mcp_config() -> Result<HashMap<String, MCPConfig>, String> {
    info!("[MCP Config] Getting MCP configuration...");
    
    let configs = load_mcp_config_file()?;
        
    info!("[MCP Config] Found {} MCP servers", configs.len());
    Ok(configs)
}

/// Save MCP configuration
#[command]
pub async fn save_mcp_config(
    name: String,
    config: Option<MCPConfig>,
) -> Result<String, String> {
    info!("[Save MCP] Saving MCP configuration for: {}", name);
    
    let mut configs = load_mcp_config_file()?;
    
    if let Some(mcp) = config {
        configs.insert(name.clone(), mcp);
        info!("[Save MCP] Updated configuration for {}", name);
    } else {
        configs.remove(&name);
        info!("[Save MCP] Deleted configuration for {}", name);
    }
    
    save_mcp_config_file(&configs)?;
    Ok(format!("MCP configuration saved for {}", name))
}

/// Install MCP server from a Git repository URL
#[command]
pub async fn install_mcp_from_git(url: String) -> Result<String, String> {
    info!("[MCP Install] Installing MCP from: {}", url);

    // Extract repo name from URL (e.g. "excalidraw-mcp" from "https://github.com/excalidraw/excalidraw-mcp")
    let repo_name = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .ok_or_else(|| "Invalid repository URL".to_string())?
        .to_string();

    if repo_name.is_empty() {
        return Err("Could not extract repository name from URL".to_string());
    }

    info!("[MCP Install] Repository name: {}", repo_name);

    // Create mcps directory if it doesn't exist
    let mcps_dir = platform::get_mcp_install_dir();
    std::fs::create_dir_all(&mcps_dir)
        .map_err(|e| format!("Failed to create mcps directory: {}", e))?;

    let install_path = if platform::is_windows() {
        format!("{}\\{}", mcps_dir, repo_name)
    } else {
        format!("{}/{}", mcps_dir, repo_name)
    };

    // Remove existing directory if present (re-install)
    if std::path::Path::new(&install_path).exists() {
        info!("[MCP Install] Removing existing installation at {}", install_path);
        std::fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
    }

    // Step 1: Clone the repository
    info!("[MCP Install] Cloning repository...");
    let clone_output = shell::run_command("git", &["clone", &url, &install_path])
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }
    info!("[MCP Install] Clone successful");

    // Step 2: npm install
    info!("[MCP Install] Running npm install...");
    let npm_cmd = if platform::is_windows() { "npm.cmd" } else { "npm" };

    let mut npm_install = std::process::Command::new(npm_cmd);
    npm_install.args(&["install"]).current_dir(&install_path);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        npm_install.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let install_output = npm_install.output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        return Err(format!("npm install failed: {}", stderr));
    }
    info!("[MCP Install] npm install successful");

    // Step 3: npm run build
    info!("[MCP Install] Running npm run build...");
    let mut npm_build = std::process::Command::new(npm_cmd);
    npm_build.args(&["run", "build"]).current_dir(&install_path);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        npm_build.creation_flags(0x08000000);
    }

    let build_output = npm_build.output()
        .map_err(|e| format!("Failed to run npm run build: {}", e))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        warn!("[MCP Install] npm run build failed (may not have a build step): {}", stderr);
        // Don't fail — some MCPs don't need a build step
    } else {
        info!("[MCP Install] npm run build successful");
    }

    // Step 4: Auto-configure in mcps.json
    info!("[MCP Install] Configuring MCP in mcps.json...");
    let mut configs = load_mcp_config_file()?;

    // Determine the entry point (dist/index.js or index.js)
    let dist_index = if platform::is_windows() {
        format!("{}\\dist\\index.js", install_path)
    } else {
        format!("{}/dist/index.js", install_path)
    };

    let entry_point = if std::path::Path::new(&dist_index).exists() {
        dist_index
    } else {
        let root_index = if platform::is_windows() {
            format!("{}\\index.js", install_path)
        } else {
            format!("{}/index.js", install_path)
        };
        if std::path::Path::new(&root_index).exists() {
            root_index
        } else {
            dist_index
        }
    };

    configs.insert(repo_name.clone(), MCPConfig {
        command: "node".to_string(),
        args: vec![entry_point, "--stdio".to_string()],
        env: HashMap::new(),
        url: String::new(),
        enabled: true,
    });

    save_mcp_config_file(&configs)?;
    info!("[MCP Install] Installation complete for {}", repo_name);
    Ok(format!("Successfully installed MCP: {}", repo_name))
}

/// Uninstall an MCP server
#[command]
pub async fn uninstall_mcp(name: String) -> Result<String, String> {
    info!("[MCP Uninstall] Uninstalling MCP: {}", name);

    // Remove directory
    let mcps_dir = platform::get_mcp_install_dir();
    let install_path = if platform::is_windows() {
        format!("{}\\{}", mcps_dir, name)
    } else {
        format!("{}/{}", mcps_dir, name)
    };

    if std::path::Path::new(&install_path).exists() {
        std::fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove MCP directory: {}", e))?;
        info!("[MCP Uninstall] Removed directory: {}", install_path);
    }

    // Remove from mcps.json
    let mut configs = load_mcp_config_file()?;
    configs.remove(&name);
    save_mcp_config_file(&configs)?;

    info!("[MCP Uninstall] Uninstalled MCP: {}", name);
    Ok(format!("Successfully uninstalled MCP: {}", name))
}

/// Check if mcporter is installed
#[command]
pub async fn check_mcporter_installed() -> Result<bool, String> {
    info!("[mcporter] Checking if mcporter is installed...");
    let installed = shell::command_exists("mcporter");
    info!("[mcporter] Installed: {}", installed);
    Ok(installed)
}

/// Install mcporter via npm
#[command]
pub async fn install_mcporter() -> Result<String, String> {
    info!("[mcporter] Installing mcporter globally via npm...");

    let npm_cmd = if platform::is_windows() { "npm.cmd" } else { "npm" };

    let mut cmd = std::process::Command::new(npm_cmd);
    cmd.args(&["install", "-g", "mcporter"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run npm install: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("npm install -g mcporter failed: {}", stderr));
    }

    info!("[mcporter] Installation successful");
    Ok("mcporter installed successfully".to_string())
}

/// Uninstall Mcporter
#[command]
pub async fn uninstall_mcporter() -> Result<String, String> {
    info!("Uninstalling mcporter globally via npm");

    #[cfg(target_os = "windows")]
    let program = "cmd";
    #[cfg(target_os = "windows")]
    let args = ["/C", "npm uninstall -g @openclaw/mcporter"];

    #[cfg(not(target_os = "windows"))]
    let program = "npm";
    #[cfg(not(target_os = "windows"))]
    let args = ["uninstall", "-g", "@openclaw/mcporter"];

    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute npm uninstall: {}", e))?;

    if output.status.success() {
        info!("mcporter uninstalled successfully");
        Ok("MCPorter uninstalled successfully".to_string())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to uninstall mcporter: {}", error_msg);
        Err(format!("Failed to uninstall mcporter: {}", error_msg))
    }
}

/// Install MCP server as an OpenClaw plugin (using openclaw plugins install)
#[command]
pub async fn install_mcp_plugin(url: String) -> Result<String, String> {
    info!("[MCP Plugin] Installing MCP plugin from: {}", url);

    let result = shell::run_openclaw(&["plugins", "install", &url])
        .map_err(|e| format!("Failed to install plugin: {}", e))?;

    info!("[MCP Plugin] Installation result: {}", result);
    Ok(format!("Successfully installed MCP plugin from: {}", url))
}

/// Set openclaw config via CLI (openclaw config set <key> <value>)
#[command]
pub async fn openclaw_config_set(key: String, value: String) -> Result<String, String> {
    info!("[Config CLI] Setting config: {} = {}", key, value);

    let result = shell::run_openclaw(&["config", "set", &key, &value])
        .map_err(|e| format!("Failed to set config: {}", e))?;

    info!("[Config CLI] Set result: {}", result);
    Ok(format!("Set {} = {}", key, value))
}

/// Test an MCP server connectivity
#[command]
pub async fn test_mcp_server(server_type: String, target: String, command: Option<String>, args: Option<Vec<String>>) -> Result<String, String> {
    info!("[MCP Test] Testing MCP server: type={}, target={}", server_type, target);

    if server_type == "url" {
        // Remote HTTP MCP: POST an MCP initialize request to the URL
        let mut cmd = std::process::Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
        cmd.args(&[
            "-s", "-w", "\n%{http_code}",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-H", "Accept: text/event-stream, application/json",
            "-d", r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#,
            "--max-time", "10",
            &target,
        ]);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        match cmd.output() {
            Ok(out) => {
                let output_str = String::from_utf8_lossy(&out.stdout).to_string();
                let lines: Vec<&str> = output_str.trim().lines().collect();
                let status_code = lines.last().unwrap_or(&"0");
                let body = if lines.len() > 1 { lines[..lines.len()-1].join("\n") } else { String::new() };

                if status_code.starts_with("2") {
                    // Try to extract server name from JSON response
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        if let Some(name) = json.pointer("/result/serverInfo/name") {
                            return Ok(format!("✅ Server reachable: {} (HTTP {})", name.as_str().unwrap_or("unknown"), status_code));
                        }
                    }
                    // Try to parse SSE response for server info
                    for line in body.lines() {
                        if line.starts_with("data:") {
                            let data = line.trim_start_matches("data:").trim();
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(name) = json.pointer("/result/serverInfo/name") {
                                    return Ok(format!("✅ Server reachable: {} (HTTP {})", name.as_str().unwrap_or("unknown"), status_code));
                                }
                            }
                        }
                    }
                    Ok(format!("✅ Server reachable (HTTP {})", status_code))
                } else {
                    Err(format!("❌ Server returned HTTP {}", status_code))
                }
            }
            Err(e) => Err(format!("Failed to test URL: {}", e))
        }
    } else {
        // Local stdio MCP: spawn the command directly with proper args
        let cmd_name = command.unwrap_or(target.clone());
        let cmd_args = args.unwrap_or_default();
        
        info!("[MCP Test] Spawning: {} {:?}", cmd_name, cmd_args);

        let extended_path = shell::get_extended_path();
        
        // On Windows, use cmd /c to resolve .cmd files (npx.cmd, node.cmd, etc.)
        #[cfg(windows)]
        let mut cmd = {
            let mut c = std::process::Command::new("cmd");
            let mut full_args = vec!["/c".to_string(), cmd_name.clone()];
            full_args.extend(cmd_args.clone());
            c.args(&full_args);
            c
        };
        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = std::process::Command::new(&cmd_name);
            c.args(&cmd_args);
            c
        };

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("PATH", &extended_path);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                // Send MCP initialize request via stdin
                if let Some(ref mut stdin) = child.stdin {
                    use std::io::Write;
                    let init_msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
                    let _ = writeln!(stdin, "Content-Length: {}\r\n\r\n{}", init_msg.len(), init_msg);
                }
                
                // Wait briefly then check
                std::thread::sleep(std::time::Duration::from_millis(3000));
                
                match child.try_wait() {
                    Ok(Some(status)) => {
                        // Process exited — read stderr for error info
                        let stderr = child.stderr.take().map(|mut s| {
                            let mut buf = String::new();
                            use std::io::Read;
                            let _ = s.read_to_string(&mut buf);
                            buf
                        }).unwrap_or_default();
                        
                        if status.success() {
                            Ok("✅ Server process started and exited cleanly".to_string())
                        } else {
                            Err(format!("❌ Server exited with {}\n{}", status, stderr.trim()))
                        }
                    }
                    Ok(None) => {
                        // Still running — good! Kill it and report success
                        let _ = child.kill();
                        Ok(format!("✅ Server is running (process started successfully)\nCommand: {} {}", cmd_name, cmd_args.join(" ")))
                    }
                    Err(e) => {
                        let _ = child.kill();
                        Err(format!("Failed to check process: {}", e))
                    }
                }
            }
            Err(e) => {
                Err(format!("❌ Failed to start server: {}\nCommand: {} {}", e, cmd_name, cmd_args.join(" ")))
            }
        }
    }
}

// ============ Legacy Compatibility ============

/// Get all supported AI Providers (legacy compatibility)
#[command]
pub async fn get_ai_providers() -> Result<Vec<crate::models::AIProviderOption>, String> {
    info!("[AI Provider] Getting supported AI Provider list (legacy)...");

    let official = get_official_providers().await?;
    let providers: Vec<crate::models::AIProviderOption> = official
        .into_iter()
        .map(|p| crate::models::AIProviderOption {
            id: p.id,
            name: p.name,
            icon: p.icon,
            default_base_url: p.default_base_url,
            requires_api_key: p.requires_api_key,
            models: p
                .suggested_models
                .into_iter()
                .map(|m| crate::models::AIModelOption {
                    id: m.id,
                    name: m.name,
                    description: m.description,
                    recommended: m.recommended,
                })
                .collect(),
        })
        .collect();

    Ok(providers)
}

// ============ Channel Configuration ============

/// Get channel configuration - read from openclaw.json and env file
#[command]
pub async fn get_channels_config() -> Result<Vec<ChannelConfig>, String> {
    info!("[Channel Config] Getting channel configuration list...");

    let config = load_openclaw_config()?;
    let channels_obj = config.get("channels").cloned().unwrap_or(json!({}));
    let env_path = platform::get_env_file_path();
    debug!("[Channel Config] Environment file path: {}", env_path);

    let mut channels = Vec::new();

    // List of supported channel types and their test fields
    let channel_types = vec![
        ("telegram", "telegram", vec!["userId"]),
        ("discord", "discord", vec!["testChannelId"]),
        ("slack", "slack", vec!["testChannelId"]),
        ("feishu", "feishu", vec!["testChatId"]),
        ("whatsapp", "whatsapp", vec![]),
        ("imessage", "imessage", vec![]),
        ("wechat", "wechat", vec![]),
        ("dingtalk", "dingtalk", vec![]),
    ];

    for (channel_id, channel_type, test_fields) in channel_types {
        let channel_config = channels_obj.get(channel_id);

        let enabled = channel_config
            .and_then(|c| c.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Convert channel configuration to HashMap
        let mut config_map: HashMap<String, Value> = if let Some(cfg) = channel_config {
            if let Some(obj) = cfg.as_object() {
                obj.iter()
                    .filter(|(k, _)| *k != "enabled") // Exclude enabled field
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Read test fields from env file
        for field in test_fields {
            let env_key = format!(
                "OPENCLAW_{}_{}",
                channel_id.to_uppercase(),
                field.to_uppercase()
            );
            if let Some(value) = file::read_env_value(&env_path, &env_key) {
                config_map.insert(field.to_string(), json!(value));
            }
        }

        // Clean up any legacy 'pairing' or 'allowlist' keys that shouldn't be here
        config_map.remove("pairing");
        config_map.remove("allowlist");

        // Determine if configured (has any non-empty configuration items)
        let has_config = !config_map.is_empty() || enabled;

        channels.push(ChannelConfig {
            id: channel_id.to_string(),
            channel_type: channel_type.to_string(),
            enabled: has_config,
            config: config_map,
        });
    }

    info!("[Channel Config] Returned {} channel configurations", channels.len());
    for ch in &channels {
        debug!("[Channel Config] - {}: enabled={}", ch.id, ch.enabled);
    }
    Ok(channels)
}

/// Save channel configuration - save to openclaw.json
#[command]
pub async fn save_channel_config(channel: ChannelConfig) -> Result<String, String> {
    info!(
        "[Save Channel Config] Saving channel configuration: {} ({})",
        channel.id, channel.channel_type
    );

    let mut config = load_openclaw_config()?;
    let env_path = platform::get_env_file_path();
    debug!("[Save Channel Config] Environment file path: {}", env_path);

    // DEBUG: Log received keys
    info!("[Save Channel Config] Config keys: {:?}", channel.config.keys());

    // Ensure channels object exists
    if config.get("channels").is_none() {
        config["channels"] = json!({});
    }

    if config.get("plugins").is_none() {
        config["plugins"] = json!({
            "allow": [],
            "entries": {}
        });
    }
    if config["plugins"].get("allow").is_none() {
        config["plugins"]["allow"] = json!([]);
    }
    if config["plugins"].get("entries").is_none() {
        config["plugins"]["entries"] = json!({});
    }

    // These fields are only for testing, not saved to openclaw.json, but saved to env file
    let test_only_fields = vec!["userId", "testChatId", "testChannelId"];

    // Update channels configuration - MERGE with existing
    if let Some(existing_channel) = config["channels"].get_mut(&channel.id).and_then(|v| v.as_object_mut()) {
        existing_channel.insert("enabled".to_string(), json!(true));
        
        // Clean up legacy invalid keys
        existing_channel.remove("pairing");
        existing_channel.remove("allowlist");

        for (key, value) in &channel.config {
            if test_only_fields.contains(&key.as_str()) {
                let env_key = format!("OPENCLAW_{}_{}", channel.id.to_uppercase(), key.to_uppercase());
                if let Some(val_str) = value.as_str() {
                    let _ = file::set_env_value(&env_path, &env_key, val_str);
                }
            } else {
                 existing_channel.insert(key.clone(), value.clone());
            }
        }
    } else {
        let mut channel_obj = json!({ "enabled": true });

        for (key, value) in &channel.config {
            if test_only_fields.contains(&key.as_str()) {
                let env_key = format!("OPENCLAW_{}_{}", channel.id.to_uppercase(), key.to_uppercase());
                if let Some(val_str) = value.as_str() {
                    let _ = file::set_env_value(&env_path, &env_key, val_str);
                }
            } else {
                channel_obj[key] = value.clone();
            }
        }
        config["channels"][&channel.id] = channel_obj;
    }

    // Cleanup legacy attempts
    if let Some(plugin_entry) = config["plugins"]["entries"].get_mut(&channel.id).and_then(|v| v.as_object_mut()) {
        plugin_entry.remove("allowlist");
        plugin_entry.remove("pairing");
    }
    // Remove global allowlist (invalid at root level)
    if let Some(obj) = config.as_object_mut() {
        obj.remove("allowlist");
    }

    // Save configuration
    info!("[Save Channel Config] Writing configuration file...");
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!(
                "[Save Channel Config] {} configuration saved successfully",
                channel.channel_type
            );
            Ok(format!("{} configuration saved", channel.channel_type))
        }
        Err(e) => {
            error!("[Save Channel Config] Failed to save: {}", e);
            Err(e)
        }
    }
}

/// Clear channel configuration - delete specified channel configuration from openclaw.json
#[command]
pub async fn clear_channel_config(channel_id: String) -> Result<String, String> {
    info!("[Clear Channel Config] Clearing channel configuration: {}", channel_id);

    let mut config = load_openclaw_config()?;
    let env_path = platform::get_env_file_path();

    // Delete channel from channels object
    if let Some(channels) = config.get_mut("channels").and_then(|v| v.as_object_mut()) {
        channels.remove(&channel_id);
        info!("[Clear Channel Config] Deleted from channels: {}", channel_id);
    }

    // Delete from plugins.allow array
    if let Some(allow_arr) = config.pointer_mut("/plugins/allow").and_then(|v| v.as_array_mut()) {
        allow_arr.retain(|v| v.as_str() != Some(&channel_id));
        info!("[Clear Channel Config] Deleted from plugins.allow: {}", channel_id);
    }

    // Delete from plugins.entries
    if let Some(entries) = config.pointer_mut("/plugins/entries").and_then(|v| v.as_object_mut()) {
        entries.remove(&channel_id);
        info!("[Clear Channel Config] Deleted from plugins.entries: {}", channel_id);
    }

    // Clear related environment variables
    let env_prefixes = vec![
        format!("OPENCLAW_{}_USERID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHATID", channel_id.to_uppercase()),
        format!("OPENCLAW_{}_TESTCHANNELID", channel_id.to_uppercase()),
    ];
    for env_key in env_prefixes {
        let _ = file::remove_env_value(&env_path, &env_key);
    }

    // Save configuration
    match save_openclaw_config(&config) {
        Ok(_) => {
            info!("[Clear Channel Config] {} configuration cleared", channel_id);
            Ok(format!("{} configuration cleared", channel_id))
        }
        Err(e) => {
            error!("[Clear Channel Config] Failed to clear: {}", e);
            Err(e)
        }
    }
}

// ============ Telegram Multi-Account Management ============

/// Telegram account info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramAccount {
    pub id: String,
    #[serde(alias = "botToken", alias = "bot_token")]
    pub bot_token: String,
    #[serde(alias = "groupPolicy", alias = "group_policy")]
    pub group_policy: Option<String>,
    #[serde(alias = "dmPolicy", alias = "dm_policy")]
    pub dm_policy: Option<String>,
    #[serde(alias = "streamMode", alias = "stream_mode")]
    pub stream_mode: Option<String>,
    #[serde(alias = "exclusiveTopics", alias = "exclusive_topics")]
    pub exclusive_topics: Option<Vec<String>>,
    pub groups: Option<serde_json::Value>,
    pub primary: Option<bool>,
    #[serde(alias = "allowFrom", alias = "allow_from")]
    pub allow_from: Option<Vec<String>>,
}

/// Get all Telegram bot accounts
#[command]
pub async fn get_telegram_accounts() -> Result<Vec<TelegramAccount>, String> {
    info!("[Telegram Accounts] Getting accounts...");
    let config = load_openclaw_config()?;

    let mut accounts = Vec::new();

    // Check for multi-account structure: channels.telegram.accounts
    if let Some(accts) = config.pointer("/channels/telegram/accounts").and_then(|v| v.as_object()) {
        for (id, acct_val) in accts {
            accounts.push(TelegramAccount {
                id: id.to_lowercase().replace(' ', "-"),
                bot_token: acct_val.get("botToken").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                group_policy: acct_val.get("groupPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                dm_policy: acct_val.get("dmPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                stream_mode: acct_val.get("streamMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
                exclusive_topics: {
                    // Re-infer exclusive topics from group config
                    // Logic: If a group has requireMention=true and specific topics have requireMention=false, those are exclusive topics.
                    let mut inferred_topics = Vec::new();
                    if let Some(groups_map) = acct_val.get("groups").and_then(|g| g.as_object()) {
                        for (_, group_val) in groups_map {
                             // Check if group is muted (requireMention=true)
                             if group_val.get("requireMention").and_then(|v| v.as_bool()).unwrap_or(false) {
                                 if let Some(topics_map) = group_val.get("topics").and_then(|t| t.as_object()) {
                                     for (tid, tval) in topics_map {
                                         // Check if topic is unmuted (requireMention=false)
                                         if !tval.get("requireMention").and_then(|v| v.as_bool()).unwrap_or(true) {
                                             inferred_topics.push(tid.clone());
                                         }
                                     }
                                 }
                             }
                        }
                    }
                    if inferred_topics.is_empty() { None } else { Some(inferred_topics) }
                },
                groups: acct_val.get("groups").cloned(),
                primary: None, // Will be set below
                allow_from: acct_val.get("allowFrom")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| {
                        if let Some(s) = v.as_str() { Some(s.to_string()) }
                        else if let Some(n) = v.as_i64() { Some(n.to_string()) }
                        else { None }
                    }).collect()),
            });
        }
    }

    // Fallback: single-bot config (botToken at top level)
    if accounts.is_empty() {
        if let Some(token) = config.pointer("/channels/telegram/botToken").and_then(|v| v.as_str()) {
            if !token.is_empty() {
                accounts.push(TelegramAccount {
                    id: "default".to_string(),
                    bot_token: token.to_string(),
                    group_policy: config.pointer("/channels/telegram/groupPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    dm_policy: config.pointer("/channels/telegram/dmPolicy").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    stream_mode: config.pointer("/channels/telegram/streamMode").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    exclusive_topics: None,
                    groups: config.pointer("/channels/telegram/groups").cloned(),
                    primary: None,
                    allow_from: config.pointer("/channels/telegram/allowFrom")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| {
                            if let Some(s) = v.as_str() { Some(s.to_string()) }
                            else if let Some(n) = v.as_i64() { Some(n.to_string()) }
                            else { None }
                        }).collect()),
                });
            }
        }
    }



    // Load primary bot account from manager.json (safe from Core schema)
    let manager_config = load_manager_config().unwrap_or(json!({}));
    let primary_account_id = manager_config.pointer("/primaryBotAccount").and_then(|v: &Value| v.as_str());
    
    if let Some(pid) = primary_account_id {
        for acct in &mut accounts {
            if acct.id == pid {
                acct.primary = Some(true);
            } else {
                acct.primary = Some(false);
            }
        }
    }

    info!("[Telegram Accounts] Found {} accounts", accounts.len());
    Ok(accounts)
}

/// Save a Telegram bot account
#[command]
pub async fn save_telegram_account(account: TelegramAccount) -> Result<String, String> {
    // Normalize account ID to lowercase and replace spaces with dashes
    let account_id = account.id.to_lowercase().replace(' ', "-");
    info!("[Telegram Accounts] Saving account: {}", account_id);
    let mut config = load_openclaw_config()?;

    // Ensure channels.telegram exists
    if config.get("channels").is_none() {
        config["channels"] = json!({});
    }
    if config["channels"].get("telegram").is_none() {
        config["channels"]["telegram"] = json!({ "enabled": true });
    }

    // Ensure accounts object exists
    if config["channels"]["telegram"].get("accounts").is_none() {
        config["channels"]["telegram"]["accounts"] = json!({});
    }

    // Migrate single-bot to accounts if this is the first additional account
    if let Some(top_token) = config["channels"]["telegram"].get("botToken").and_then(|v| v.as_str()).map(|s| s.to_string()) {
        if !top_token.is_empty() {
            // Move existing single-bot config to accounts["default"]
            let mut existing = json!({
                "botToken": top_token,
                "groupPolicy": config["channels"]["telegram"].get("groupPolicy").cloned().unwrap_or(json!(null)),
                "dmPolicy": config["channels"]["telegram"].get("dmPolicy").cloned().unwrap_or(json!(null)),
                "streamMode": config["channels"]["telegram"].get("streamMode").cloned().unwrap_or(json!(null)),
                "groups": config["channels"]["telegram"].get("groups").cloned().unwrap_or(json!(null)),
            });

            // Migrate allowList
            if let Some(allow_from) = config["channels"]["telegram"].get("allowFrom").cloned() {
                existing["allowFrom"] = allow_from;
            }
             if let Some(group_allow_from) = config["channels"]["telegram"].get("groupAllowFrom").cloned() {
                existing["groupAllowFrom"] = group_allow_from;
            }

            config["channels"]["telegram"]["accounts"]["default"] = existing;
            
            // Remove top-level single-bot fields
            if let Some(tg) = config["channels"]["telegram"].as_object_mut() {
                tg.remove("botToken");
                tg.remove("groupPolicy");
                tg.remove("dmPolicy");
                tg.remove("streamMode");
                tg.remove("groups");
                tg.remove("allowFrom");
                tg.remove("groupAllowFrom");
            }
        }
    }

    // If this account is set as primary, unset primary for all others
    // (This is now handled by only storing one ID in `meta`, so no need to iterate and clear others manually)

    // Build account object
    let mut acct_obj = json!({
        "botToken": account.bot_token,
    });
    if let Some(gp) = &account.group_policy {
        acct_obj["groupPolicy"] = json!(gp);
    }
    if let Some(dp) = &account.dm_policy {
        acct_obj["dmPolicy"] = json!(dp);
    }

    // Save allowFrom (DM user IDs) — handled independently of dm_policy
    info!("[Telegram Accounts] allow_from received: {:?}", account.allow_from);
    let dm_policy_str = account.dm_policy.as_deref().unwrap_or("");
    if dm_policy_str == "open" {
        // dmPolicy="open" requires allowFrom to include "*"
        acct_obj["allowFrom"] = json!(["*"]);
    } else if let Some(ref af) = account.allow_from {
        if !af.is_empty() {
            // Convert string IDs to numbers where possible for Core compatibility
            let allow_vals: Vec<serde_json::Value> = af.iter().map(|id| {
                if let Ok(n) = id.parse::<i64>() { json!(n) } else { json!(id) }
            }).collect();
            info!("[Telegram Accounts] Saving allowFrom: {:?}", allow_vals);
            acct_obj["allowFrom"] = json!(allow_vals);
        }
    } else {
        // Auto-inherit from primary bot if no explicit allow_from provided
        let primary_id = load_manager_config()
            .unwrap_or(json!({}))
            .pointer("/primaryBotAccount")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if let Some(pid) = primary_id {
            if pid != account_id {
                // Read primary account's allowFrom
                if let Some(primary_allow) = config.pointer(&format!("/channels/telegram/accounts/{}/allowFrom", pid))
                    .and_then(|v| v.as_array()) {
                    if !primary_allow.is_empty() && primary_allow.iter().any(|v| v.as_str() != Some("*")) {
                        acct_obj["allowFrom"] = json!(primary_allow);
                    }
                }
            }
        }
    }
    if let Some(sm) = &account.stream_mode {
        acct_obj["streamMode"] = json!(sm);
    }
    // Do NOT save primary to the account object (schema limit)
    // if let Some(pr) = account.primary {
    //    if pr { acct_obj["primary"] = json!(true); }
    // }

    // Update meta.primaryBotAccount
    // Update primaryBotAccount in manager.json (to avoid schema validation errors in Core)
    let mut manager_config = load_manager_config().unwrap_or(json!({}));
    
    if account.primary == Some(true) {
        manager_config["primaryBotAccount"] = json!(account_id);

        // --- NEW LOGIC DISABLED: Do NOT auto-create main agent or binding ---
        /*
        // 1. Ensure "main" agent exists pointing to ~/.openclaw/workspace
        let openclaw_home = platform::get_config_dir();
        // Resolve ~/.openclaw/workspace
        let main_workspace = std::path::Path::new(&openclaw_home).join("workspace");
        let main_workspace_str = main_workspace.to_string_lossy().to_string();

        let mut agents_list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            Vec::new()
        };

        let mut main_agent_exists = false;
        for agent in &mut agents_list {
            if agent.get("id").and_then(|v| v.as_str()) == Some("main") {
                main_agent_exists = true;
                // Ensure workspace is set correctly if it was missing or different?
                // For now, let's just assume if it exists, the user might have customized it.
                // But we should ensure the directory exists.
                if let Err(e) = std::fs::create_dir_all(&main_workspace) {
                     error!("[Telegram Accounts] Failed to create main workspace: {}", e);
                }
                break;
            }
        }

        if !main_agent_exists {
            info!("[Telegram Accounts] Creating 'main' agent for primary bot");
            // Create agentDir path: ~/.openclaw/agents/main/agent
            let main_agent_dir = std::path::Path::new(&openclaw_home).join("agents").join("main").join("agent");
            let main_agent_dir_str = main_agent_dir.to_string_lossy().to_string().replace('\\', "/");
            
            let main_agent = json!({
                "id": "main",
                "name": "General",
                "workspace": main_workspace_str,
                "agentDir": main_agent_dir_str,
                "default": true,
                "model": { "primary": "glm/glm-5" }
            });
            agents_list.push(main_agent);
            
            // Auto-create workspace directory
             if let Err(e) = std::fs::create_dir_all(&main_workspace) {
                 error!("[Telegram Accounts] Failed to create main workspace: {}", e);
            }
            // Auto-create agentDir and sessions directories
            let _ = std::fs::create_dir_all(&main_agent_dir);
            let sessions_dir = std::path::Path::new(&openclaw_home).join("agents").join("main").join("sessions");
            let _ = std::fs::create_dir_all(&sessions_dir);

            let soul_path = main_workspace.join("SOUL.md");
            if !soul_path.exists() {
                let root_soul = std::path::Path::new(&openclaw_home).join("SOUL.md");
                 if root_soul.exists() {
                     let _ = std::fs::copy(&root_soul, &soul_path);
                 } else {
                     let _ = std::fs::write(&soul_path, "# Primary Agent\n\nYou are the primary assistant.");
                 }
                 let _ = std::fs::write(main_workspace.join("AGENTS.md"), "# Agent Instructions\n\nBe helpful.");
                 let _ = std::fs::write(main_workspace.join("IDENTITY.md"), "name: Primary\nemoji: 🦞");
            }
            
            // Save updated agents list
             if config.get("agents").is_none() { config["agents"] = json!({}); }
            config["agents"]["list"] = json!(agents_list);
        }

        // 2. Ensure binding exists: main -> account.id
        let mut bindings = if let Some(arr) = config.get("bindings").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            Vec::new()
        };
        
        // Remove any existing binding for "main" agent to avoid duplicates/conflicts?
        // Or check if it already points to this account.
        let mut binding_exists = false;
        for b in &mut bindings {
            if b.get("agentId").and_then(|v| v.as_str()) == Some("main") {
                // Update existing binding to point to this account
                 if let Some(m) = b.get_mut("match").and_then(|v| v.as_object_mut()) {
                     m.insert("accountId".to_string(), json!(account.id));
                     m.insert("channel".to_string(), json!("telegram"));
                 }
                 binding_exists = true;
                 break;
            }
        }

        if !binding_exists {
            info!("[Telegram Accounts] Binding 'main' agent to primary bot");
            bindings.push(json!({
                "agentId": "main",
                "match": {
                    "channel": "telegram",
                    "accountId": account.id
                }
            }));
        }
        config["bindings"] = json!(bindings);
        */
        // --- END NEW LOGIC ---

    } else {
        // If we are saving this account and it is NOT primary, check if it WAS the primary account
        let current_primary = manager_config.pointer("/primaryBotAccount").and_then(|v| v.as_str());
        if current_primary == Some(account_id.as_str()) {
            if let Some(obj) = manager_config.as_object_mut() {
                obj.remove("primaryBotAccount");
            }
        }
    }
    
    if let Err(e) = save_manager_config(&manager_config) {
        error!("[Telegram Accounts] Failed to save manager config: {}", e);
        // Continue anyway, as we still want to save the account config
    }

    // Clean up legacy location in openclaw.json
    if let Some(meta) = config.get_mut("meta").and_then(|v| v.as_object_mut()) {
        meta.remove("primaryBotAccount");
    }

    // Handle groups configuration
    // If exclusive_topics is set, we need to modify the group config to enforce it
    // 1. Set group-level requireMention = true (default behavior: ignore everything)
    // 2. Set topic-level requireMention = false for whitelisted topics (exception: auto-reply)
    let mut groups_json = account.groups.clone();
    
    if let Some(exclusive_topics) = &account.exclusive_topics {
        if !exclusive_topics.is_empty() {
             // We also save the raw list so the UI can reload it (using a hidden field or relying on inference)
             // However, OpenClaw core rejects unknown fields. So we must ONLY output valid config.
             // Strategy: The UI will need to infer exclusive topics from the config structure if we can't save the field.
             // OR: We save it as a comment? No, JSON doesn't support comments.
             // COMPROMISE: We will NOT save "exclusiveTopics" to the file to avoid validation errors.
             // The UI will have to populate the field by checking if a group has topics configured.
             // For now, let's just apply the logic to the groups logic.

            if let Some(groups_map) = groups_json.as_mut().and_then(|g| g.as_object_mut()) {
                for (_, group_val) in groups_map.iter_mut() {
                    if let Some(group_obj) = group_val.as_object_mut() {
                        // Enforce whitelist logic:
                        // 1. Group requires mention (mute general)
                        group_obj.insert("requireMention".to_string(), json!(true));
                        group_obj.insert("enabled".to_string(), json!(true));

                        // 2. Allow specific topics
                        let mut topics_map = serde_json::Map::new();
                        for topic_id in exclusive_topics {
                            let mut topic_config = serde_json::Map::new();
                            topic_config.insert("requireMention".to_string(), json!(false));
                            topics_map.insert(topic_id.clone(), json!(topic_config));
                        }

                        // 3. Explicitly block topics owned by OTHER bot accounts
                        //    This prevents cross-talk when OpenClaw core doesn't
                        //    fall back to group-level requireMention for unlisted topics.
                        if let Some(all_accts) = config.pointer("/channels/telegram/accounts").and_then(|v| v.as_object()) {
                            for (other_id, other_val) in all_accts {
                                if other_id == &account.id { continue; }
                                if let Some(other_groups) = other_val.get("groups").and_then(|g| g.as_object()) {
                                    for (_, other_group) in other_groups {
                                        if let Some(other_topics) = other_group.get("topics").and_then(|t| t.as_object()) {
                                            for (other_tid, _) in other_topics {
                                                if !exclusive_topics.contains(other_tid) && !topics_map.contains_key(other_tid) {
                                                    let mut block_config = serde_json::Map::new();
                                                    block_config.insert("requireMention".to_string(), json!(true));
                                                    topics_map.insert(other_tid.clone(), json!(block_config));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        group_obj.insert("topics".to_string(), json!(topics_map));
                    }
                }
            }
        }
    }

    if let Some(g) = groups_json {
        acct_obj["groups"] = g;
    }

    // NOTE: We do NOT save "exclusiveTopics" field to avoid schema validation errors in OpenClaw core.
    // The UI state for this field might be lost on restart unless we infer it back from the topics structure,
    // but the *behavior* will be correct.
    // Remove any old keys with different casing to prevent duplicates
    // e.g. if "Chronos" exists and we're saving as "chronos", remove "Chronos"
    if let Some(accts) = config.pointer_mut("/channels/telegram/accounts").and_then(|v| v.as_object_mut()) {
        let old_keys: Vec<String> = accts.keys()
            .filter(|k| k.to_lowercase().replace(' ', "-") == account_id && *k != &account_id)
            .cloned()
            .collect();
        for old_key in old_keys {
            info!("[Telegram Accounts] Removing old key '{}' (normalized to '{}')", old_key, account_id);
            accts.remove(&old_key);
        }
    }

    config["channels"]["telegram"]["accounts"][&account_id] = acct_obj;

    // Ensure telegram is enabled and in plugins
    config["channels"]["telegram"]["enabled"] = json!(true);
    if config.get("plugins").is_none() {
        config["plugins"] = json!({ "allow": ["telegram"], "entries": { "telegram": { "enabled": true } } });
    }

    save_openclaw_config(&config)?;
    Ok(format!("Account '{}' saved", account_id))
}

/// Delete a Telegram bot account
#[command]
pub async fn delete_telegram_account(account_id: String) -> Result<String, String> {
    let account_id = account_id.to_lowercase().replace(' ', "-");
    info!("[Telegram Accounts] Deleting account: {}", account_id);
    let mut config = load_openclaw_config()?;

    if let Some(accts) = config.pointer_mut("/channels/telegram/accounts").and_then(|v| v.as_object_mut()) {
        accts.remove(&account_id);
    }

    // Also clean up any bindings referencing this account
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.pointer("/match/accountId").and_then(|v| v.as_str()) != Some(&account_id));
    }

    save_openclaw_config(&config)?;
    Ok(format!("Account '{}' deleted", account_id))
}

// ============ Feishu Plugin Management ============

/// Feishu plugin status
#[derive(Debug, Serialize, Deserialize)]
pub struct FeishuPluginStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub plugin_name: Option<String>,
}

/// Check if Feishu plugin is installed
#[command]
pub async fn check_feishu_plugin() -> Result<FeishuPluginStatus, String> {
    info!("[Feishu Plugin] Checking Feishu plugin installation status...");

    // Execute openclaw plugins list command
    match shell::run_openclaw(&["plugins", "list"]) {
        Ok(output) => {
            debug!("[Feishu Plugin] plugins list output: {}", output);

            // Find line containing feishu (case-insensitive)
            let lines: Vec<&str> = output.lines().collect();
            let feishu_line = lines.iter().find(|line| {
                line.to_lowercase().contains("feishu")
            });

            if let Some(line) = feishu_line {
                info!("[Feishu Plugin] Feishu plugin installed: {}", line);

                // Try to parse version number (usually format is "name@version" or "name version")
                let version = if line.contains('@') {
                    line.split('@').last().map(|s| s.trim().to_string())
                } else {
                    // Try to match version number pattern (e.g. 0.1.2)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    parts.iter()
                        .find(|p| p.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                        .map(|s| s.to_string())
                };

                Ok(FeishuPluginStatus {
                    installed: true,
                    version,
                    plugin_name: Some(line.trim().to_string()),
                })
            } else {
                info!("[Feishu Plugin] Feishu plugin not installed");
                Ok(FeishuPluginStatus {
                    installed: false,
                    version: None,
                    plugin_name: None,
                })
            }
        }
        Err(e) => {
            warn!("[Feishu Plugin] Failed to check plugin list: {}", e);
            // If command fails, assume plugin is not installed
            Ok(FeishuPluginStatus {
                installed: false,
                version: None,
                plugin_name: None,
            })
        }
    }
}

/// Install Feishu plugin
#[command]
pub async fn install_feishu_plugin() -> Result<String, String> {
    info!("[Feishu Plugin] Starting Feishu plugin installation...");

    // First check if already installed
    let status = check_feishu_plugin().await?;
    if status.installed {
        info!("[Feishu Plugin] Feishu plugin already installed, skipping");
        return Ok(format!("Feishu plugin already installed: {}", status.plugin_name.unwrap_or_default()));
    }

    // Install Feishu plugin
    // Note: Using @m1heng-clawd/feishu package name
    info!("[Feishu Plugin] Executing openclaw plugins install @m1heng-clawd/feishu ...");
    match shell::run_openclaw(&["plugins", "install", "@m1heng-clawd/feishu"]) {
        Ok(output) => {
            info!("[Feishu Plugin] Installation output: {}", output);

            // Verify installation result
            let verify_status = check_feishu_plugin().await?;
            if verify_status.installed {
                info!("[Feishu Plugin] Feishu plugin installed successfully");
                Ok(format!("Feishu plugin installed successfully: {}", verify_status.plugin_name.unwrap_or_default()))
            } else {
                warn!("[Feishu Plugin] Installation command succeeded but plugin not found");
                Err("Installation command succeeded but plugin not found, please check openclaw version".to_string())
            }
        }
        Err(e) => {
            error!("[Feishu Plugin] Installation failed: {}", e);
            Err(format!("Failed to install Feishu plugin: {}\n\nPlease run manually: openclaw plugins install @m1heng-clawd/feishu", e))
        }
    }
}

// ============ OpenClaw Home Directory ============

/// Get the OpenClaw home directory path (~/.openclaw)
#[command]
pub async fn get_openclaw_home_dir() -> Result<String, String> {
    Ok(platform::get_config_dir())
}

// ============ Multi-Agent Routing ============

/// Agent configuration for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: Option<String>,
    pub workspace: Option<String>,
    #[serde(alias = "agentDir", alias = "agent_dir")]
    pub agent_dir: Option<String>,
    pub model: Option<String>,
    pub sandbox: Option<bool>,
    pub heartbeat: Option<String>,
    pub default: Option<bool>,
    pub subagents: Option<SubagentConfig>,
}

/// Per-agent subagent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentConfig {
    #[serde(alias = "allowAgents", alias = "allow_agents")]
    pub allow_agents: Option<Vec<String>>,
}

/// Global subagent defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubagentDefaults {
    #[serde(alias = "maxSpawnDepth", alias = "max_spawn_depth")]
    pub max_spawn_depth: Option<u32>,
    #[serde(alias = "maxChildrenPerAgent", alias = "max_children_per_agent")]
    pub max_children_per_agent: Option<u32>,
    #[serde(alias = "maxConcurrent", alias = "max_concurrent")]
    pub max_concurrent: Option<u32>,
}

/// Agent binding rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    #[serde(alias = "agentId", alias = "agent_id")]
    pub agent_id: String,
    #[serde(alias = "matchRule", alias = "match_rule")]
    pub match_rule: MatchRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    pub channel: Option<String>,
    #[serde(alias = "accountId", alias = "account_id")]
    pub account_id: Option<String>,
    pub peer: Option<serde_json::Value>,
}

/// Combined agents config for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfigResponse {
    pub agents: Vec<AgentInfo>,
    pub bindings: Vec<AgentBinding>,
    pub subagent_defaults: SubagentDefaults,
}

/// Get multi-agent routing configuration
#[command]
pub async fn get_agents_config() -> Result<AgentsConfigResponse, String> {
    info!("[Agents] Getting agents configuration...");
    let config = load_openclaw_config()?;

    let mut agents = Vec::new();
    let mut bindings = Vec::new();

    // Read agents.list — supports both array format (correct) and object format (legacy)
    if let Some(list_arr) = config.pointer("/agents/list").and_then(|v| v.as_array()) {
        // Correct format: array of { id, workspace, agentDir, model, ... }
        for agent_val in list_arr {
            agents.push(AgentInfo {
                id: agent_val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: agent_val.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                workspace: agent_val.get("workspace").and_then(|v| v.as_str()).map(|s| s.to_string()),
                agent_dir: agent_val.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
                model: agent_val.pointer("/model/primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sandbox: agent_val.get("sandbox").and_then(|v| v.as_bool()),
                heartbeat: agent_val.pointer("/heartbeat/every").and_then(|v| v.as_str()).map(|s| s.to_string()),
                default: agent_val.get("default").and_then(|v| v.as_bool()),
                subagents: agent_val.get("subagents").and_then(|v| {
                    let allow = v.get("allowAgents").and_then(|a| a.as_array()).map(|arr| {
                        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
                    });
                    Some(SubagentConfig { allow_agents: allow })
                }),
            });
        }
    } else if let Some(list_obj) = config.pointer("/agents/list").and_then(|v| v.as_object()) {
        // Legacy format: object with id as keys
        for (id, agent_val) in list_obj {
            agents.push(AgentInfo {
                id: id.clone(),
                name: agent_val.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                workspace: agent_val.get("workspace").and_then(|v| v.as_str()).map(|s| s.to_string()),
                agent_dir: agent_val.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
                model: agent_val.pointer("/model/primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sandbox: agent_val.get("sandbox").and_then(|v| v.as_bool()),
                heartbeat: agent_val.pointer("/heartbeat/every").and_then(|v| v.as_str()).map(|s| s.to_string()),
                default: agent_val.get("default").and_then(|v| v.as_bool()),
                subagents: agent_val.get("subagents").and_then(|v| {
                    let allow = v.get("allowAgents").and_then(|a| a.as_array()).map(|arr| {
                        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
                    });
                    Some(SubagentConfig { allow_agents: allow })
                }),
            });
        }
    }

    // Read bindings — check top-level first (correct), then agents.bindings (legacy)
    let bindings_arr = config.get("bindings").and_then(|v| v.as_array())
        .or_else(|| config.pointer("/agents/bindings").and_then(|v| v.as_array()));
    
    if let Some(bindings_arr) = bindings_arr {
        for binding_val in bindings_arr {
            let empty_match = json!({});
            let match_obj = binding_val.get("match").unwrap_or(&empty_match);
            
            bindings.push(AgentBinding {
                agent_id: binding_val.get("agentId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                match_rule: MatchRule {
                    channel: match_obj.get("channel").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    account_id: match_obj.get("accountId").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    peer: match_obj.get("peer").cloned(),
                }
            });
        }
    }

    // Read global subagent defaults from agents.defaults.subagents
    let subagent_defaults = if let Some(sub_val) = config.pointer("/agents/defaults/subagents") {
        SubagentDefaults {
            max_spawn_depth: sub_val.get("maxSpawnDepth").and_then(|v| v.as_u64()).map(|v| v as u32),
            max_children_per_agent: sub_val.get("maxChildrenPerAgent").and_then(|v| v.as_u64()).map(|v| v as u32),
            max_concurrent: sub_val.get("maxConcurrent").and_then(|v| v.as_u64()).map(|v| v as u32),
        }
    } else {
        SubagentDefaults::default()
    };

    info!("[Agents] Found {} agents, {} bindings", agents.len(), bindings.len());
    Ok(AgentsConfigResponse { agents, bindings, subagent_defaults })
}

/// Save (add/update) an agent
#[command]
pub async fn save_agent(agent: AgentInfo) -> Result<String, String> {
    info!("[Agents] Saving agent: {}", agent.id);
    let mut config = load_openclaw_config()?;

    // Ensure agents object exists
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }

    // Build agent object (array element format with "id" field)
    let mut agent_obj = json!({ "id": agent.id });
    if let Some(name) = &agent.name {
        if !name.is_empty() {
            agent_obj["name"] = json!(name);
        }
    }
    if let Some(workspace) = &agent.workspace {
        if !workspace.is_empty() {
            agent_obj["workspace"] = json!(workspace);
        }
    }
    if let Some(agent_dir) = &agent.agent_dir {
        if !agent_dir.is_empty() {
            agent_obj["agentDir"] = json!(agent_dir);
        }
    }
    if let Some(model) = &agent.model {
        if !model.is_empty() {
            agent_obj["model"] = json!({ "primary": model });
        }
    }
    if let Some(sandbox) = agent.sandbox {
        agent_obj["sandbox"] = json!(sandbox);
    }
    if let Some(heartbeat) = &agent.heartbeat {
        if !heartbeat.is_empty() {
            agent_obj["heartbeat"] = json!({ "every": heartbeat });
        }
    }
    if let Some(is_default) = agent.default {
        if is_default {
            agent_obj["default"] = json!(true);
        }
    }
    if let Some(sub) = &agent.subagents {
        if let Some(allow) = &sub.allow_agents {
            if !allow.is_empty() {
                agent_obj["subagents"] = json!({ "allowAgents": allow });
            }
        }
    }

    // Migrate legacy object format to array if needed
    let mut list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
        arr.clone()
    } else if let Some(obj) = config["agents"].get("list").and_then(|v| v.as_object()) {
        // Convert legacy object to array
        obj.iter().map(|(id, val)| {
            let mut entry = val.clone();
            entry["id"] = json!(id);
            entry
        }).collect()
    } else {
        Vec::new()
    };

    // For NEW agents: use `openclaw agents add <id> --workspace <dir>` to create proper directory structure
    // The --workspace flag is required to make the CLI non-interactive
    let is_new_agent = !list.iter().any(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent.id));
    let mut cli_error: Option<String> = None;
    let is_reserved_name = agent.id.eq_ignore_ascii_case("main"); // Check if name is "main" to bypass CLI
    
    if is_new_agent {
        if !is_reserved_name {
            let openclaw_home = platform::get_config_dir();
            let workspace_dir = if let Some(ws) = &agent.workspace {
                ws.clone()
            } else if agent.default == Some(true) {
                std::path::Path::new(&openclaw_home).join("workspace").to_string_lossy().to_string()
            } else {
                std::path::Path::new(&openclaw_home).join(format!("workspace-{}", agent.id)).to_string_lossy().to_string()
            };
            
            info!("[Agents] New agent '{}' — running `openclaw agents add --workspace {}`", agent.id, workspace_dir);
            match shell::run_openclaw(&["agents", "add", &agent.id, "--workspace", &workspace_dir]) {
                Ok(output) => {
                    info!("[Agents] openclaw agents add succeeded: {}", output);
                }
                Err(e) => {
                    // NOTE: The CLI may exit with code 1 due to TUI stdin issues in non-interactive mode,
                    // but it still writes the agent entry to openclaw.json successfully.
                    warn!("[Agents] openclaw agents add exited with error (may still have written config): {}", e);
                    cli_error = Some(e);
                }
            }
            
            // CRITICAL: Always reload config after CLI runs — it may have written the entry
            config = load_openclaw_config()?;
            list = if let Some(arr) = config["agents"].get("list").and_then(|v| v.as_array()) {
                arr.clone()
            } else if let Some(obj) = config["agents"].get("list").and_then(|v| v.as_object()) {
                obj.iter().map(|(id, val)| {
                    let mut entry = val.clone();
                    entry["id"] = json!(id);
                    entry
                }).collect()
            } else {
                Vec::new()
            };
        } else {
             info!("[Agents] Skipping CLI for reserved name '{}', will create manually.", agent.id);
        }
    }

    // Find agent in list (handle case-insensitive match if CLI normalized the ID, e.g. AgentTest -> agenttest)
    let match_index = list.iter().position(|a| {
        a.get("id").and_then(|v| v.as_str()) == Some(&agent.id)
    }).or_else(|| {
        list.iter().position(|a| {
             a.get("id").and_then(|v| v.as_str()).map(|s| s.to_lowercase()) == Some(agent.id.to_lowercase())
        })
    });

    // Helper closure to create agent directories
    let ensure_directories = |agent_entry: &serde_json::Value| {
        let openclaw_home = platform::get_config_dir();
        
        // 1. Agent Config Directory
        // Use configured 'agentDir' or default to ~/.openclaw/agents/<id>/agent
        // The CLI standard is to have the agent files inside an `agent` subdirectory
        let agent_dir_path = if let Some(dir) = agent_entry.get("agentDir").and_then(|v| v.as_str()) {
             std::path::PathBuf::from(dir)
        } else {
             let id = agent_entry.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
             std::path::Path::new(&openclaw_home).join("agents").join(id).join("agent")
        };
        
        if !agent_dir_path.exists() {
             info!("[Agents] Creating agent directory: {:?}", agent_dir_path);
             let _ = std::fs::create_dir_all(&agent_dir_path);
        }
        
        // SOUL.md
        let soul_path = agent_dir_path.join("SOUL.md");
        if !soul_path.exists() {
             info!("[Agents] SOUL.md missing, creating default");
             let name = agent_entry.get("name").and_then(|v| v.as_str()).unwrap_or("agent");
             let default_soul = format!("You are {}, a helpful AI assistant.", name);
             let _ = std::fs::write(soul_path, default_soul);
        }

        // models.json
        let models_path = agent_dir_path.join("models.json");
        if !models_path.exists() {
             info!("[Agents] models.json missing, creating default");
             let default_models = json!({
                "providers": {
                    "glm": {
                        "baseUrl": "https://open.bigmodel.cn/api/paas/v4",
                        "apiKey": "",
                        "models": [ 
                            {
                                "id": "glm-4",
                                "name": "GLM-4",
                                "api": "openai-completions",
                                "reasoning": false,
                                "input": ["text", "image"],
                                "contextWindow": 128000,
                                "maxTokens": 8192
                            }
                        ]
                    }
                }
             });
             // Pretty print the JSON
             if let Ok(content) = serde_json::to_string_pretty(&default_models) {
                 let _ = std::fs::write(models_path, content);
             }
        }
        
        // 2. Workspace Directory
        // Use configured 'workspace' or default to ~/.openclaw/workspace-<id>
        let workspace_path = if let Some(ws) = agent_entry.get("workspace").and_then(|v| v.as_str()) {
             std::path::PathBuf::from(ws)
        } else {
             let id = agent_entry.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
             std::path::Path::new(&openclaw_home).join(format!("workspace-{}", id))
        };
        
        if !workspace_path.exists() {
             info!("[Agents] Creating workspace directory: {:?}", workspace_path);
             let _ = std::fs::create_dir_all(&workspace_path);
        }
        
        // Return paths to update config if they were defaults
        (agent_dir_path.to_string_lossy().to_string(), workspace_path.to_string_lossy().to_string())
    };

    // Update or add the agent
    if let Some(idx) = match_index {
        let existing = &mut list[idx];
        
        // Merge: only overwrite fields the user explicitly set (non-empty)
        if let Some(name) = &agent.name {
            if !name.is_empty() {
                existing["name"] = json!(name);
            }
        }
        if let Some(model) = &agent.model {
            if !model.is_empty() {
                existing["model"] = json!({ "primary": model });
            }
        }
        if let Some(is_default) = agent.default {
            if is_default {
                existing["default"] = json!(true);
            }
        }
        
        // Enforce "Main" agent properties
        if agent.id.eq_ignore_ascii_case("main") {
            // "Main" should always be default unless user explicitly sets another default (which handles itself)
            // But to ensure fallback behavior, we mark it.
            existing["default"] = json!(true);
        }

        if let Some(sub) = &agent.subagents {
            if let Some(allow) = &sub.allow_agents {
                if !allow.is_empty() {
                    existing["subagents"] = json!({ "allowAgents": allow });
                }
            }
        }
        if let Some(sandbox) = agent.sandbox {
            existing["sandbox"] = json!(sandbox);
        }
        if let Some(heartbeat) = &agent.heartbeat {
            if !heartbeat.is_empty() {
                existing["heartbeat"] = json!({ "every": heartbeat });
            }
        }
        
        // Repair directories for existing agent
        let _ = ensure_directories(existing);
        
    } else {
        // Not found in config (New agent, manual addition)
        
        // If we tried to create it via CLI and it's missing (and NOT reserved), that means CLI strictly failed.
        if let Some(err) = cli_error {
             if !is_reserved_name {
                 return Err(format!("Failed to create agent via CLI: {}. Check logs or name uniqueness.", err));
             }
        }

        // Add to list
        let mut new_entry = agent_obj.clone();
        
        // Ensure directories and get default paths if we need to explicitly save them
        let (actual_agent_dir, actual_workspace) = ensure_directories(&new_entry);
        
        // If user didn't specify paths, save the defaults we just used/created
        if new_entry.get("agentDir").is_none() {
             new_entry["agentDir"] = json!(actual_agent_dir);
        }
        if new_entry.get("workspace").is_none() {
             new_entry["workspace"] = json!(actual_workspace);
        }
        
        list.push(new_entry);
    }

    config["agents"]["list"] = json!(list);

    // Auto-create binding if a Telegram bot account is available and this agent has no binding yet
    let agent_id = agent.id.clone();
    let available_accounts: Vec<String> = config.pointer("/channels/telegram/accounts")
        .and_then(|v| v.as_object())
        .map(|accts| accts.keys().cloned().collect())
        .unwrap_or_default();

    if !available_accounts.is_empty() {
        // Check if this agent already has ANY binding
        let has_existing_binding = config.get("bindings")
            .and_then(|v| v.as_array())
            .map(|bindings| bindings.iter().any(|b| {
                b.get("agentId").and_then(|v| v.as_str()) == Some(&agent_id)
            }))
            .unwrap_or(false);

        if !has_existing_binding {
            // Find accounts already bound to other agents
            let bound_accounts: Vec<String> = config.get("bindings")
                .and_then(|v| v.as_array())
                .map(|bindings| bindings.iter().filter_map(|b| {
                    b.get("match").and_then(|m| m.get("accountId")).and_then(|v| v.as_str()).map(|s| s.to_string())
                }).collect())
                .unwrap_or_default();

            // Prefer: exact match > substring match > first unbound account > first account
            let best_account = available_accounts.iter()
                .find(|a| **a == agent_id) // exact match
                .or_else(|| available_accounts.iter().find(|a| a.contains(&agent_id) || agent_id.contains(a.as_str()))) // substring
                .or_else(|| available_accounts.iter().find(|a| !bound_accounts.contains(a))) // unbound
                .or_else(|| available_accounts.first()) // fallback
                .cloned();

            if let Some(account_id) = best_account {
                info!("[Agents] Auto-creating binding for agent '{}' → account '{}'", agent_id, account_id);
                if config.get("bindings").is_none() {
                    config["bindings"] = json!([]);
                }
                if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
                    bindings.push(json!({
                        "agentId": agent_id,
                        "match": { "channel": "telegram", "accountId": account_id }
                    }));
                }
            }
        }
    }

    save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' saved", agent.id))
}

/// Save global subagent defaults
#[command]
pub async fn save_subagent_defaults(defaults: SubagentDefaults) -> Result<String, String> {
    info!("[Agents] Saving subagent defaults");
    let mut config = load_openclaw_config()?;

    // Ensure agents.defaults exists
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("defaults").is_none() {
        config["agents"]["defaults"] = json!({});
    }

    let mut sub_obj = json!({});
    if let Some(depth) = defaults.max_spawn_depth {
        sub_obj["maxSpawnDepth"] = json!(depth);
    }
    if let Some(children) = defaults.max_children_per_agent {
        sub_obj["maxChildrenPerAgent"] = json!(children);
    }
    if let Some(concurrent) = defaults.max_concurrent {
        sub_obj["maxConcurrent"] = json!(concurrent);
    }

    config["agents"]["defaults"]["subagents"] = sub_obj;

    save_openclaw_config(&config)?;
    Ok("Subagent defaults saved".to_string())
}

/// Delete an agent
#[command]
pub async fn delete_agent(agent_id: String) -> Result<String, String> {
    info!("[Agents] Deleting agent: {}", agent_id);
    let mut config = load_openclaw_config()?;

    // 1. Find the agent to get its paths (before deleting from config)
    let mut agent_dir_to_delete: Option<String> = None;
    let mut workspace_to_delete: Option<String> = None;

    if let Some(list) = config.pointer("/agents/list").and_then(|v| v.as_array()) {
        if let Some(agent) = list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)) {
            // Get agent directory
            if let Some(dir) = agent.get("agentDir").and_then(|v| v.as_str()) {
                agent_dir_to_delete = Some(dir.to_string());
            }
            // Get workspace directory
            if let Some(ws) = agent.get("workspace").and_then(|v| v.as_str()) {
                workspace_to_delete = Some(ws.to_string());
            } else {
                // Fallback: deduce workspace path if default pattern was used
                let openclaw_home = platform::get_config_dir();
                let default_ws = std::path::Path::new(&openclaw_home).join(format!("workspace-{}", agent_id));
                if default_ws.exists() {
                    workspace_to_delete = Some(default_ws.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2. Delete the files (if they exist)
    // We do this BEFORE updating config, but we don't abort if it fails (just warn)
    // because we still want to remove the broken/stale entry from config.

    if let Some(agent_dir) = agent_dir_to_delete {
        let path = std::path::Path::new(&agent_dir);
        // Check if this is a nested 'agent' directory (standard structure: .../agents/<id>/agent)
        // If so, we want to delete the PARENT directory (e.g. .../agents/<id>) to clean up everything including sessions.
        let path_to_remove = if path.ends_with("agent") {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        if path_to_remove.exists() {
            info!("[Agents] Removing agent directory tree: {:?}", path_to_remove);
            if let Err(e) = std::fs::remove_dir_all(path_to_remove) {
                warn!("[Agents] Failed to remove agent directory {:?}: {}", path_to_remove, e);
            }
        }
    } else {
        // Fallback: try default location if not specified in config
        let openclaw_home = platform::get_config_dir();
        // Default structure is now ~/.openclaw/agents/<id> (which contains agent/, sessions/, etc.)
        let default_agent_root = std::path::Path::new(&openclaw_home).join("agents").join(&agent_id);
        
        if default_agent_root.exists() {
             info!("[Agents] Removing default agent directory tree: {:?}", default_agent_root);
             if let Err(e) = std::fs::remove_dir_all(&default_agent_root) {
                warn!("[Agents] Failed to remove default agent directory: {}", e);
            }
        }
    }

    if let Some(workspace) = workspace_to_delete {
        let path = std::path::Path::new(&workspace);
        if path.exists() {
            info!("[Agents] Removing workspace directory: {}", workspace);
            if let Err(e) = std::fs::remove_dir_all(path) {
                warn!("[Agents] Failed to remove workspace directory {}: {}", workspace, e);
            }
        }
    }

    // 3. Remove from agents.list (array format)
    if let Some(list) = config.pointer_mut("/agents/list").and_then(|v| v.as_array_mut()) {
        list.retain(|a| a.get("id").and_then(|v| v.as_str()) != Some(&agent_id));
    }

    // Remove related bindings (top-level)
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.get("agentId").and_then(|v| v.as_str()) != Some(&agent_id));
    }
    // Also clean legacy agents.bindings
    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.get("agentId").and_then(|v| v.as_str()) != Some(&agent_id));
    }

    save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' and its files were deleted", agent_id))
}

/// Save an agent binding rule
#[command]

pub async fn save_agent_binding(binding: AgentBinding) -> Result<String, String> {
    info!("[Agents] Saving binding for agent: {}", binding.agent_id);
    let mut config = load_openclaw_config()?;

    // Ensure top-level bindings array exists
    if config.get("bindings").is_none() {
        config["bindings"] = json!([]);
    }

    // Migrate legacy agents.bindings to top-level if present
    if let Some(legacy) = config.pointer("/agents/bindings").and_then(|v| v.as_array()).map(|a| a.clone()) {
        if let Some(top) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
            for b in legacy {
                top.push(b);
            }
        }
        // Remove legacy location
        if let Some(agents) = config.get_mut("agents").and_then(|v| v.as_object_mut()) {
            agents.remove("bindings");
        }
    }

    let mut match_obj = json!({});
    if let Some(ch) = &binding.match_rule.channel {
        if !ch.is_empty() { match_obj["channel"] = json!(ch); }
    }
    if let Some(acc) = &binding.match_rule.account_id {
        if !acc.is_empty() { match_obj["accountId"] = json!(acc); }
    }
    if let Some(peer) = &binding.match_rule.peer {
        match_obj["peer"] = peer.clone();
    }

    let binding_obj = json!({
        "agentId": binding.agent_id,
        "match": match_obj
    });

    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        bindings.push(binding_obj);
    }

    save_openclaw_config(&config)?;
    Ok(format!("Binding for agent '{}' saved", binding.agent_id))
}

/// Delete an agent binding by index
#[command]
pub async fn delete_agent_binding(index: usize) -> Result<String, String> {
    info!("[Agents] Deleting binding at index: {}", index);
    let mut config = load_openclaw_config()?;

    // Try top-level bindings first (correct location)
    if let Some(bindings) = config.get_mut("bindings").and_then(|v| v.as_array_mut()) {
        if index < bindings.len() {
            bindings.remove(index);
            save_openclaw_config(&config)?;
            return Ok(format!("Binding at index {} deleted", index));
        } else {
            return Err(format!("Binding index {} out of range", index));
        }
    }

    // Fallback to legacy agents.bindings
    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        if index < bindings.len() {
            bindings.remove(index);
            save_openclaw_config(&config)?;
            return Ok(format!("Binding at index {} deleted", index));
        } else {
            return Err(format!("Binding index {} out of range", index));
        }
    }

    Err("No bindings found".to_string())
}

// ============ Agent Soul / Personality ============

/// Read the personality (SOUL.md) for an agent
#[command]
pub async fn get_agent_system_prompt(agent_id: String, workspace: Option<String>) -> Result<String, String> {
    let base = workspace.unwrap_or_else(|| platform::get_config_dir());
    let sep = if cfg!(windows) { "\\" } else { "/" };
    
    // Resolve agent directory from config to handle case where ID != dir name
    let config = load_openclaw_config().map_err(|e| e.to_string())?;
    let agent_dir_rel = config.pointer("/agents/list")
        .and_then(|v| v.as_array())
        .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)))
        .and_then(|agent| agent.get("agentDir").and_then(|v| v.as_str()))
        .map(|s| s.replace("/", sep)) //normalize separators
        .unwrap_or_else(|| format!("agents{}{}", sep, agent_id)); // fallback

    // If agentDir is already an absolute path, use it directly; otherwise join with base
    let dir_config = if std::path::Path::new(&agent_dir_rel).is_absolute() {
        agent_dir_rel
    } else {
        format!("{}{}{}", base, sep, agent_dir_rel)
    };
    
    // Try locations in order of likelihood - prioritizing the CORRECT one first
    let paths = vec![
        format!("{}{}SOUL.md", dir_config, sep),                                // 1. agents/{id}/SOUL.md (CORRECT)
        format!("{}{}{}{}{}{}SOUL.md", base, sep, "agent", sep, agent_id, sep), // 2. agent/{id}/SOUL.md (Legacy/Buggy)
        format!("{}{}agent{}SOUL.md", dir_config, sep, sep),                    // 3. agents/{id}/agent/SOUL.md (Legacy/Buggy)
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            info!("[Agents] Found SOUL.md at: {}", path);
            return std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read SOUL.md: {}", e));
        }
    }
    
    Ok(String::new())
}

/// Save the personality (SOUL.md) for an agent
#[command]
pub async fn save_agent_system_prompt(agent_id: String, workspace: Option<String>, content: String) -> Result<String, String> {
    let base = workspace.unwrap_or_else(|| platform::get_config_dir());
    let sep = if cfg!(windows) { "\\" } else { "/" };
    
    // Resolve agent directory from config
    let config = load_openclaw_config().map_err(|e| e.to_string())?;
    let agent_dir_rel = config.pointer("/agents/list")
        .and_then(|v| v.as_array())
        .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(&agent_id)))
        .and_then(|agent| agent.get("agentDir").and_then(|v| v.as_str()))
        .map(|s| s.replace("/", sep))
        .unwrap_or_else(|| format!("agents{}{}", sep, agent_id));

    // If agentDir is already an absolute path, use it directly; otherwise join with base
    let dir_config = if std::path::Path::new(&agent_dir_rel).is_absolute() {
        agent_dir_rel
    } else {
        format!("{}{}{}", base, sep, agent_dir_rel)
    };
    
    // ONLY save to the correct canonical path
    let path = format!("{}{}SOUL.md", dir_config, sep);

    if let Some(parent) = std::path::Path::new(&path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory for {}: {}", path, e));
        }
    }
    
    match std::fs::write(&path, &content) {
        Ok(_) => {
            info!("[Agents] Wrote SOUL.md to: {}", path);
            Ok(format!("Personality (SOUL.md) saved for agent '{}'", agent_id))
        },
        Err(e) => Err(format!("Failed to save SOUL.md to {}: {}", path, e))
    }
}

/// Test agent routing: given an account ID, find which agent handles it
#[command]
pub async fn test_agent_routing(account_id: String) -> Result<serde_json::Value, String> {
    let config = load_openclaw_config()?;

    // Walk through bindings to find a match
    let bindings = config.get("bindings").and_then(|v| v.as_array());

    if let Some(bindings) = bindings {
        let empty_match = json!({});
        for binding in bindings {
            let match_obj = binding.get("match").unwrap_or(&empty_match);
            let binding_account = match_obj.get("accountId").and_then(|v| v.as_str());
            let binding_channel = match_obj.get("channel").and_then(|v| v.as_str());

            // Check if this binding matches
            let account_matches = binding_account.map(|a| a == account_id).unwrap_or(true); // None = catch-all
            let channel_matches = binding_channel.map(|c| c == "telegram").unwrap_or(true);

            if account_matches && channel_matches {
                let agent_id = binding.get("agentId").and_then(|v| v.as_str()).unwrap_or("unknown");

                // Find agent details
                let agent_info = config.pointer("/agents/list")
                    .and_then(|v| v.as_array())
                    .and_then(|list| list.iter().find(|a| a.get("id").and_then(|v| v.as_str()) == Some(agent_id)));

                // Read SOUL.md preview (try all 3 locations)
                let base = platform::get_config_dir();
                let sep = if cfg!(windows) { "\\" } else { "/" };
                let agent_dir_rel = agent_info.and_then(|a| a.get("agentDir").and_then(|v| v.as_str()))
                    .map(|s| s.replace("/", sep))
                    .unwrap_or_else(|| format!("agents{}{}", sep, agent_id));
                
                let dir_config = format!("{}{}{}", base, sep, agent_dir_rel);
                let check_paths = vec![
                    format!("{}{}{}{}{}{}SOUL.md", base, sep, "agent", sep, agent_id, sep),
                    format!("{}{}agent{}SOUL.md", dir_config, sep, sep),
                    format!("{}{}SOUL.md", dir_config, sep),
                ];
                
                let mut prompt_preview = String::new();
                for path in check_paths {
                    if std::path::Path::new(&path).exists() {
                        prompt_preview = std::fs::read_to_string(&path).unwrap_or_default();
                        break;
                    }
                }
                let prompt_preview = if prompt_preview.len() > 200 {
                    format!("{}...", &prompt_preview[..200])
                } else {
                    prompt_preview
                };

                return Ok(json!({
                    "matched": true,
                    "agent_id": agent_id,
                    "agent_dir": agent_info.and_then(|a| a.get("agentDir").and_then(|v| v.as_str())),
                    "model": agent_info.and_then(|a| a.pointer("/model/primary").and_then(|v| v.as_str())),
                    "system_prompt_preview": prompt_preview,
                    "binding": binding
                }));
            }
        }
    }

    Ok(json!({
        "matched": false,
        "agent_id": "default",
        "message": "No specific binding found. Messages will be handled by the default agent."
    }))
}

// ============ Heartbeat & Compaction ============

/// Heartbeat configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub every: Option<String>,
    pub target: Option<String>,
}

/// Compaction configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub enabled: bool,
    pub threshold: Option<u32>,
    pub context_pruning: bool,
    pub max_context_messages: Option<u32>,
}

/// Get heartbeat configuration
#[command]
pub async fn get_heartbeat_config() -> Result<HeartbeatConfig, String> {
    info!("[Heartbeat] Getting heartbeat config...");
    let config = load_openclaw_config()?;

    let every = config.pointer("/agents/defaults/heartbeat/every")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let target = config.pointer("/agents/defaults/heartbeat/target")
        .and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(HeartbeatConfig { every, target })
}

/// Save heartbeat configuration
#[command]
pub async fn save_heartbeat_config(every: Option<String>, target: Option<String>) -> Result<String, String> {
    info!("[Heartbeat] Saving heartbeat config: every={:?}, target={:?}", every, target);
    let mut config = load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    if every.is_some() || target.is_some() {
        let mut hb = json!({});
        if let Some(e) = &every { hb["every"] = json!(e); }
        if let Some(t) = &target { hb["target"] = json!(t); }
        config["agents"]["defaults"]["heartbeat"] = hb;
    } else {
        // Remove heartbeat if both are None
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("heartbeat");
        }
    }

    save_openclaw_config(&config)?;
    Ok("Heartbeat configuration saved".to_string())
}

/// Get compaction configuration
#[command]
pub async fn get_compaction_config() -> Result<CompactionConfig, String> {
    info!("[Compaction] Getting compaction config...");
    let config = load_openclaw_config()?;

    let compaction_val = config.pointer("/agents/defaults/compaction");
    let pruning_val = config.pointer("/agents/defaults/contextPruning");

    let enabled = compaction_val.map(|v| {
        // compaction can be true/false or an object with settings
        v.as_bool().unwrap_or(true)
    }).unwrap_or(false);

    let threshold = compaction_val
        .and_then(|v| v.get("threshold"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let context_pruning = pruning_val.map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);

    let max_context_messages = pruning_val
        .and_then(|v| v.get("maxMessages"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    Ok(CompactionConfig { enabled, threshold, context_pruning, max_context_messages })
}

/// Save compaction configuration
#[command]
pub async fn save_compaction_config(
    enabled: bool,
    threshold: Option<u32>,
    context_pruning: bool,
    max_context_messages: Option<u32>,
) -> Result<String, String> {
    info!("[Compaction] Saving compaction config: enabled={}, pruning={}", enabled, context_pruning);
    let mut config = load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    if enabled {
        let mut comp = json!({});
        if let Some(t) = threshold { comp["threshold"] = json!(t); }
        config["agents"]["defaults"]["compaction"] = comp;
    } else {
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("compaction");
        }
    }

    if context_pruning {
        let mut pruning = json!(true);
        if let Some(max) = max_context_messages {
            pruning = json!({ "maxMessages": max });
        }
        config["agents"]["defaults"]["contextPruning"] = pruning;
    } else {
        if let Some(defaults) = config["agents"]["defaults"].as_object_mut() {
            defaults.remove("contextPruning");
        }
    }

    save_openclaw_config(&config)?;
    Ok("Compaction configuration saved".to_string())
}

// ============ Workspace & Agent Personality ============

/// Workspace configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub workspace: Option<String>,
    pub timezone: Option<String>,
    pub time_format: Option<String>,
    pub skip_bootstrap: bool,
    pub bootstrap_max_chars: Option<u32>,
}

/// Get workspace configuration
#[command]
pub async fn get_workspace_config() -> Result<WorkspaceConfig, String> {
    info!("[Workspace] Getting workspace config...");
    let config = load_openclaw_config()?;

    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let timezone = config.pointer("/agents/defaults/timezone")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let time_format = config.pointer("/agents/defaults/timeFormat")
        .and_then(|v| v.as_str()).map(|s| s.to_string());
    let skip_bootstrap = config.pointer("/agents/defaults/skipBootstrap")
        .and_then(|v| v.as_bool()).unwrap_or(false);
    let bootstrap_max_chars = config.pointer("/agents/defaults/bootstrapMaxChars")
        .and_then(|v| v.as_u64()).map(|v| v as u32);

    Ok(WorkspaceConfig { workspace, timezone, time_format, skip_bootstrap, bootstrap_max_chars })
}

/// Save workspace configuration
#[command]
pub async fn save_workspace_config(
    workspace: Option<String>,
    timezone: Option<String>,
    time_format: Option<String>,
    skip_bootstrap: bool,
    bootstrap_max_chars: Option<u32>,
) -> Result<String, String> {
    info!("[Workspace] Saving workspace config...");
    let mut config = load_openclaw_config()?;

    if config.get("agents").is_none() { config["agents"] = json!({}); }
    if config["agents"].get("defaults").is_none() { config["agents"]["defaults"] = json!({}); }

    let defaults = config["agents"]["defaults"].as_object_mut().unwrap();

    // Set or remove each field
    match &workspace {
        Some(w) if !w.is_empty() => { defaults.insert("workspace".into(), json!(w)); }
        _ => { defaults.remove("workspace"); }
    }
    match &timezone {
        Some(tz) if !tz.is_empty() => { defaults.insert("timezone".into(), json!(tz)); }
        _ => { defaults.remove("timezone"); }
    }
    match &time_format {
        Some(tf) if !tf.is_empty() => { defaults.insert("timeFormat".into(), json!(tf)); }
        _ => { defaults.remove("timeFormat"); }
    }
    if skip_bootstrap {
        defaults.insert("skipBootstrap".into(), json!(true));
    } else {
        defaults.remove("skipBootstrap");
    }
    match bootstrap_max_chars {
        Some(max) => { defaults.insert("bootstrapMaxChars".into(), json!(max)); }
        None => { defaults.remove("bootstrapMaxChars"); }
    }

    save_openclaw_config(&config)?;
    Ok("Workspace configuration saved".to_string())
}

/// Get a personality file from the workspace directory
#[command]
pub async fn get_personality_file(filename: String) -> Result<String, String> {
    info!("[Personality] Reading file: {}", filename);

    // Validate filename
    let allowed = ["AGENTS.md", "SOUL.md", "TOOLS.md"];
    if !allowed.contains(&filename.as_str()) {
        return Err(format!("Invalid file: {}. Allowed: {:?}", filename, allowed));
    }

    // Get workspace path from config, fallback to ~/.openclaw
    let config = load_openclaw_config()?;
    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let dir = if workspace.is_empty() {
        platform::get_config_dir()
    } else {
        workspace.to_string()
    };

    let filepath = if platform::is_windows() {
        format!("{}\\{}", dir, filename)
    } else {
        format!("{}/{}", dir, filename)
    };

    match file::read_file(&filepath) {
        Ok(content) => Ok(content),
        Err(_) => Ok(String::new()), // File doesn't exist yet, return empty
    }
}

/// Save a personality file to the workspace directory
#[command]
pub async fn save_personality_file(filename: String, content: String) -> Result<String, String> {
    info!("[Personality] Saving file: {}", filename);

    let allowed = ["AGENTS.md", "SOUL.md", "TOOLS.md"];
    if !allowed.contains(&filename.as_str()) {
        return Err(format!("Invalid file: {}. Allowed: {:?}", filename, allowed));
    }

    let config = load_openclaw_config()?;
    let workspace = config.pointer("/agents/defaults/workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let dir = if workspace.is_empty() {
        platform::get_config_dir()
    } else {
        workspace.to_string()
    };

    let filepath = if platform::is_windows() {
        format!("{}\\{}", dir, filename)
    } else {
        format!("{}/{}", dir, filename)
    };

    file::write_file(&filepath, &content)
        .map_err(|e| format!("Failed to save {}: {}", filename, e))?;

    Ok(format!("{} saved successfully", filename))
}

// ============ Browser Control ============

/// Browser configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub enabled: bool,
    pub color: Option<String>,
}

/// Get browser configuration
#[command]
pub async fn get_browser_config() -> Result<BrowserConfig, String> {
    info!("[Browser] Getting browser config...");
    let config = load_openclaw_config()?;

    // Read from meta (Manager specific)
    let enabled = config.pointer("/meta/gui/browser/enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to true if not set

    let color = config.pointer("/meta/gui/browser/color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(BrowserConfig { enabled, color })
}

/// Save browser configuration
#[command]
pub async fn save_browser_config(enabled: bool, color: Option<String>) -> Result<String, String> {
    info!("[Browser] Saving browser config: enabled={}, color={:?}", enabled, color);
    let mut config = load_openclaw_config()?;

    // Store in meta.gui.browser to avoid polluting core config
    if config.get("meta").is_none() { config["meta"] = json!({}); }
    if config["meta"].get("gui").is_none() { config["meta"]["gui"] = json!({}); }
    
    let mut browser_config = json!({
        "enabled": enabled
    });

    if let Some(c) = color {
        if !c.is_empty() {
            browser_config["color"] = json!(c);
        }
    }

    config["meta"]["gui"]["browser"] = browser_config;

    save_openclaw_config(&config)?;
    Ok("Browser configuration saved".to_string())
}

// ============ Web Search ============

/// Web Search configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub brave_api_key: Option<String>,
}

/// Get web search configuration
#[command]
pub async fn get_web_config() -> Result<WebConfig, String> {
    info!("[Web] Getting web search config...");
    let config = load_openclaw_config()?;

    let brave_api_key = config.pointer("/web/braveApiKey")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(WebConfig { brave_api_key })
}

/// Save web search configuration
#[command]
pub async fn save_web_config(brave_api_key: Option<String>) -> Result<String, String> {
    info!("[Web] Saving web search config...");
    let mut config = load_openclaw_config()?;

    if config.get("web").is_none() {
        config["web"] = json!({});
    }

    match brave_api_key {
        Some(key) if !key.is_empty() => {
            config["web"]["braveApiKey"] = json!(key);
        }
        _ => {
            if let Some(web) = config.get_mut("web").and_then(|v| v.as_object_mut()) {
                web.remove("braveApiKey");
            }
        }
    }

    save_openclaw_config(&config)?;
    Ok("Web search configuration saved".to_string())
}
