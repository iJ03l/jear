//! NEAR AI Cloud chat JSON wire types.
//!
//! `serde` shapes for `POST /v1/chat/completions` (OpenAI-compatible) —
//! no HTTP calls yet. Includes usage for budget tracking.

use serde::{Deserialize, Serialize};

/// One chat message. `tool_calls` passes through untouched so the gate
/// never drops function calls the caller attached.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: `system`, `user`, or `assistant`.
    pub role: String,
    /// Message text.
    pub content: String,
    /// Pending tool calls from the assistant (forwarded verbatim).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

impl ChatMessage {
    /// Build a user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            tool_calls: Vec::new(),
        }
    }
}

/// A function tool definition, forwarded verbatim (OpenAI shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatTool {
    /// Always `"function"` for function tools.
    #[serde(rename = "type")]
    pub kind: String,
    /// The function signature.
    pub function: FunctionDef,
}

/// A callable function signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Function name.
    pub name: String,
    /// Human description (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema parameters (optional, kept as `Value`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// A pending tool call returned by the assistant, forwarded verbatim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call id for pairing with results.
    pub id: String,
    /// Always `"function"` for function calls.
    #[serde(rename = "type")]
    pub kind: String,
    /// The invoked function + JSON arguments string.
    pub function: CalledFunction,
}

/// An invoked function with raw JSON arguments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalledFunction {
    /// Function name.
    pub name: String,
    /// JSON-encoded arguments.
    pub arguments: String,
}

/// Chat completions request body (subset we send).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatRequest {
    /// Model id, e.g. `zai-org/GLM-5.1-FP8`.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Function tools the model may call (forwarded verbatim).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ChatTool>,
}

impl ChatRequest {
    /// Build a single-turn user request.
    #[must_use]
    pub fn new(model: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![ChatMessage::user(content)],
            tools: Vec::new(),
        }
    }

    /// Attach function tools (chainable).
    #[must_use]
    pub fn with_tools(mut self, tools: Vec<ChatTool>) -> Self {
        self.tools = tools;
        self
    }
}

/// Token usage for budget tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Usage {
    /// Input tokens billed.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Output tokens billed.
    #[serde(default)]
    pub completion_tokens: u64,
    /// Total tokens billed.
    #[serde(default)]
    pub total_tokens: u64,
}

/// One completion choice.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Choice {
    /// Choice index.
    pub index: u32,
    /// Assistant reply.
    pub message: ChatMessage,
    /// Why generation stopped.
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Chat completions response body (subset we route on).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChatResponse {
    /// Response id.
    pub id: String,
    /// Serving model.
    pub model: String,
    /// Completion choices.
    #[serde(default)]
    pub choices: Vec<Choice>,
    /// Token usage (may be absent on some proxies).
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl ChatResponse {
    /// First assistant text, if any.
    #[must_use]
    pub fn first_text(&self) -> Option<&str> {
        self.choices.first().map(|c| c.message.content.as_str())
    }
}

/// One catalog entry from `GET /v1/models` (subset we route on).
/// All fields are optional-tolerant: proxies omit what they lack.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelEntry {
    /// Model id, e.g. `zai-org/GLM-5.1-FP8`.
    pub id: String,
    /// Context window in tokens, when reported.
    #[serde(default)]
    pub context_length: Option<u64>,
    /// Max output tokens, when reported.
    #[serde(default)]
    pub max_output_length: Option<u64>,
    /// USD per 1M input tokens, when reported.
    #[serde(default)]
    pub input_price_per_mtok: Option<f64>,
    /// USD per 1M output tokens, when reported.
    #[serde(default)]
    pub output_price_per_mtok: Option<f64>,
    /// Tool calling support, when reported.
    #[serde(default)]
    pub supports_tools: bool,
    /// Reasoning support, when reported.
    #[serde(default)]
    pub supports_reasoning: bool,
}

impl ModelEntry {
    /// Estimate cost in cents for a token plan, or `None` when unpriced.
    #[must_use]
    pub fn estimated_cents(&self, in_tokens: u64, out_tokens: u64) -> Option<u64> {
        let (inp, outp) = match (self.input_price_per_mtok, self.output_price_per_mtok) {
            (Some(i), Some(o)) => (i, o),
            _ => return None,
        };
        let dollars = in_tokens as f64 * inp / 1_000_000.0 + out_tokens as f64 * outp / 1_000_000.0;
        Some((dollars * 100.0).round() as u64)
    }
}

/// Catalog response from `GET /v1/models` (list shape).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelsResponse {
    /// Catalog entries.
    #[serde(default)]
    pub data: Vec<ModelEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_openai_shape() {
        let v = serde_json::to_value(ChatRequest::new("zai-org/GLM-5.1-FP8", "Hello!"))
            .expect("serialize");
        assert_eq!(v["model"], "zai-org/GLM-5.1-FP8");
        assert_eq!(v["messages"][0]["role"], "user");
        assert_eq!(v["messages"][0]["content"], "Hello!");
    }

    #[test]
    fn response_deserializes_with_usage() {
        let raw = serde_json::json!({
            "id": "chatcmpl-1",
            "object": "chat.completion",
            "created": 1758240000,
            "model": "zai-org/GLM-5.1-FP8",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hi!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 8, "completion_tokens": 3, "total_tokens": 11}
        });
        let r: ChatResponse = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(r.first_text(), Some("Hi!"));
        let u = r.usage.expect("usage");
        assert_eq!(u.total_tokens, 11);
    }

    #[test]
    fn missing_usage_is_none() {
        let raw = serde_json::json!({
            "id": "chatcmpl-2",
            "model": "openai/gpt-5",
            "choices": []
        });
        let r: ChatResponse = serde_json::from_value(raw).expect("deserialize");
        assert!(r.usage.is_none());
        assert!(r.first_text().is_none());
    }

    #[test]
    fn tools_round_trip_verbatim() {
        let req =
            ChatRequest::new("zai-org/GLM-5.1-FP8", "What is 2+2?").with_tools(vec![ChatTool {
                kind: "function".to_string(),
                function: FunctionDef {
                    name: "calc".to_string(),
                    description: Some("Evaluate math".to_string()),
                    parameters: Some(serde_json::json!({"type": "object"})),
                },
            }]);
        let v = serde_json::to_value(&req).expect("serialize");
        assert_eq!(v["tools"][0]["type"], "function");
        assert_eq!(v["tools"][0]["function"]["name"], "calc");
        let raw = serde_json::json!({
            "id": "chatcmpl-3",
            "model": "zai-org/GLM-5.1-FP8",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "calc", "arguments": "{\"x\": 2}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });
        let r: ChatResponse = serde_json::from_value(raw).expect("deserialize");
        let call = &r.choices[0].message.tool_calls[0];
        assert_eq!(call.id, "call_1");
        assert_eq!(call.function.name, "calc");
    }

    #[test]
    fn catalog_entry_prices_estimate() {
        let raw = serde_json::json!({
            "id": "zai-org/GLM-5.1-FP8",
            "context_length": 128000,
            "max_output_length": 8000,
            "input_price_per_mtok": 0.20,
            "output_price_per_mtok": 1.00,
            "supports_tools": true,
            "supports_reasoning": false
        });
        let e: ModelEntry = serde_json::from_value(raw).expect("deserialize");
        assert_eq!(e.context_length, Some(128000));
        assert!(e.supports_tools);
        // 1000 in + 500 out at $0.20/$1.00 per MTok = $0.0007 = 0c rounded.
        assert_eq!(e.estimated_cents(1000, 500), Some(0));
        // 1M in + 1M out = $1.20 = 120c.
        assert_eq!(e.estimated_cents(1_000_000, 1_000_000), Some(120));
    }

    #[test]
    fn unpriced_entry_has_no_estimate() {
        let raw = serde_json::json!({"id": "mystery-model"});
        let e: ModelEntry = serde_json::from_value(raw.clone()).expect("deserialize");
        assert!(e.estimated_cents(1000, 1000).is_none());
        let list: ModelsResponse =
            serde_json::from_value(serde_json::json!({"data": [raw]})).expect("deserialize");
        assert_eq!(list.data.len(), 1);
    }
}
