use crate::setup::{read_instructions, Provider};
use serde_json::json;

fn extract_model_names(body: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();

    // OpenAI-compatible: {"data": [{"id": "..."}]}
    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        for m in data {
            if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                names.push(id.to_string());
            } else if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            }
        }
        if !names.is_empty() {
            return names;
        }
    }

    // Ollama / alternative: {"models": [{"name": "...", "model": "..."}]}
    if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
        for m in models {
            if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                names.push(id.to_string());
            } else if let Some(name) = m.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            } else if let Some(model) = m.get("model").and_then(|v| v.as_str()) {
                names.push(model.to_string());
            } else if let Some(s) = m.as_str() {
                names.push(s.to_string());
            }
        }
        if !names.is_empty() {
            return names;
        }
    }

    // Direct array
    if let Some(arr) = body.as_array() {
        for v in arr {
            if let Some(s) = v.as_str() {
                names.push(s.to_string());
            }
        }
    }

    names
}

pub fn auth_provider(
    provider_name: &str,
    provider_info: &Provider,
    api_key: &str,
    model: &str
) -> Result<(), String> {
    let is_ollama = provider_name == "ollama";

    if !is_ollama && api_key.trim().is_empty() {
        return Err(format!("API key is required for {}", provider_name));
    }

    let verify_model = !model.trim().is_empty();

    let client = reqwest::blocking::Client::new();

    // Ollama (Local) — no API key, verify via /api/tags directly
    let response = if is_ollama {
        client
            .get(format!("{}/api/tags", provider_info.endpoint.trim_end_matches('/')))
            .send()
            .map_err(|e| format!("Failed to connect to Ollama at {}: {}. Is Ollama running? (ollama serve)", provider_info.endpoint, e))?
    } else {
        client
            .get(&format!("{}/models", provider_info.endpoint))
            .bearer_auth(api_key)
            .send()
            .map_err(|e| format!("Failed to connect to {}: {}", provider_name, e))?
    };

    // Claude / providers without /models listing: 404 means we can't verify model list
    if response.status().as_u16() == 404 {
        if verify_model {
            println!("{} authentication successful (model '{}' not verified via API - listing not supported)", provider_name, model);
        } else {
            println!("{} authentication successful", provider_name);
        }
        return Ok(());
    }

    if response.status() == reqwest::StatusCode::UNAUTHORIZED || response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(format!(
            "Authentication failed for {}: {}",
            provider_name,
            response.status()
        ));
    }

    if !response.status().is_success() {
        return Err(format!(
            "Authentication failed for {}: {}",
            provider_name,
            response.status()
        ));
    }

    let body: serde_json::Value = response.json()
        .map_err(|e| format!("Failed to parse response from {}: {}", provider_name, e))?;

    if verify_model {
        let model_names = extract_model_names(&body);

        if !model_names.is_empty() {
            if !model_names.iter().any(|m| m == model) {
                return Err(format!(
                    "Model '{}' not found for {}. Available models: {:?}",
                    model, provider_name, model_names
                ));
            }
        } else {
            println!("{}: could not list models, skipping model verification for '{}'", provider_name, model);
        }

        println!("{} authentication successful with model '{}'", provider_name, model);
    } else {
        println!("{} authentication successful", provider_name);
    }
    Ok(())
}

pub fn generate_commit_message(
    diff: &str,
    provider_name: &str,
    provider_info: &Provider,
    model: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let system_prompt = read_instructions();

    let user_prompt = format!("Generate a commit message for this diff:\n\n{}", diff);

    // Ollama local — use /api/chat
    let (url, body, use_bearer) = if provider_name == "ollama" {
        let url = format!("{}/api/chat", provider_info.endpoint.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "stream": false
        });
        (url, body, false)
    } else if provider_name == "claude" {
        // Anthropic messages API
        let url = format!("{}/messages", provider_info.endpoint.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "max_tokens": 512,
            "system": system_prompt,
            "messages": [{"role": "user", "content": user_prompt}]
        });
        (url, body, false)
    } else {
        // OpenAI-compatible: /chat/completions (groq, openai, openrouter, gemini-openai)
        let url = format!("{}/chat/completions", provider_info.endpoint.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3
        });
        (url, body, true)
    };

    let mut req = client.post(&url).json(&body);

    if provider_name == "claude" {
        let key = provider_info.api_key.as_deref().unwrap_or("");
        req = req.header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");
    } else if use_bearer {
        let key = provider_info.api_key.as_deref().unwrap_or("");
        req = req.bearer_auth(key);
    }

    let resp = req
        .send()
        .map_err(|e| format!("Failed to call {}: {}", provider_name, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let txt = resp.text().unwrap_or_default();
        return Err(format!("LLM error {}: {} — {}", provider_name, status, txt));
    }

    let v: serde_json::Value = resp
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Parse per provider
    let msg = if provider_name == "ollama" {
        v.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .or_else(|| v.get("response").and_then(|c| c.as_str()))
            .map(|s| s.trim().to_string())
    } else if provider_name == "claude" {
        v.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|o| o.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.trim().to_string())
    } else {
        v.get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|o| o.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.trim().to_string())
    };

    match msg {
        Some(m) if !m.is_empty() => Ok(m),
        _ => Err(format!("Empty response from {}: {:?}", provider_name, v)),
    }
}
