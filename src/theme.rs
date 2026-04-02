use crossterm::style::Color;

#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    pub const NEON_GREEN: Color = Color::Rgb { r: 0, g: 255, b: 0 };
    pub const NEON_CYAN: Color = Color::Rgb {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const NEON_MAGENTA: Color = Color::Rgb {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const NEON_YELLOW: Color = Color::Rgb {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const NEON_RED: Color = Color::Rgb {
        r: 255,
        g: 51,
        b: 51,
    };
    pub const NEON_BLUE: Color = Color::Rgb {
        r: 51,
        g: 153,
        b: 255,
    };
    pub const NEON_ORANGE: Color = Color::Rgb {
        r: 255,
        g: 153,
        b: 0,
    };

    pub const BG_DARK: Color = Color::Rgb {
        r: 13,
        g: 13,
        b: 13,
    };
    pub const BG_MATRIX: Color = Color::Rgb { r: 0, g: 20, b: 0 };
    pub const BG_CYAN_DARK: Color = Color::Rgb { r: 0, g: 30, b: 30 };

    pub const TEXT_BRIGHT: Color = Color::Rgb {
        r: 224,
        g: 224,
        b: 224,
    };
    pub const TEXT_DIM: Color = Color::Rgb {
        r: 128,
        g: 128,
        b: 128,
    };

    pub const PROMPT: Color = Self::NEON_GREEN;
    pub const SUCCESS: Color = Self::NEON_GREEN;
    pub const ERROR: Color = Self::NEON_RED;
    pub const WARNING: Color = Self::NEON_YELLOW;
    pub const INFO: Color = Self::NEON_CYAN;

    pub const BOX_TOP_LEFT: &'static str = "╭";
    pub const BOX_TOP_RIGHT: &'static str = "╮";
    pub const BOX_BOTTOM_LEFT: &'static str = "╰";
    pub const BOX_BOTTOM_RIGHT: &'static str = "╯";
    pub const BOX_HORIZONTAL: &'static str = "─";
    pub const BOX_VERTICAL: &'static str = "│";

    pub const PROMPT_SYMBOL: &'static str = "❯";
    pub const PROMPT_SYMBOL_ALT: &'static str = "➜";
    pub const SHELL_SYMBOL: &'static str = "$";
}

pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn render(markdown: &str) -> String {
        use pulldown_cmark::{html, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Self::html_to_styled(&html_output)
    }

    fn html_to_styled(html: &str) -> String {
        use colored::Colorize;

        let mut result = String::new();
        let mut in_paragraph = false;
        let in_list_item = false;
        let mut in_heading = false;
        let mut heading_level = 0;
        let mut pending_heading = String::new();

        for line in html.lines() {
            let line = line.trim();

            if line.starts_with("<h1>") {
                in_heading = true;
                heading_level = 1;
                pending_heading.clear();
                continue;
            }
            if line.starts_with("<h2>") {
                in_heading = true;
                heading_level = 2;
                pending_heading.clear();
                continue;
            }
            if line.starts_with("<h3>") {
                in_heading = true;
                heading_level = 3;
                pending_heading.clear();
                continue;
            }

            if in_heading {
                if line.contains("</h1>") || line.contains("</h2>") || line.contains("</h3>") {
                    let text = Self::strip_tags(&pending_heading);
                    let hashes = "#".repeat(heading_level);
                    let styled = format!("{} {}", hashes, text.cyan().bold());
                    let underline = "-".repeat(text.len());
                    result.push_str(&format!("\n{}\n{}\n\n", styled, underline));
                    in_heading = false;
                    pending_heading.clear();
                } else {
                    pending_heading.push_str(line);
                }
                continue;
            }

            // Handle <p>text</p> on same line
            if line.starts_with("<p>") && line.contains("</p>") {
                let text = Self::strip_tags(line);
                let styled = Self::style_inline(&text);
                result.push_str(&format!("{}\n", styled));
                continue;
            }
            // Handle <p>text</p> where closing is on same line as opening
            if line.starts_with("<p>") {
                in_paragraph = true;
                // Check if there's text on the same line after <p>
                let text = line.strip_prefix("<p>").unwrap_or("");
                if !text.is_empty() && !text.starts_with("<") {
                    let styled = Self::style_inline(text);
                    result.push_str(&format!("{}", styled));
                }
                continue;
            }
            if line.starts_with("</p>") {
                in_paragraph = false;
                result.push('\n');
                // Check if there's text before </p> on same line
                let text = line.strip_prefix("</p>").unwrap_or("");
                if !text.is_empty() {
                    let styled = Self::style_inline(text);
                    result.push_str(&format!("{}", styled));
                }
                continue;
            }

            if line.starts_with("<li>") {
                let text = Self::strip_tags(line);
                let styled = Self::style_inline(&text);
                result.push_str(&format!("  ▸ {}\n", styled.yellow()));
                continue;
            }
            if line.starts_with("</li>") {
                continue;
            }

            if line.starts_with("<code>") {
                let text = Self::strip_tags(line);
                result.push_str(&format!("    {}\n", text.green()));
                continue;
            }
            if line.starts_with("</code>") {
                continue;
            }
            if line.starts_with("<pre>") {
                continue;
            }
            if line.starts_with("</pre>") {
                continue;
            }

            // Handle <strong> and <b> tags
            if line.contains("<strong>") || line.contains("<b>") {
                let styled = Self::strip_tags(line).bold().to_string();
                result.push_str(&format!("{}\n", styled));
                continue;
            }

            // Handle <em> and <i> tags
            if line.contains("<em>") || line.contains("<i>") {
                let styled = Self::strip_tags(line).dimmed().to_string();
                result.push_str(&format!("{}\n", styled));
                continue;
            }

            // Handle <blockquote>
            if line.starts_with("<blockquote>") {
                let text = Self::strip_tags(line);
                result.push_str(&format!("  │ {}\n", text.cyan()));
                continue;
            }
            if line.starts_with("</blockquote>") {
                continue;
            }

            // Handle <hr> or <hr/>
            if line.contains("<hr") {
                result.push_str(&format!("{}\n", "-".repeat(50).dimmed()));
                continue;
            }

            if in_paragraph
                && !in_list_item
                && !line.is_empty()
                && !line.starts_with("<")
                && !line.starts_with(">")
            {
                let styled = Self::style_inline(line);
                result.push_str(&format!("{}\n", styled));
            }
        }

        result
    }

    fn strip_tags(html: &str) -> String {
        let mut result = html.to_string();
        let tags = [
            "<h1>",
            "</h1>",
            "<h2>",
            "</h2>",
            "<h3>",
            "</h3>",
            "<p>",
            "</p>",
            "<strong>",
            "</strong>",
            "<em>",
            "</em>",
            "<code>",
            "</code>",
            "<li>",
            "</li>",
            "<ul>",
            "</ul>",
            "<ol>",
            "</ol>",
            "<br>",
            "<br/>",
            "<br />",
            "<a href=\"",
            "\">",
            "</a>",
            "<b>",
            "</b>",
            "<i>",
            "</i>",
            "<strike>",
            "</strike>",
            "<pre>",
            "</pre>",
            "<div>",
            "</div>",
            "<span>",
            "</span>",
        ];

        for tag in tags {
            result = result.replace(tag, "");
        }

        while let Some(start) = result.find("[") {
            if let Some(end) = result[start..].find("]") {
                let text_start = start + 1;
                let text_end = start + end;
                let text = &result[text_start..text_end];

                let link_start = text_end + 1;
                if let Some(_url_start_pos) = result[link_start..].find("(") {
                    if let Some(url_end_pos) = result[link_start..].find(")") {
                        let url = &result[link_start + 1..link_start + url_end_pos];
                        result = format!(
                            "{}[{}]({}){}",
                            &result[..start],
                            text,
                            url,
                            &result[link_start + url_end_pos + 1..]
                        );
                    }
                }
            }
        }

        result
    }

    fn style_inline(text: &str) -> String {
        use colored::Colorize;

        let mut result = text.to_string();

        // Handle **bold** text - must process before single *
        let mut output = String::new();
        let mut current = result.clone();
        
        while let Some(start) = current.find("**") {
            // Add text before the opening **
            output.push_str(&current[..start]);
            
            if let Some(end) = current[start+2..].find("**") {
                let bold_content = &current[start+2..start+2+end];
                output.push_str(&bold_content.bold().to_string());
                current = current[start+4+end..].to_string();
            } else {
                // No closing ** found
                output.push_str(&current[start..]);
                break;
            }
        }
        if !current.is_empty() {
            output.push_str(&current);
        }

        // Handle __bold__ (alternative syntax)
        result = output;
        output = String::new();
        current = result.clone();
        
        while let Some(start) = current.find("__") {
            output.push_str(&current[..start]);
            
            if let Some(end) = current[start+2..].find("__") {
                let bold_content = &current[start+2..start+2+end];
                output.push_str(&bold_content.bold().to_string());
                current = current[start+4+end..].to_string();
            } else {
                output.push_str(&current[start..]);
                break;
            }
        }
        if !current.is_empty() {
            output.push_str(&current);
        }

        // Handle *italic* text (but not **)
        result = output;
        output = String::new();
        current = result.clone();
        
        while let Some(start) = current.find('*') {
            // Skip ** as those are bold
            if start + 1 < current.len() && current.chars().nth(start + 1) == Some('*') {
                output.push_str(&current[..start+1]);
                current = current[start+1..].to_string();
                continue;
            }
            
            output.push_str(&current[..start]);
            
            if let Some(end) = current[start+1..].find('*') {
                let italic_content = &current[start+1..start+1+end];
                // Use dim as substitute for italic (colored doesn't have italic)
                output.push_str(&italic_content.dimmed().to_string());
                current = current[start+2+end..].to_string();
            } else {
                output.push_str(&current[start..]);
                break;
            }
        }
        if !current.is_empty() {
            output.push_str(&current);
        }

        // Handle _italic_ (but not __)
        result = output;
        output = String::new();
        current = result.clone();
        
        while let Some(start) = current.find('_') {
            // Skip __ as those are bold
            if start + 1 < current.len() && current.chars().nth(start + 1) == Some('_') {
                output.push_str(&current[..start+1]);
                current = current[start+1..].to_string();
                continue;
            }
            
            output.push_str(&current[..start]);
            
            if let Some(end) = current[start+1..].find('_') {
                let italic_content = &current[start+1..start+1+end];
                output.push_str(&italic_content.dimmed().to_string());
                current = current[start+2+end..].to_string();
            } else {
                output.push_str(&current[start..]);
                break;
            }
        }
        if !current.is_empty() {
            output.push_str(&current);
        }

        output
    }
}
