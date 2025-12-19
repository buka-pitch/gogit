use futures_util::StreamExt;
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("API Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Environment variable missing: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Stream error: {0}")]
    Stream(String),
}

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    pub model: String,
}

impl GeminiClient {
    pub fn new(model: String) -> Result<Self, AiError> {
        let api_key = env::var("GEMINI_API_KEY")?;
        Ok(Self {
            client: Client::new(),
            api_key,
            model,
        })
    }

    pub async fn stream_completion(
        &self,
        prompt: &str,
        system_instruction: &str,
    ) -> Result<impl futures_util::Stream<Item = Result<String, AiError>>, AiError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "system_instruction": {
                "parts": [{ "text": system_instruction }]
            },
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 8192
            }
        });

        // Retry Loop for 429
        let mut attempts = 0;
        let max_retries = 3;
        
        loop {
            let resp = self.client
                .post(&url)
                .json(&body)
                .send()
                .await?;

            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                if attempts >= max_retries {
                    return Err(AiError::Stream("Rate limit exceeded after retries".to_string()));
                }
                attempts += 1;
                eprintln!("\n[Rate Limit] Waiting 10s before retry ({}/{})...", attempts, max_retries);
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            if !resp.status().is_success() {
                 return Err(AiError::Stream(format!("API Error: {}", resp.status())));
            }

            let stream = resp.bytes_stream();
            let mut stream = Box::pin(stream);

            // Buffer for incomplete JSON chunks
            let mut buffer = String::new();

            return Ok(async_stream::try_stream! {
                while let Some(item) = stream.next().await {
                    let bytes = item?;
                    let chunk_s = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&chunk_s);
                    
                    // Attempt to parse accumulated buffer
                    // Naive strategy: Check if it looks like a complete JSON array item or object
                    // In a robust impl we'd use a parser. 
                    // Here we look for balanced braces if we suspect it's an object, 
                    // or just rely on the API returning valid JSON objects per line/chunk mostly.
                    
                    // Simple hygiene:
                    // The Gemini stream usually returns `[{}, {}, ...]`
                    // We try to clean up the array brackets.
                    
                    let clean = buffer.trim();
                    if let Some(start) = clean.find('{') {
                         if let Some(end) = clean.rfind('}') {
                             if end > start {
                                 // We have a candidate object
                                 let potential_json = &clean[start..=end];
                                 
                                 match serde_json::from_str::<serde_json::Value>(potential_json) {
                                    Ok(json) => {
                                        // Success! Clear buffer up to this point... actually this is streaming.
                                        // If we matched, we consume the buffer.
                                        buffer.clear(); 
                                        
                                        if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                                            yield text.to_string();
                                        } else if let Some(err) = json.get("error") {
                                            let msg = err["message"].as_str().unwrap_or("Unknown error");
                                            eprintln!("\n[API Error] {}", msg);
                                        }
                                    },
                                    Err(_) => {
                                        // Not complete yet, keep buffering
                                    }
                                 }
                             }
                         }
                    }
                }
            });
        }
    }

    pub async fn generate_text(&self, prompt: &str, system_instruction: &str) -> Result<String, AiError> {
         let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
             "system_instruction": {
                "parts": [{ "text": system_instruction }]
            },
            "safetySettings": [
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_ONLY_HIGH" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_ONLY_HIGH" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_ONLY_HIGH" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_ONLY_HIGH" }
            ]
        });

        // Retry logic for 429
        let mut attempts = 0;
        loop {
            let resp = self.client
                .post(&url)
                .json(&body)
                .send()
                .await?;
                
            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                if attempts >= 3 {
                    return Err(AiError::Stream("Rate limit exceeded".to_string()));
                }
                attempts += 1;
                sleep(Duration::from_secs(5)).await;
                continue;
            }

            let json: serde_json::Value = resp.json().await?;
            
            if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                return Ok(text.to_string());
            } else if let Some(error) = json.get("error") {
                let msg = error["message"].as_str().unwrap_or("Unknown API error");
                return Err(AiError::Stream(format!("Gemini API Error: {}", msg)));
            } else if let Some(candidates) = json.get("candidates") {
                if !candidates.as_array().map_or(false, |a| !a.is_empty()) {
                    return Err(AiError::Stream("Gemini returned no candidates. This usually means the prompt was blocked by safety filters.".to_string()));
                }
                let finish_reason = candidates[0].get("finishReason").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                return Err(AiError::Stream(format!("Gemini failed to generate text. Finish reason: {}", finish_reason)));
            } else {
                return Err(AiError::Stream(format!("Unexpected JSON response from Gemini: {}", json)));
            }
        }
    }

    pub async fn smart_diff_summary(&self, diff: &str) -> Result<String, AiError> {
        // Gemini 1.5+ has 1M+ token context. 
        // 500,000 chars is roughly 125k tokens, well within limits.
        // We only want to map-reduce if it's TRULY massive to avoid rate limits.
        if diff.len() < 500_000 {
            return Ok(diff.to_string());
        }
        let parts: Vec<&str> = diff.split("diff --git").collect();
        let mut summaries = Vec::new();
        for part in parts {
            if part.trim().is_empty() { continue; }
            let chunk = format!("diff --git{}", part);
            let summary = self.generate_text(
                &format!("Summarize the code changes in this git diff chunk:\n\n{}", chunk),
                "You are a code summarizer. Output a concise summary of changes."
            ).await?;
            summaries.push(summary);
        }
        Ok(summaries.join("\n\n"))
    }

    pub async fn generate_git_command(&self, description: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Translate this natural language request into a valid git command:\n\n\"{}\"\n\nOutput ONLY the command itself, no explanation, no backticks. Example: git log --author=\"John\"", description),
            "You are a Git expert. You translate natural language descriptions into precise git commands."
        ).await
    }

    pub async fn suggest_branch_name(&self, description: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Suggest a concise and descriptive git branch name for the following task:\n\n\"{}\"\n\nRules:\n- Use kebab-case.\n- Include a prefix like feat/, fix/, chore/, or docs/ if appropriate.\n- Output ONLY the branch name, no explanation, no backticks.", description),
            "You are a Git expert. You suggest clean and standard branch names based on task descriptions."
        ).await
    }

    pub async fn resolve_conflicts(&self, file_content: &str, file_name: &str) -> Result<String, AiError> {
        self.generate_text(
            &format!("Resolve the merge conflicts in the following file: {}\n\nContent:\n{}\n\nRules:\n- Maintain the intended logic from both sides where possible.\n- Remove all conflict markers (<<<<<<<, =======, >>>>>>>).\n- Output ONLY the resolved file content, no explanation, no backticks.", file_name, file_content),
            "You are a Senior Software Engineer. You resolve merge conflicts precisely, ensuring code integrity and following project conventions."
        ).await
    }
}
