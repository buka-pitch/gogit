use crate::ai::{AiClient, AiError, Message, Role, ToolCall};
use crate::tools::ToolRegistry;
use crate::tui::Tui;

pub struct Agent {
    ai: AiClient,
    tui: Tui,
    history: Vec<Message>,
    tools_enabled: bool,
}

impl Agent {
    pub fn new(ai: AiClient, tui: Tui, system_prompt: &str) -> Self {
        let history = vec![Message {
            role: Role::System,
            content: Some(system_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }];
        Self { ai, tui, history, tools_enabled: true }
    }

    pub fn tools_enabled(&self) -> bool {
        self.tools_enabled
    }

    pub fn provider_label(&self) -> &'static str {
        match self.ai.provider {
            crate::ai::AiProvider::OpenRouter => "OpenRouter",
            crate::ai::AiProvider::Gemini => "Gemini",
            crate::ai::AiProvider::Ollama => "Ollama",
        }
    }

    pub fn model_name(&self) -> &str {
        &self.ai.model
    }

    pub async fn chat(&mut self, user_input: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.tui.print_user_message(user_input);
        self.history.push(Message {
            role: Role::User,
            content: Some(user_input.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        loop {
            let spinner = self.tui.start_thinking("Thinking...");
            let tools = if self.tools_enabled {
                Some(ToolRegistry::get_definitions())
            } else {
                None
            };
            
            let result = self.ai.chat_with_tools(self.history.clone(), tools).await;
            self.tui.stop_spinner(spinner);

            let response = match result {
                Ok(msg) => msg,
                Err(e) => {
                    match e {
                        AiError::ToolIncompatibility(msg) => {
                            self.tui.print_warning(&format!(
                                "Tool incompatibility detected: {}",
                                msg
                            ));
                            if self.tui.prompt_yes_no("This model/provider doesn't seem to support tools. Continue with tools disabled for this session?")? {
                                self.tools_enabled = false;
                                continue; // Retry the same user input without tools
                            } else {
                                return Ok(());
                            }
                        }
                        _ => {
                            let err_msg = e.to_string();
                            if err_msg.contains("Rate limit") {
                                self.tui.print_warning("Model rate limit reached. The agent loop might be too intense for this free model.");
                                self.tui.print_info("Try waiting a minute or switch to a different model using 'gogit models'.");
                            } else {
                                self.tui.print_error(&format!("AI Error: {}", err_msg));
                            }
                            return Ok(());
                        }
                    }
                }
            };

            // Add assistant response to history
            self.history.push(response.clone());

            if let Some(tool_calls) = &response.tool_calls {
                if tool_calls.is_empty() {
                    if let Some(content) = &response.content {
                        self.tui.print_assistant_panel(content);
                    }
                    break;
                }

                for call in tool_calls {
                    self.handle_tool_call(call).await?;
                }
                // Continue loop to let AI process tool results
            } else {
                if let Some(content) = &response.content {
                    self.tui.print_assistant_panel(content);
                }
                break;
            }
        }

        Ok(())
    }

    async fn handle_tool_call(&mut self, call: &ToolCall) -> Result<(), Box<dyn std::error::Error>> {
        let name = &call.function.name;
        let args = &call.function.arguments;

        // Basic validation for malformed tool calls
        if args.trim().is_empty() && !["git_status"].contains(&name.as_str()) {
             self.tui.print_info(&format!("Repairing empty arguments for tool: {}", name));
        }

        self.tui.print_tool_call(name, args);

        let result = if name == "write_file" || name == "run_command" || name == "git_commit" {
            if self.tui.prompt_yes_no(&format!("Allow tool '{}' to proceed?", name))? {
                ToolRegistry::call_tool(name, args).await
            } else {
                "Error: User denied permission to run this tool.".to_string()
            }
        } else {
            ToolRegistry::call_tool(name, args).await
        };

        // Display a summarized result to keep the UI clean
        let display_result = if result.len() > 200 {
            format!("{}... (total {} chars)", &result[..200].replace('\n', " "), result.len())
        } else {
            result.clone().replace('\n', " ")
        };
        
        self.tui.print_tool_result(&display_result);

        self.history.push(Message {
            role: Role::Tool,
            content: Some(result),
            tool_calls: None,
            tool_call_id: Some(call.id.clone()),
        });

        Ok(())
    }
}
