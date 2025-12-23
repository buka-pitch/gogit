use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::process::Command;
use walkdir::WalkDir;
use crate::ai::{ToolDefinition, ToolFunctionDefinition};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn get_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "read_file".to_string(),
                    description: "Read the contents of a file in the repository.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the file to read."
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "write_file".to_string(),
                    description: "Write content to a file in the repository.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The path to the file to write."
                            },
                            "content": {
                                "type": "string",
                                "description": "The content to write to the file."
                            }
                        },
                        "required": ["path", "content"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "list_directory".to_string(),
                    description: "List the contents of a directory (shallow).".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "The directory path (default is '.')."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "get_file_tree".to_string(),
                    description: "Get a recursive tree of files in the project.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Starting directory (default is '.')."
                            },
                            "depth": {
                                "type": "integer",
                                "description": "Maximum recursion depth (default 3)."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "web_search".to_string(),
                    description: "Search the web for information using Tavily or DuckDuckGo.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query."
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "read_url".to_string(),
                    description: "Read the text content of a URL.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to fetch."
                            }
                        },
                        "required": ["url"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "run_command".to_string(),
                    description: "Execute a shell command. Requires user confirmation.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute."
                            }
                        },
                        "required": ["command"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_status".to_string(),
                    description: "Get the current status of the git repository.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_diff".to_string(),
                    description: "Get the diff of current changes.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "staged": {
                                "type": "boolean",
                                "description": "Whether to show staged changes (default false)."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_add".to_string(),
                    description: "Add file contents to the git index.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "paths": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "List of paths to add (e.g. ['.'] or ['src/main.rs'])."
                            }
                        },
                        "required": ["paths"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_commit".to_string(),
                    description: "Record changes to the repository. Requires user confirmation.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "The commit message."
                            }
                        },
                        "required": ["message"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_log".to_string(),
                    description: "Show the commit logs.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of commits to show (default 5)."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "git_branch".to_string(),
                    description: "List, create, or delete branches.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": ["list", "create", "delete"],
                                "description": "The action to perform (default 'list')."
                            },
                            "name": {
                                "type": "string",
                                "description": "The branch name (required for create/delete)."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "search_code".to_string(),
                    description: "Search for a string or pattern in the codebase.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "pattern": {
                                "type": "string",
                                "description": "The pattern to search for."
                            },
                            "path": {
                                "type": "string",
                                "description": "Directory to search in (default '.')."
                            }
                        },
                        "required": ["pattern"]
                    }),
                },
            },
        ]
    }

    pub async fn call_tool(name: &str, args: &str) -> String {
        match name {
            "read_file" => Self::read_file(args).await,
            "write_file" => Self::write_file(args).await,
            "list_directory" => Self::list_directory(args).await,
            "get_file_tree" => Self::get_file_tree(args).await,
            "web_search" => Self::web_search(args).await,
            "read_url" => Self::read_url(args).await,
            "run_command" => Self::run_command(args).await,
            "git_status" => Self::git_status().await,
            "git_diff" => Self::git_diff(args).await,
            "git_add" => Self::git_add(args).await,
            "git_commit" => Self::git_commit(args).await,
            "git_log" => Self::git_log(args).await,
            "git_branch" => Self::git_branch(args).await,
            "search_code" => Self::search_code(args).await,
            _ => format!("Error: Tool '{}' not found.", name),
        }
    }

    async fn read_file(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or_default();
        match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => format!("Error reading file: {}", e),
        }
    }

    async fn write_file(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or_default();
        let content = val["content"].as_str().unwrap_or_default();
        
        match fs::write(path, content) {
            Ok(_) => format!("Successfully wrote to {}", path),
            Err(e) => format!("Error writing file: {}", e),
        }
    }

    async fn list_directory(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or(".");
        
        let mut entries = Vec::new();
        for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.depth() == 0 { continue; }
            let p = entry.path().display().to_string();
            let kind = if entry.file_type().is_dir() { "[DIR]" } else { "[FILE]" };
            entries.push(format!("{} {}", kind, p));
        }
        
        if entries.is_empty() {
            "Directory is empty or not found.".to_string()
        } else {
            entries.join("\n")
        }
    }

    async fn get_file_tree(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or(".");
        let depth = val["depth"].as_u64().unwrap_or(3) as usize;
        
        let mut entries = Vec::new();
        for entry in WalkDir::new(path).max_depth(depth).into_iter().filter_map(|e| e.ok()) {
            if entry.depth() == 0 { continue; }
            let p = entry.path().display().to_string();
            let kind = if entry.file_type().is_dir() { "[DIR]" } else { "[FILE]" };
            entries.push(format!("{} {}", kind, p));
        }
        
        if entries.is_empty() {
            "Empty tree.".to_string()
        } else {
            entries.join("\n")
        }
    }

    async fn web_search(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let query = val["query"].as_str().unwrap_or_default();

        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            let client = reqwest::Client::new();
            let body = json!({
                "api_key": api_key,
                "query": query,
                "search_depth": "basic",
                "max_results": 5
            });

            match client.post("https://api.tavily.com/search").json(&body).send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(results) = data["results"].as_array() {
                            let mut output = String::new();
                            for r in results {
                                output.push_str(&format!("### {}\nURL: {}\n{}\n\n", r["title"], r["url"], r["content"]));
                            }
                            return if output.is_empty() { "No results found.".to_string() } else { output };
                        }
                    }
                }
                Err(e) => eprintln!("Tavily error: {}", e),
            }
        }

        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        match reqwest::get(&url).await {
            Ok(resp) => {
                if let Ok(_text) = resp.text().await {
                    return format!("DuckDuckGo Search Results for '{}':\n(Parsing HTML is complex; Tavily key recommended for better results)\nURL: {}", query, url);
                }
            }
            Err(e) => return format!("Search failed: {}", e),
        }

        "Search unavailable.".to_string()
    }

    async fn read_url(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let url = val["url"].as_str().unwrap_or_default();

        match reqwest::get(url).await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(text) => {
                        if text.len() > 10000 {
                            format!("{}... (truncated)", &text[..10000])
                        } else {
                            text
                        }
                    }
                    Err(e) => format!("Error reading response text: {}", e),
                }
            }
            Err(e) => format!("Failed to fetch URL: {}", e),
        }
    }

    async fn run_command(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let command_str = val["command"].as_str().unwrap_or_default();
        
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", command_str]).output()
        } else {
            Command::new("sh").args(["-c", command_str]).output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
            }
            Err(e) => format!("Command failed: {}", e),
        }
    }

    async fn git_status() -> String {
        let output = Command::new("git").arg("status").output();
        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => format!("Failed to run git status: {}", e),
        }
    }

    async fn git_diff(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let staged = val["staged"].as_bool().unwrap_or(false);
        
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        if staged {
            cmd.arg("--staged");
        }
        
        match cmd.output() {
            Ok(out) => {
                let diff = String::from_utf8_lossy(&out.stdout);
                if diff.is_empty() {
                    "No changes found.".to_string()
                } else {
                    diff.to_string()
                }
            }
            Err(e) => format!("Failed to run git diff: {}", e),
        }
    }

    async fn git_add(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let paths = val["paths"].as_array();
        
        let mut cmd = Command::new("git");
        cmd.arg("add");
        
        if let Some(p_list) = paths {
            for p in p_list {
                if let Some(s) = p.as_str() {
                    cmd.arg(s);
                }
            }
        } else {
            return "Error: paths argument missing or not an array.".to_string();
        }

        match cmd.output() {
            Ok(_) => "Successfully added paths to git index.".to_string(),
            Err(e) => format!("Failed to run git add: {}", e),
        }
    }

    async fn git_commit(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let message = val["message"].as_str().unwrap_or_default();
        
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
            }
            Err(e) => format!("Failed to run git commit: {}", e),
        }
    }

    async fn git_log(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let limit = val["limit"].as_u64().unwrap_or(5);
        
        let output = Command::new("git")
            .args(["log", &format!("-n{}", limit), "--oneline"])
            .output();

        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => format!("Failed to run git log: {}", e),
        }
    }

    async fn git_branch(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let action = val["action"].as_str().unwrap_or("list");
        let name = val["name"].as_str().unwrap_or_default();

        let mut cmd = Command::new("git");
        cmd.arg("branch");

        match action {
            "create" => {
                if name.is_empty() { return "Error: branch name required for create.".to_string(); }
                cmd.arg(name);
            }
            "delete" => {
                if name.is_empty() { return "Error: branch name required for delete.".to_string(); }
                cmd.args(["-d", name]);
            }
            _ => {} // list is default
        }

        match cmd.output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.is_empty() && action != "list" {
                    format!("Successfully performed '{}' on branch '{}'.", action, name)
                } else {
                    stdout.to_string()
                }
            }
            Err(e) => format!("Failed to run git branch: {}", e),
        }
    }

    async fn search_code(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let pattern = val["pattern"].as_str().unwrap_or_default();
        let path = val["path"].as_str().unwrap_or(".");

        let output = Command::new("grep")
            .args(["-rni", pattern, path])
            .arg("--exclude-dir=.git")
            .arg("--exclude-dir=target")
            .output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout);
                if result.is_empty() {
                    "No matches found.".to_string()
                } else {
                    if result.len() > 5000 {
                        format!("{}... (truncated)", &result[..5000])
                    } else {
                        result.to_string()
                    }
                }
            }
            Err(e) => format!("Failed to run search: {}", e),
        }
    }
}
