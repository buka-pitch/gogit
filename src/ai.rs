use futures_util::StreamExt;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

/// Represents an error that can occur during AI operations.
#[derive(Error, Debug)]
pub enum AiError {
    /// An error occurred during an API request.
    #[error("API Request failed: {0}")]
    Request(#[from] reqwest::Error),
    /// An environment variable was missing.
    #[error("Environment variable missing: {0}")]
    Env(#[from] std::env::VarError),
    /// An error occurred while processing the response stream.
    #[error("Stream error: {0}")]
    Stream(String),
    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Specific error for missing tool support
    #[error("Tool-use incompatibility: {0}")]
    ToolIncompatibility(String),
}

/// Response from OpenRouter models API
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

/// Information about an AI model
#[derive(Debug, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub pricing: Option<ModelPricing>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionDefinition,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// Gemini Native specific structures
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTool>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    thought_signature: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenRouter,
    Gemini,
}

/// A client for interacting with AI APIs (OpenRouter or Native Gemini).
#[derive(Clone)]
pub struct AiClient {
    client: Client,
    api_key: String,
    pub provider: AiProvider,
    /// The name of the model to use (e.g., "openai/gpt-4o-mini")
    pub model: String,
}

impl AiClient {
    /// Creates a new `AiClient`.
    pub fn new(
        model: String,
        provider_str: &str,
        config_openrouter_key: Option<String>,
        config_gemini_key: Option<String>,
    ) -> Result<Self, AiError> {
        let provider = match provider_str.to_lowercase().as_str() {
            "gemini" => AiProvider::Gemini,
            _ => AiProvider::OpenRouter,
        };

        let raw_api_key = match provider {
            AiProvider::OpenRouter => env::var("OPENROUTER_API_KEY")
                .or_else(|_| config_openrouter_key.ok_or(std::env::VarError::NotPresent)),
            AiProvider::Gemini => env::var("GEMINI_API_KEY")
                .or_else(|_| config_gemini_key.ok_or(std::env::VarError::NotPresent)),
        }
        .map_err(|_| {
            let key_name = if provider == AiProvider::Gemini {
                "GEMINI_API_KEY"
            } else {
                "OPENROUTER_API_KEY"
            };
            AiError::Stream(format!(
                "No API key found. Set {} or add it to config.toml",
                key_name
            ))
        })?;

        let api_key = raw_api_key.trim().to_string();

        if api_key.is_empty() {
            return Err(AiError::Stream("API Key is empty".to_string()));
        }

        Ok(Self {
            client: Client::new(),
            api_key,
            provider,
            model,
        })
    }

    fn get_base_url(&self) -> String {
        match self.provider {
            AiProvider::OpenRouter => "https://openrouter.ai/api/v1".to_string(),
            AiProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    /// Lists all available models from OpenRouter or Native Gemini list.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, AiError> {
        if self.provider == AiProvider::Gemini {
            return Ok(vec![
                ModelInfo {
                    id: "gemini-3-flash-preview".to_string(),
                    name: Some("Gemini 3 Flash Preview (Native)".to_string()),
                    description: None,
                    context_length: Some(250000),
                    pricing: Some(ModelPricing {
                        prompt: "0".to_string(),
                        completion: "0".to_string(),
                    }),
                },
                ModelInfo {
                    id: "gemma-3-27b".to_string(),
                    name: Some("gemma-3-27b".to_string()),
                    description: None,
                    context_length: Some(15000),
                    pricing: Some(ModelPricing {
                        prompt: "0".to_string(),
                        completion: "0".to_string(),
                    }),
                },
                ModelInfo {
                    id: "gemini-2.5-flash-lite".to_string(),
                    name: Some("Gemini 2.5 Flash LITE (Native)".to_string()),
                    description: None,
                    context_length: Some(250000),
                    pricing: Some(ModelPricing {
                        prompt: "0".to_string(),
                        completion: "0".to_string(),
                    }),
                },
                ModelInfo {
                    id: "gemini-2.5-flash-native-audio-dialog".to_string(),
                    name: Some("gemini-2.5-flash-native-audio-dialog".to_string()),
                    description: None,
                    context_length: Some(15000),
                    pricing: Some(ModelPricing {
                        prompt: "0".to_string(),
                        completion: "0".to_string(),
                    }),
                },
            ]);
        }

        let url = format!("{}/models", self.get_base_url());
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "gogit/0.1.0")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(AiError::Stream(format!("API Error: {}", resp.status())));
        }

        let models_response: ModelsResponse = resp.json().await?;
        Ok(models_response.data)
    }

    fn map_messages_to_gemini(
        &self,
        messages: &[Message],
    ) -> (Option<GeminiContent>, Vec<GeminiContent>) {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    if let Some(content) = &msg.content {
                        system_instruction = Some(GeminiContent {
                            role: None,
                            parts: vec![GeminiPart {
                                text: Some(content.clone()),
                                function_call: None,
                                function_response: None,
                            }],
                        });
                    }
                }
                Role::User => {
                    contents.push(GeminiContent {
                        role: Some("user".to_string()),
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                            function_call: None,
                            function_response: None,
                        }],
                    });
                }
                Role::Assistant => {
                    let mut parts = Vec::new();
                    if let Some(content) = &msg.content {
                        parts.push(GeminiPart {
                            text: Some(content.clone()),
                            function_call: None,
                            function_response: None,
                        });
                    }
                    if let Some(tool_calls) = &msg.tool_calls {
                        for call in tool_calls {
                            let thought_signature = call
                                .extra_content
                                .as_ref()
                                .and_then(|extra| extra.get("thought_signature"))
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string());

                            parts.push(GeminiPart {
                                text: None,
                                function_call: Some(GeminiFunctionCall {
                                    name: call.function.name.clone(),
                                    args: serde_json::from_str(&call.function.arguments)
                                        .unwrap_or(json!({})),
                                    thought_signature,
                                }),
                                function_response: None,
                            });
                        }
                    }
                    contents.push(GeminiContent {
                        role: Some("model".to_string()),
                        parts,
                    });
                }
                Role::Tool => {
                    // In gogit, we use tool_call_id to store the function name for Gemini Native link
                    contents.push(GeminiContent {
                        role: Some("user".to_string()),
                        parts: vec![GeminiPart {
                            text: None,
                            function_call: None,
                            function_response: Some(GeminiFunctionResponse {
                                name: msg.tool_call_id.clone().unwrap_or_default(),
                                response: json!({ "result": msg.content.clone().unwrap_or_default() }),
                            }),
                        }],
                    });
                }
            }
        }
        (system_instruction, contents)
    }

    fn map_tools_to_gemini(tools: &[ToolDefinition]) -> GeminiTool {
        let function_declarations = tools
            .iter()
            .map(|t| GeminiFunctionDeclaration {
                name: t.function.name.clone(),
                description: t.function.description.clone(),
                parameters: t.function.parameters.clone(),
            })
            .collect();
        GeminiTool {
            function_declarations,
        }
    }

    /// Lists only free models from OpenRouter (where pricing is 0).
    pub async fn list_free_models(&self) -> Result<Vec<ModelInfo>, AiError> {
        if self.provider == AiProvider::Gemini {
            return self.list_models().await;
        }

        let all_models = self.list_models().await?;
        let free_models: Vec<ModelInfo> = all_models
            .into_iter()
            .filter(|model| {
                if let Some(pricing) = &model.pricing {
                    pricing.prompt == "0" && pricing.completion == "0"
                } else {
                    false
                }
            })
            .collect();

        Ok(free_models)
    }

    /// Streams content generated by the AI API in response to a prompt.
    pub async fn stream_completion(
        &self,
        prompt: &str,
        system_instruction: &str,
    ) -> Result<impl futures_util::Stream<Item = Result<String, AiError>>, AiError> {
        let client = self.client.clone();
        let provider = self.provider;
        let model = self.model.clone();
        let api_key = self.api_key.clone();
        let base_url = self.get_base_url();
        let prompt = prompt.to_string();
        let system_instruction = system_instruction.to_string();

        Ok(async_stream::try_stream! {
            let (url, body) = if provider == AiProvider::Gemini {
                let url = format!("{}/models/{}:streamGenerateContent?alt=sse&key={}", base_url, model, api_key);
                let body = json!({
                    "contents": [{
                        "role": "user",
                        "parts": [{ "text": prompt }]
                    }],
                    "systemInstruction": {
                        "parts": [{ "text": system_instruction }]
                    }
                });
                (url, body)
            } else {
                let url = format!("{}/chat/completions", base_url);
                let body = json!({
                    "model": model,
                    "messages": [
                        { "role": "system", "content": system_instruction },
                        { "role": "user", "content": prompt }
                    ],
                    "stream": true,
                    "temperature": 0.7,
                    "max_tokens": 8192
                });
                (url, body)
            };

            let mut request = client.post(&url);
            if provider == AiProvider::OpenRouter {
                request = request
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("User-Agent", "gogit/0.1.0")
                    .header("HTTP-Referer", "gogit")
                    .header("X-Title", "gogit");
            }
            if provider == AiProvider::Gemini {
                request = request.header("x-goog-api-key",format!("{}",api_key))
            }
            let resp = request.json(&body).send().await?;
            if !resp.status().is_success() {
                Err(AiError::Stream(format!("API Error {}: {}", resp.status(), resp.text().await.unwrap_or_default())))?;
            } else {
                let stream = resp.bytes_stream();
                let mut stream = Box::pin(stream);

                while let Some(item) = stream.next().await {
                    let bytes = item?;
                    let chunk_str = String::from_utf8_lossy(&bytes);

                    for line in chunk_str.lines() {
                        if line.starts_with("data: ") {
                            let json_str = line.strip_prefix("data: ").unwrap();

                            if json_str == "[DONE]" { break; }

                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                                if provider == AiProvider::Gemini {
                                    if let Some(delta) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                        yield delta.to_string();
                                    }
                                } else {
                                    if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                                        yield delta.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    /// Generates a text completion from the AI API.
    pub async fn generate_text(
        &self,
        prompt: &str,
        system_instruction: &str,
    ) -> Result<String, AiError> {
        if self.provider == AiProvider::Gemini {
            let url = format!(
                "{}/models/{}:generateContent?key={}",
                self.get_base_url(),
                self.model,
                self.api_key
            );
            let body = json!({
                "contents": [{
                    "role": "user",
                    "parts": [{ "text": prompt }]
                }],
                "systemInstruction": {
                    "parts": [{ "text": system_instruction }]
                }
            });

            let resp = self.client.post(&url).json(&body).send().await?;
            if !resp.status().is_success() {
                return Err(AiError::Stream(format!(
                    "Gemini API Error {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }
            let res: GeminiResponse = resp.json().await?;
            return Ok(res.candidates[0].content.parts[0]
                .text
                .clone()
                .unwrap_or_default());
        }

        let url = format!("{}/chat/completions", self.get_base_url());
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_instruction
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 8192
        });

        let mut attempts = 0;
        let max_retries = 3;
        loop {
            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("User-Agent", "gogit/0.1.0")
                .header("HTTP-Referer", "https://github.com/buka-pitch/gogit")
                .header("X-Title", "gogit")
                .json(&body)
                .send()
                .await?;

            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                if attempts >= max_retries {
                    return Err(AiError::Stream("Rate limit exceeded".to_string()));
                }
                attempts += 1;
                let wait_secs = 10 * (2u64.pow(attempts as u32 - 1));
                sleep(Duration::from_secs(wait_secs)).await;
                continue;
            }

            let json: serde_json::Value = resp.json().await?;

            if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                return Ok(text.to_string());
            } else if let Some(error) = json.get("error") {
                let msg = error["message"].as_str().unwrap_or("Unknown API error");
                return Err(AiError::Stream(format!("API Error: {}", msg)));
            } else {
                return Err(AiError::Stream(format!(
                    "Unexpected JSON response: {}",
                    json
                )));
            }
        }
    }

    /// Generates a concise summary of code changes from a git diff.
    pub async fn smart_diff_summary(&self, diff: &str) -> Result<String, AiError> {
        if diff.len() < 500_000 {
            return Ok(diff.to_string());
        }
        let parts: Vec<&str> = diff.split("diff --git").collect();
        let mut summaries = Vec::new();
        for part in parts {
            if part.trim().is_empty() {
                continue;
            }
            let chunk = format!("diff --git{}", part);
            let summary = self
                .generate_text(
                    &format!(
                        "Summarize the code changes in this git diff chunk:\n\n{}",
                        chunk
                    ),
                    "You are a code summarizer. Output a concise summary of changes.",
                )
                .await?;
            summaries.push(summary);
        }
        Ok(summaries.join("\n\n"))
    }

    /// Translates a natural language description into a git command.
    pub async fn generate_git_command(&self, description: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Translate this natural language request into a valid git command:\n\n\"{}\"\n\nOutput ONLY the command itself, no explanation, no backticks. Example: git log --author=\"John\"", description),
            "You are a Git expert. You translate natural language descriptions into precise git commands."
        ).await
    }

    /// Suggests a git branch name based on a task description.
    pub async fn suggest_branch_name(&self, description: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Suggest a concise and descriptive git branch name for the following task:\n\n\"{}\"\n\nRules:\n- Use kebab-case.\n- Include a prefix like feat/, fix/, chore/, or docs/ if appropriate.\n- Output ONLY the branch name, no explanation, no backticks.", description),
            "You are a Git expert. You suggest clean and standard branch names based on task descriptions."
        ).await
    }

    /// Resolves merge conflicts in a given file content.
    pub async fn resolve_conflicts(
        &self,
        file_content: &str,
        file_name: &str,
    ) -> Result<String, AiError> {
        self.generate_text(
            &format!("Resolve the merge conflicts in the following file: {}\n\nContent:\n{}\n\nRules:\n- Maintain the intended logic from both sides where possible.\n- Remove all conflict markers (<<<<<<<, =======, >>>>>>>).\n- Output ONLY the resolved file content, no explanation, no backticks.", file_name, file_content),
            "You are a Senior Software Engineer. You resolve merge conflicts precisely, ensuring code integrity and following project conventions."
        ).await
    }

    /// Generates release notes in Markdown format based on a list of commits.
    pub async fn generate_release_notes(&self, commits: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Generate a professional and concise RELEASE_NOTES.md content based on these commits:\n\n{}\n\nStructure:\n- # [Version/Date]\n- ## 🚀 New Features\n- ## 🛠️ Bug Fixes\n- ## 🔒 Security\n- ## 📦 Internal Changes\n\n- Focus on the impact and value for stakeholders.\n- Output ONLY the Markdown content, no extra explanation.", commits),
            "You are a Project Manager and Senior Engineer. You write clear, impactful, and categorized release notes from raw commit logs."
        ).await
    }

    pub async fn answer_repo_question(
        &self,
        question: &str,
        files: &str,
        context: &str,
    ) -> Result<String, AiError> {
        self.generate_text(
            &format!("Question about the repository:\n\"{}\"\n\nProject Structure:\n{}\n\nRelevant Context/File Contents:\n{}\n\nInstructions:\n- Answer the question accurately based on the provided files and structure.\n- Explain 'where things are' and 'how things work'.\n- Be concise and helpful for a new developer onboarding.", question, files, context),
            "You are a Senior Architect. You act as a technical guide for the codebase, helping developers navigate and understand the repository."
        ).await
    }

    pub async fn analyze_branch_staleness(&self, branch_info: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Analyze these Git branches for staleness/redundancy:\n\n{}\n\nInstructions:\n- Identify which branches are safe to delete.\n- A branch is safe if its changes are likely already in the base branch or if it was a temporary fix.\n- Be conservative but helpful.\n- Output a clear, bulleted list with 'Safe to Delete' or 'Keep' for each branch, with a one-sentence reason.", branch_info),
            "You are a DevOps Engineer specializing in repository maintenance. You help teams keep their branches tidy and secure."
        ).await
    }

    /// Conducts a chat completion with support for tool calls.
    pub async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<Message, AiError> {
        if self.provider == AiProvider::Gemini {
            let url = format!(
                "{}/models/{}:generateContent?key={}",
                self.get_base_url(),
                self.model,
                self.api_key
            );
            let (system_instruction, contents) = self.map_messages_to_gemini(&messages);
            let gemini_tools = tools.as_ref().map(|t| vec![Self::map_tools_to_gemini(t)]);

            let body = GeminiRequest {
                contents,
                system_instruction,
                tools: gemini_tools,
            };

            let resp = self.client.post(&url).json(&body).send().await?;
            if !resp.status().is_success() {
                return Err(AiError::Stream(format!(
                    "Gemini API Error {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }

            let gemini_resp: GeminiResponse = resp.json().await?;
            let candidate = gemini_resp
                .candidates
                .get(0)
                .ok_or_else(|| AiError::Stream("No candidates in Gemini response".to_string()))?;

            let mut content = None;
            let mut tool_calls = Vec::new();

            for part in &candidate.content.parts {
                if let Some(text) = &part.text {
                    content = Some(text.clone());
                }
                if let Some(fc) = &part.function_call {
                    tool_calls.push(ToolCall {
                        id: fc.name.clone(),
                        tool_type: "function".to_string(),
                        function: ToolFunction {
                            name: fc.name.clone(),
                            arguments: fc.args.to_string(),
                        },
                        extra_content: fc
                            .thought_signature
                            .clone()
                            .map(|s| json!({ "thought_signature": s })),
                    });
                }
            }

            return Ok(Message {
                role: Role::Assistant,
                content,
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
                tool_call_id: None,
            });
        }

        let url = format!("{}/chat/completions", self.get_base_url());

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
        });

        let has_tools = if let Some(t) = tools {
            if !t.is_empty() {
                body["tools"] = json!(t);
                body["tool_choice"] = json!("auto");
                true
            } else {
                false
            }
        } else {
            false
        };

        let mut attempts = 0;
        let max_retries = 3;

        loop {
            let resp = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("User-Agent", "gogit/0.1.0")
                .header("HTTP-Referer", "https://github.com/buka-pitch/gogit")
                .header("X-Title", "gogit")
                .json(&body)
                .send()
                .await?;

            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                if attempts >= max_retries {
                    return Err(AiError::Stream(
                        "Rate limit exceeded after retries".to_string(),
                    ));
                }
                attempts += 1;
                let wait_secs = 10 * (2u64.pow(attempts as u32 - 1));
                eprintln!(
                    "\n[Rate Limit] Waiting {}s before retry ({}/{})...",
                    wait_secs, attempts, max_retries
                );
                sleep(Duration::from_secs(wait_secs)).await;
                continue;
            }

            if resp.status() == StatusCode::NOT_FOUND && has_tools {
                let text = resp.text().await.unwrap_or_default();
                if text.contains("tool use") || text.contains("endpoint") {
                    return Err(AiError::ToolIncompatibility(
                        "This model/provider does not support tool use.".to_string(),
                    ));
                }
                return Err(AiError::Stream(format!("API Error 404: {}", text)));
            }

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(AiError::Stream(format!("API Error {}: {}", status, text)));
            }

            let json: serde_json::Value = resp.json().await?;

            if let Some(choice) = json["choices"].get(0) {
                let msg_val = &choice["message"];
                let res_msg: Message = serde_json::from_value(msg_val.clone())?;
                return Ok(res_msg);
            } else if let Some(error) = json.get("error") {
                let msg = error["message"].as_str().unwrap_or("Unknown error");
                if has_tools && (msg.contains("tool") || msg.contains("support")) {
                    return Err(AiError::ToolIncompatibility(msg.to_string()));
                }
                return Err(AiError::Stream(format!("API Error: {}", msg)));
            } else {
                return Err(AiError::Stream("No response choices found".to_string()));
            }
        }
    }
}
