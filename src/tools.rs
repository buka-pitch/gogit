use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::process::Command;
use walkdir::WalkDir;
use crate::ai::{ToolDefinition, ToolFunctionDefinition};
use toml;

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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
                    description: "Execute a shell command with options. Requires user confirmation.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The shell command to execute."
                            },
                            "cwd": {
                                "type": "string",
                                "description": "Working directory to run command in (default: current directory)."
                            },
                            "timeout": {
                                "type": "integer",
                                "description": "Timeout in seconds (default: 60, max: 300)."
                            },
                            "env": {
                                "type": "object",
                                "description": "Additional environment variables as key-value pairs.",
                                "additionalProperties": { "type": "string" }
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
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "detect_project_type".to_string(),
                    description: "Detect the project type and available information from config files.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Directory to scan (default '.')."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "get_workspace_info".to_string(),
                    description: "Get comprehensive workspace information including dependencies, scripts, and configuration.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Directory to scan (default '.')."
                            }
                        }
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "generate_tests".to_string(),
                    description: "Generate unit or integration tests for a source file. Supports Rust, JavaScript, TypeScript, Python, Go, Java, and C#.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Path to the source file to generate tests for."
                            },
                            "test_type": {
                                "type": "string",
                                "enum": ["unit", "integration"],
                                "description": "Type of tests to generate (default: unit)."
                            },
                            "framework": {
                                "type": "string",
                                "description": "Specific test framework to use (optional). Examples: jest, vitest, pytest, tokio, junit5, xunit."
                            }
                        },
                        "required": ["file_path"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "execute_code".to_string(),
                    description: "Execute a source code file and return the output. Supports Rust, JavaScript, TypeScript, Python, Go, Java, and C#.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Path to the source file to execute."
                            },
                            "args": {
                                "type": "string",
                                "description": "Command line arguments to pass to the program."
                            },
                            "timeout": {
                                "type": "integer",
                                "description": "Timeout in seconds (default: 60, max: 300)."
                            }
                        },
                        "required": ["file_path"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "preview_edit".to_string(),
                    description: "Show AI proposed changes as a diff before applying. Safer than direct editing.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Path to the file to edit."
                            },
                            "instruction": {
                                "type": "string",
                                "description": "Instructions for the changes to make."
                            },
                            "show_diff": {
                                "type": "boolean",
                                "description": "Whether to show unified diff format (default: true)."
                            }
                        },
                        "required": ["file_path", "instruction"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "scaffold_template".to_string(),
                    description: "Generate boilerplate code using AI. The AI will analyze your project and generate appropriate scaffolding.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Name of the component, module, or file to create."
                            },
                            "template_type": {
                                "type": "string",
                                "description": "Type: component, module, class, function, service, controller, model, etc."
                            },
                            "output_path": {
                                "type": "string",
                                "description": "Where to save the generated file."
                            },
                            "options": {
                                "type": "object",
                                "description": "Additional options like framework, style, etc."
                            }
                        },
                        "required": ["name", "output_path"]
                    }),
                },
            },
            ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: "security_command".to_string(),
                    description: "Execute a security/reconnaissance command with explanation. Shows command before execution.".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The security/reconnaissance command to execute."
                            },
                            "explanation": {
                                "type": "string",
                                "description": "Explanation of what this command does (for educational purpose)."
                            },
                            "timeout": {
                                "type": "integer",
                                "description": "Timeout in seconds (default: 30, max: 120)."
                            }
                        },
                        "required": ["command", "explanation"]
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
            "detect_project_type" => Self::detect_project_type(args).await,
            "get_workspace_info" => Self::get_workspace_info(args).await,
            "generate_tests" => Self::generate_tests_impl(args).await,
            "execute_code" => Self::execute_code(args).await,
            "preview_edit" => Self::preview_edit(args).await,
            "scaffold_template" => Self::scaffold_template(args).await,
            "security_command" => Self::security_command(args).await,
            _ => format!("Error: Tool '{}' not found.", name),
        }
    }

    async fn read_file(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = expand_path(val["path"].as_str().unwrap_or_default());
        match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => format!("Error reading file: {}", e),
        }
    }

    async fn write_file(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = expand_path(val["path"].as_str().unwrap_or_default());
        let content = val["content"].as_str().unwrap_or_default();
        let path_for_display = path.clone();
        
        match fs::write(&path, content) {
            Ok(_) => format!("Successfully wrote to {}", path_for_display),
            Err(e) => format!("Error writing file: {}", e),
        }
    }

    async fn list_directory(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = expand_path(val["path"].as_str().unwrap_or("."));
        
        let mut entries = Vec::new();
        for entry in WalkDir::new(&path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
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
        let path = expand_path(val["path"].as_str().unwrap_or("."));
        let depth = val["depth"].as_u64().unwrap_or(3) as usize;
        
        let mut entries = Vec::new();
        for entry in WalkDir::new(&path).max_depth(depth).into_iter().filter_map(|e| e.ok()) {
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
        let command_str = val["command"].as_str().unwrap_or_default().to_string();
        let cwd = val["cwd"].as_str().map(|s| expand_path(s));
        let _timeout_secs = val["timeout"].as_u64().unwrap_or(60).min(300);
        
        if command_str.trim().is_empty() {
            return "Error: command is required.".to_string();
        }

        let output = tokio::task::spawn_blocking(move || {
            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = Command::new("cmd");
                c.args(["/C", &command_str]);
                c
            } else {
                let mut c = Command::new("sh");
                c.args(["-c", &command_str]);
                c
            };
            
            if let Some(ref dir) = cwd {
                cmd.current_dir(dir);
            }
            
            cmd.output()
        })
        .await
        .map_err(|e| format!("Task error: {}", e));

        match output {
            Ok(Ok(out)) => {
                let exit_code = out.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                
                let stdout_truncated = Self::truncate_output(&stdout, 150);
                let stderr_truncated = if stderr.is_empty() { 
                    String::new() 
                } else { 
                    format!("\nSTDERR:\n{}", Self::truncate_output(&stderr, 50))
                };
                
                format!(
                    "Exit Code: {}\nSTDOUT:\n{}{}",
                    exit_code, stdout_truncated, stderr_truncated
                )
            }
            Ok(Err(e)) => format!("Command failed: {}", e),
            Err(e) => format!("Execution error: {}", e),
        }
    }

    fn truncate_output(output: &str, max_lines: usize) -> String {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() <= max_lines {
            output.to_string()
        } else {
            let first_half: Vec<&str> = lines[..max_lines / 2].to_vec();
            let last_half: Vec<&str> = lines[lines.len() - max_lines / 2..].to_vec();
            format!(
                "{}\n... ({} lines truncated) ...\n{}",
                first_half.join("\n"),
                lines.len() - max_lines,
                last_half.join("\n")
            )
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

    async fn detect_project_type(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or(".");
        let base_path = std::path::Path::new(path);

        let detected_types = vec![
            ("package.json", "nodejs", "Node.js"),
            ("Cargo.toml", "rust", "Rust"),
            ("go.mod", "go", "Go"),
            ("requirements.txt", "python", "Python"),
            ("pom.xml", "java", "Java (Maven)"),
            ("build.gradle", "java", "Java (Gradle)"),
            ("*.csproj", "csharp", "C#/.NET"),
            ("composer.json", "php", "PHP"),
            ("Gemfile", "ruby", "Ruby"),
            ("pyproject.toml", "python", "Python (Poetry)"),
            ("Cargo.toml", "rust", "Rust"),
        ];

        let config_files = vec![
            "package.json", "Cargo.toml", "go.mod", "requirements.txt",
            "pom.xml", "build.gradle", "composer.json", "Gemfile",
            "pyproject.toml", "setup.py", "Makefile", "docker-compose.yml",
            "Dockerfile", ".env.example", ".eslintrc", "tsconfig.json",
            ".gitignore", "Cargo.lock", "package-lock.json", "yarn.lock"
        ];

        let mut found_configs = Vec::new();
        let mut project_type = "unknown".to_string();
        let mut project_name = "unknown".to_string();

        for entry in WalkDir::new(base_path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                
                if config_files.iter().any(|cf| cf == &file_name) {
                    found_configs.push(file_name.clone());
                }

                for (config, ptype, _pname) in &detected_types {
                    if file_name == *config {
                        project_type = ptype.to_string();
                        
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if file_name == "package.json" {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                    if let Some(name) = json["name"].as_str() {
                                        project_name = name.to_string();
                                    }
                                }
                            } else if file_name == "Cargo.toml" {
                                if let Some(content_str) = content.lines().next() {
                                    if content_str.starts_with("name = ") {
                                        project_name = content_str.trim_start_matches("name = \"").trim_end_matches('"').to_string();
                                    }
                                }
                            } else if file_name == "go.mod" {
                                if let Some(line) = content.lines().find(|l| l.starts_with("module ")) {
                                    project_name = line.trim_start_matches("module ").to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        let package_manager = match project_type.as_str() {
            "nodejs" => {
                if found_configs.contains(&"yarn.lock".to_string()) {
                    "yarn"
                } else if found_configs.contains(&"pnpm-lock.yaml".to_string()) {
                    "pnpm"
                } else {
                    "npm"
                }
            }
            "python" => {
                if found_configs.contains(&"poetry.lock".to_string()) || found_configs.contains(&"pyproject.toml".to_string()) {
                    "poetry"
                } else {
                    "pip"
                }
            }
            _ => "default",
        };

        let result = serde_json::json!({
            "project_type": project_type,
            "project_name": project_name,
            "package_manager": package_manager,
            "config_files": found_configs,
            "detected": project_type != "unknown"
        });

        result.to_string()
    }

    async fn get_workspace_info(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let path = val["path"].as_str().unwrap_or(".");
        let base_path = std::path::Path::new(path);

        let mut info = serde_json::json!({
            "workspace": base_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            "sections": serde_json::json!({})
        });

        if let Ok(content) = fs::read_to_string(base_path.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let scripts = json.get("scripts").map(|s| {
                    s.as_object()
                        .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                        .unwrap_or_default()
                }).unwrap_or_default();
                
                let deps = json.get("dependencies").map(|d| d.as_object().map(|o| o.len()).unwrap_or(0)).unwrap_or(0);
                let dev_deps = json.get("devDependencies").map(|d| d.as_object().map(|o| o.len()).unwrap_or(0)).unwrap_or(0);

                info["scripts"] = serde_json::json!({
                    "available": scripts,
                    "common_commands": scripts.iter().filter(|s| matches!(s.as_str(), "dev" | "build" | "test" | "start" | "lint" | "format")).cloned().collect::<Vec<_>>()
                });
                info["dependencies"] = serde_json::json!({ "regular": deps, "dev": dev_deps });
            }
        }

        if let Ok(content) = fs::read_to_string(base_path.join("Cargo.toml")) {
            if let Ok(json) = toml::from_str::<toml::Value>(&content) {
                let deps = json.get("dependencies").map(|d| d.as_table().map(|t| t.len()).unwrap_or(0)).unwrap_or(0);
                info["rust"] = serde_json::json!({ "dependencies": deps });
            }
        }

        if let Ok(content) = fs::read_to_string(base_path.join("go.mod")) {
            let lines: Vec<&str> = content.lines().collect();
            let deps: Vec<&str> = lines.iter().skip_while(|l| !l.starts_with("require (")).take_while(|l| !l.starts_with(")")).cloned().collect();
            info["golang"] = serde_json::json!({ "modules": deps.len() });
        }

        if base_path.join("docker-compose.yml").exists() || base_path.join("docker-compose.yaml").exists() {
            info["docker"] = serde_json::json!({ "compose_available": true });
        }

        let env_example = if base_path.join(".env.example").exists() {
            Some(".env.example")
        } else if base_path.join(".env.sample").exists() {
            Some(".env.sample")
        } else {
            None
        };
        if let Some(env_file) = env_example {
            if let Ok(content) = fs::read_to_string(base_path.join(env_file)) {
                let vars: Vec<&str> = content.lines().filter(|l| !l.starts_with('#')).map(|l| l.split('=').next().unwrap_or("")).filter(|s| !s.is_empty()).collect();
                info["environment"] = serde_json::json!({ "template_file": env_file, "variables": vars });
            }
        }

        if let Ok(content) = fs::read_to_string(base_path.join("Makefile")) {
            let targets: Vec<&str> = content.lines().filter(|l| !l.starts_with('\t') && !l.starts_with(' ') && l.contains(':')).map(|l| l.split(':').next().unwrap_or("").trim()).filter(|s| !s.is_empty() && !s.starts_with('.')).collect();
            info["makefile"] = serde_json::json!({ "targets": targets });
        }

        info.to_string()
    }

    pub async fn generate_tests_impl(args: &str) -> String {
        use crate::testgen::TestGenerator;
        
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let file_path = val["file_path"].as_str().unwrap_or_default();
        let _test_type = val["test_type"].as_str().unwrap_or("unit");
        let _framework = val["framework"].as_str().map(|s| s.to_string());
        
        if file_path.is_empty() {
            return "Error: file_path is required".to_string();
        }

        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return format!("Error: File not found: {}", file_path);
        }

        let language = TestGenerator::detect_language(file_path);
        if !language.is_supported() {
            return "Unsupported language. Supported: Rust, JavaScript, TypeScript, Python, Go, Java, C#".to_string();
        }

        let project_path = path.parent().unwrap_or(std::path::Path::new(".")).to_string_lossy().to_string();
        let _fw = TestGenerator::detect_framework(&language, &project_path);

        let _content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return format!("Error reading file: {}", e),
        };

        // Get suggested location
        let existing_tests = TestGenerator::find_existing_tests(&language, &project_path);
        let suggested_location = TestGenerator::suggest_test_location(&language, file_path, &existing_tests);

        format!(
            "I can generate tests for {} ({})!\n\n\
            Suggested location: {}\n\n\
            To generate tests, please use the CLI command:\n\
            $ gogit testgen {}\n\n\
            Or use the interactive menu to select 'AI Test Gen'.\n\n\
            The AI will analyze your code and generate comprehensive unit tests \
            using the appropriate framework for your project.",
            language.display_name(),
            _fw.display_name(),
            suggested_location,
            file_path
        )
    }

    pub async fn execute_code(args: &str) -> String {
        use std::path::Path;
        
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let file_path = val["file_path"].as_str().unwrap_or_default();
        let args_str = val["args"].as_str().unwrap_or("");
        let timeout_secs = val["timeout"].as_u64().unwrap_or(60).min(300);
        
        if file_path.is_empty() {
            return "Error: file_path is required".to_string();
        }

        let path = Path::new(file_path);
        if !path.exists() {
            return format!("Error: File not found: {}", file_path);
        }

        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let language = match extension.as_str() {
            "rs" => "rust",
            "js" | "mjs" | "cjs" => "javascript",
            "ts" | "mts" | "cts" => "typescript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "cs" => "csharp",
            _ => "unknown",
        };

        if language == "unknown" {
            return format!("Error: Unsupported file type: .{}", extension);
        }

        let (mut cmd, _working_dir) = match language {
            "rust" => {
                // Check if it's a cargo project or single file
                let cargo_toml = path.parent().map(|p| p.join("Cargo.toml"));
                if cargo_toml.map(|p| p.exists()).unwrap_or(false) {
                    // It's a cargo project - use cargo run
                    let mut c = tokio::process::Command::new("cargo");
                    c.arg("run").current_dir(path.parent().unwrap_or(Path::new(".")));
                    if !args_str.is_empty() {
                        c.arg("--").args(args_str.split_whitespace());
                    }
                    (c, path.parent().map(|p| p.to_path_buf()))
                } else {
                    // Single file - compile and run
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    let compile_cmd = format!("rustc {} -o {} && ./{}{}", file_path, stem, stem, if args_str.is_empty() { String::new() } else { format!(" {}", args_str) });
                    let mut c = tokio::process::Command::new("sh");
                    c.arg("-c").arg(&compile_cmd);
                    (c, None)
                }
            }
            "javascript" => {
                let mut c = tokio::process::Command::new("node");
                if !args_str.is_empty() {
                    c.arg(file_path).args(args_str.split_whitespace());
                } else {
                    c.arg(file_path);
                }
                (c, path.parent().map(|p| p.to_path_buf()))
            }
            "typescript" => {
                // Try ts-node first, fallback to tsc
                let compile_run = format!("npx ts-node {} {}", file_path, args_str);
                let mut c = tokio::process::Command::new("sh");
                c.arg("-c").arg(&compile_run);
                (c, path.parent().map(|p| p.to_path_buf()))
            }
            "python" => {
                let mut c = tokio::process::Command::new("python3");
                if !args_str.is_empty() {
                    c.arg(file_path).args(args_str.split_whitespace());
                } else {
                    c.arg(file_path);
                }
                (c, path.parent().map(|p| p.to_path_buf()))
            }
            "go" => {
                let mut c = tokio::process::Command::new("go");
                c.arg("run");
                if !args_str.is_empty() {
                    c.arg(file_path).args(args_str.split_whitespace());
                } else {
                    c.arg(file_path);
                }
                (c, path.parent().map(|p| p.to_path_buf()))
            }
            "java" => {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let compile_run = format!("javac {} && java {} {}", file_path, stem, args_str);
                let mut c = tokio::process::Command::new("sh");
                c.arg("-c").arg(&compile_run);
                (c, path.parent().map(|p| p.to_path_buf()))
            }
            "csharp" => {
                let compile_run = format!("dotnet script {} -- {} 2>&1 || dotnet run --project . -- {} 2>&1 || echo 'Make sure dotnet is installed'", file_path, args_str, args_str);
                let mut c = tokio::process::Command::new("sh");
                c.arg("-c").arg(&compile_run);
                (c, None)
            }
            _ => return format!("Error: Unsupported language: {}", language),
        };

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            cmd.output()
        ).await;

        match output {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let exit_code = out.status.code().unwrap_or(-1);
                
                let mut result = format!("Exit Code: {}\n", exit_code);
                
                if !stdout.is_empty() {
                    result.push_str(&format!("\nSTDOUT:\n{}\n", Self::truncate_output(&stdout, 100)));
                }
                if !stderr.is_empty() {
                    result.push_str(&format!("\nSTDERR:\n{}\n", Self::truncate_output(&stderr, 50)));
                }
                
                if stdout.is_empty() && stderr.is_empty() {
                    result.push_str("(No output)\n");
                }
                
                result
            }
            Ok(Err(e)) => format!("Error executing: {}", e),
            Err(_) => format!("Error: Timeout after {} seconds", timeout_secs),
        }
    }

    pub async fn preview_edit(args: &str) -> String {
        
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let file_path = val["file_path"].as_str().unwrap_or_default();
        let instruction = val["instruction"].as_str().unwrap_or_default();
        
        if file_path.is_empty() {
            return "Error: file_path is required".to_string();
        }
        
        if instruction.is_empty() {
            return "Error: instruction is required".to_string();
        }

        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return format!("Error: File not found: {}", file_path);
        }

        let _content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return format!("Error reading file: {}", e),
        };

        format!(
            "To preview edits, please use the CLI command:\n\
            $ gogit fix {} \"{}\"\n\n\
            Or use the interactive menu to select 'AI Fix'.\n\n\
            Current file: {}\n\
            Instruction: {}\n\n\
            This will show you the proposed changes before applying.",
            file_path,
            instruction,
            file_path,
            instruction
        )
    }

    pub async fn scaffold_template(args: &str) -> String {
        use crate::testgen::TestGenerator;
        
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let name = val["name"].as_str().unwrap_or_default();
        let template_type = val["template_type"].as_str().unwrap_or("component");
        let output_path = val["output_path"].as_str().unwrap_or_default();
        
        if name.is_empty() {
            return "Error: name is required".to_string();
        }
        
        if output_path.is_empty() {
            return "Error: output_path is required".to_string();
        }

        // Detect project type to provide context
        let project_info = TestGenerator::detect_language(output_path);
        let project_path = std::path::Path::new(output_path)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_string_lossy()
            .to_string();
        
        let framework = TestGenerator::detect_framework(&project_info, &project_path);

        format!(
            "To generate scaffolding, please use the CLI or chat mode.\n\n\
            Template: {} '{}'\n\
            Output: {}\n\
            Project Type: {} ({})\n\n\
            The AI will analyze your project context and generate appropriate boilerplate code.\n\
            Use: 'gogit chat' and ask to create a {} called {}",
            template_type,
            name,
            output_path,
            project_info.display_name(),
            framework.display_name(),
            template_type,
            name
        )
    }

    pub async fn security_command(args: &str) -> String {
        let val: serde_json::Value = serde_json::from_str(args).unwrap_or_default();
        let command = val["command"].as_str().unwrap_or_default().to_string();
        let explanation = val["explanation"].as_str().unwrap_or("Executing command").to_string();
        let _timeout_secs = val["timeout"].as_u64().unwrap_or(30).min(120);
        
        if command.is_empty() {
            return "Error: command is required".to_string();
        }

        // Safety check - block dangerous commands
        let dangerous_patterns = [
            "rm -rf /", "mkfs", "dd if=/dev/zero", 
            ":(){:|:&};:", "chmod 777 /", "> /dev/sda",
            "fork()", "while(true)", "format c:",
        ];
        
        let cmd_lower = command.to_lowercase();
        for pattern in dangerous_patterns {
            if cmd_lower.contains(&pattern.to_lowercase()) {
                return format!(
                    "⚠️ BLOCKED: This command appears dangerous and has been blocked.\n\n\
                    Command: {}\n\n\
                    If you believe this is a false positive, please run it manually in your terminal.\n\
                    This safety measure is in place to prevent accidental system damage.",
                    command
                );
            }
        }

        // Save for use after closure
        let cmd_for_display = command.clone();
        
        let output = tokio::task::spawn_blocking(move || {
            let cmd_str = command;
            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = std::process::Command::new("cmd");
                c.args(["/C", &cmd_str]);
                c
            } else {
                let mut c = std::process::Command::new("sh");
                c.args(["-c", &cmd_str]);
                c
            };
            
            cmd.output()
        })
        .await
        .map_err(|e| format!("Task error: {}", e));

        match output {
            Ok(Ok(out)) => {
                let exit_code = out.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                
                let mut result = format!(
                    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                    📋 Command: {}\n\
                    💡 Explanation: {}\n\
                    Exit Code: {}\n\
                    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
                    cmd_for_display,
                    explanation,
                    exit_code
                );
                
                if !stdout.is_empty() {
                    result.push_str(&format!("\n📤 STDOUT:\n{}\n", Self::truncate_output(&stdout, 100)));
                }
                if !stderr.is_empty() {
                    result.push_str(&format!("\n📛 STDERR:\n{}\n", Self::truncate_output(&stderr, 50)));
                }
                if stdout.is_empty() && stderr.is_empty() {
                    result.push_str("(No output)\n");
                }
                
                result
            }
            Ok(Err(e)) => format!("❌ Error executing: {}", e),
            Err(_) => format!("❌ Timeout after 30 seconds"),
        }
    }
}
