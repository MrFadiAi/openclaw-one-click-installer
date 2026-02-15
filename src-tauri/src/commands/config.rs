use crate::models::{
    AIConfigOverview, ChannelConfig, ConfiguredModel, ConfiguredProvider,
    MCPConfig, ModelConfig, OfficialProvider, SuggestedModel,
};
use crate::utils::{file, platform, shell};
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
        serde_json::to_string_pretty(&config).unwrap_or_default()
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
    info!("[Gateway Token] Generated new Token: {}...", &new_token[..8]);

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

    info!("[Dashboard URL] URL: {}...", &url[..50.min(url.len())]);
    Ok(url)
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
                    id: "glm-4".to_string(),
                    name: "GLM-4".to_string(),
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

// ============ Multi-Agent Routing ============

/// Agent configuration for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub workspace: Option<String>,
    #[serde(rename = "agentDir")]
    pub agent_dir: Option<String>,
    pub model: Option<String>,
    pub sandbox: Option<bool>,
}

/// Agent binding rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub match_rule: MatchRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    pub channel: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub peer: Option<serde_json::Value>,
}

/// Combined agents config for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfigResponse {
    pub agents: Vec<AgentInfo>,
    pub bindings: Vec<AgentBinding>,
}

/// Get multi-agent routing configuration
#[command]
pub async fn get_agents_config() -> Result<AgentsConfigResponse, String> {
    info!("[Agents] Getting agents configuration...");
    let config = load_openclaw_config()?;

    let mut agents = Vec::new();
    let mut bindings = Vec::new();

    // Read agents.list
    if let Some(list) = config.pointer("/agents/list").and_then(|v| v.as_object()) {
        for (id, agent_val) in list {
            agents.push(AgentInfo {
                id: id.clone(),
                workspace: agent_val.get("workspace").and_then(|v| v.as_str()).map(|s| s.to_string()),
                agent_dir: agent_val.get("agentDir").and_then(|v| v.as_str()).map(|s| s.to_string()),
                model: agent_val.pointer("/model/primary").and_then(|v| v.as_str()).map(|s| s.to_string()),
                sandbox: agent_val.get("sandbox").and_then(|v| v.as_bool()),
            });
        }
    }

    // Read bindings
    if let Some(bindings_arr) = config.pointer("/agents/bindings").and_then(|v| v.as_array()) {
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

    info!("[Agents] Found {} agents, {} bindings", agents.len(), bindings.len());
    Ok(AgentsConfigResponse { agents, bindings })
}

/// Save (add/update) an agent
#[command]
pub async fn save_agent(agent: AgentInfo) -> Result<String, String> {
    info!("[Agents] Saving agent: {}", agent.id);
    let mut config = load_openclaw_config()?;

    // Ensure agents.list exists
    if config.pointer("/agents/list").is_none() {
        if config.get("agents").is_none() {
            config["agents"] = json!({});
        }
        config["agents"]["list"] = json!({});
    }

    let mut agent_obj = json!({});
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

    config["agents"]["list"][&agent.id] = agent_obj;
    save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' saved", agent.id))
}

/// Delete an agent
#[command]
pub async fn delete_agent(agent_id: String) -> Result<String, String> {
    info!("[Agents] Deleting agent: {}", agent_id);
    let mut config = load_openclaw_config()?;

    // Remove from agents.list
    if let Some(list) = config.pointer_mut("/agents/list").and_then(|v| v.as_object_mut()) {
        list.remove(&agent_id);
    }

    // Remove related bindings
    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        bindings.retain(|b| b.get("agentId").and_then(|v| v.as_str()) != Some(&agent_id));
    }

    save_openclaw_config(&config)?;
    Ok(format!("Agent '{}' deleted", agent_id))
}

/// Save an agent binding rule
#[command]

pub async fn save_agent_binding(binding: AgentBinding) -> Result<String, String> {
    info!("[Agents] Saving binding for agent: {}", binding.agent_id);
    let mut config = load_openclaw_config()?;

    // Ensure agents.bindings array exists
    if config.get("agents").is_none() {
        config["agents"] = json!({});
    }
    if config["agents"].get("bindings").is_none() {
        config["agents"]["bindings"] = json!([]);
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

    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
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

    if let Some(bindings) = config.pointer_mut("/agents/bindings").and_then(|v| v.as_array_mut()) {
        if index < bindings.len() {
            bindings.remove(index);
        } else {
            return Err(format!("Binding index {} out of range", index));
        }
    } else {
        return Err("No bindings found".to_string());
    }

    save_openclaw_config(&config)?;
    Ok(format!("Binding at index {} deleted", index))
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
