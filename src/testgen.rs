use crate::ai::AiClient;
use crate::tui::Tui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use crossterm::style::Stylize;
use dialoguer::{Select, Input};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    CSharp,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Language::TypeScript,
            "py" => Language::Python,
            "go" => Language::Go,
            "java" => Language::Java,
            "cs" => Language::CSharp,
            _ => Language::Unknown,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Unknown => "Unknown",
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, Language::Unknown)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestFramework {
    RustNative,
    TokioTest,
    Jest,
    Vitest,
    Pytest,
    Unittest,
    GoTesting,
    JUnit4,
    JUnit5,
    #[allow(non_camel_case_types)]
    xUnit,
    NUnit,
    Unknown,
}

impl TestFramework {
    pub fn display_name(&self) -> &str {
        match self {
            TestFramework::RustNative => "Rust Native (#[test])",
            TestFramework::TokioTest => "Tokio Test (#[tokio::test])",
            TestFramework::Jest => "Jest",
            TestFramework::Vitest => "Vitest",
            TestFramework::Pytest => "pytest",
            TestFramework::Unittest => "unittest",
            TestFramework::GoTesting => "Go testing",
            TestFramework::JUnit4 => "JUnit 4",
            TestFramework::JUnit5 => "JUnit 5",
            TestFramework::xUnit => "xUnit",
            TestFramework::NUnit => "NUnit",
            TestFramework::Unknown => "Unknown",
        }
    }

    #[allow(dead_code)]
    pub fn test_keyword(&self) -> &str {
        match self {
            TestFramework::RustNative => "#[test]",
            TestFramework::TokioTest => "#[tokio::test]",
            TestFramework::Jest | TestFramework::Vitest => "test",
            TestFramework::Pytest | TestFramework::Unittest => "def test_",
            TestFramework::GoTesting => "func Test",
            TestFramework::JUnit4 | TestFramework::JUnit5 => "@Test",
            TestFramework::xUnit => "[Fact]",
            TestFramework::NUnit => "[Test]",
            TestFramework::Unknown => "# test",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestLocation {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub test_files: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SourceFileInfo {
    pub path: String,
    pub language: Language,
    pub framework: TestFramework,
    pub functions: Vec<String>,
    pub existing_tests: Vec<TestLocation>,
    pub suggested_location: String,
}

#[derive(Debug, Clone)]
pub struct TestOptions {
    pub test_type: String,  // "unit" or "integration"
    pub framework: Option<String>,
    pub output_path: Option<String>,
    pub preview_only: bool,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            test_type: "unit".to_string(),
            framework: None,
            output_path: None,
            preview_only: false,
        }
    }
}

pub struct TestGenerator;

impl TestGenerator {
    pub fn detect_language(file_path: &str) -> Language {
        let path = Path::new(file_path);
        if let Some(ext) = path.extension() {
            Language::from_extension(&ext.to_string_lossy())
        } else {
            Language::Unknown
        }
    }

    pub fn detect_framework(language: &Language, project_path: &str) -> TestFramework {
        let base_path = Path::new(project_path);
        
        match language {
            Language::Rust => {
                if base_path.join("Cargo.toml").exists() {
                    if let Ok(content) = fs::read_to_string(base_path.join("Cargo.toml")) {
                        if content.contains("tokio") {
                            return TestFramework::TokioTest;
                        }
                    }
                }
                TestFramework::RustNative
            }
            Language::JavaScript | Language::TypeScript => {
                if base_path.join("vite.config.ts").exists() || base_path.join("vitest.config.ts").exists() {
                    return TestFramework::Vitest;
                }
                if base_path.join("jest.config.js").exists() 
                    || base_path.join("jest.config.ts").exists()
                    || base_path.join("jest.config.json").exists() {
                    return TestFramework::Jest;
                }
                if base_path.join("package.json").exists() {
                    if let Ok(content) = fs::read_to_string(base_path.join("package.json")) {
                        if content.contains("\"vitest\"") {
                            return TestFramework::Vitest;
                        }
                        if content.contains("\"jest\"") {
                            return TestFramework::Jest;
                        }
                    }
                }
                TestFramework::Jest
            }
            Language::Python => {
                if base_path.join("pytest.ini").exists() 
                    || base_path.join("pyproject.toml").exists()
                    || base_path.join("conftest.py").exists() {
                    return TestFramework::Pytest;
                }
                TestFramework::Pytest
            }
            Language::Go => TestFramework::GoTesting,
            Language::Java => {
                if base_path.join("pom.xml").exists() {
                    if let Ok(content) = fs::read_to_string(base_path.join("pom.xml")) {
                        if content.contains("junit-jupiter") || content.contains("junit5") {
                            return TestFramework::JUnit5;
                        }
                    }
                }
                TestFramework::JUnit5
            }
            Language::CSharp => {
                let csproj_exists = WalkDir::new(base_path)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_name().to_string_lossy().ends_with(".csproj"));
                
                if csproj_exists {
                    return TestFramework::xUnit;
                }
                TestFramework::xUnit
            }
            Language::Unknown => TestFramework::Unknown,
        }
    }

    pub fn find_existing_tests(language: &Language, project_path: &str) -> Vec<TestLocation> {
        let base_path = Path::new(project_path);
        let mut locations = Vec::new();

        let test_patterns: Vec<(&str, &str)> = match language {
            Language::Rust => vec![("tests", "*.rs"), ("src", "*_test.rs")],
            Language::JavaScript | Language::TypeScript => {
                vec![("__tests__", "*"), ("tests", "*"), ("spec", "*"), (".", "*.test.*"), (".", "*.spec.*")]
            }
            Language::Python => vec![("tests", "*.py"), (".", "test_*.py"), (".", "*_test.py")],
            Language::Go => vec![("", "*_test.go")],
            Language::Java => vec![("src/test/java", "*.java"), ("test", "*.java")],
            Language::CSharp => vec![("Tests", "*.cs"), ("test", "*.cs"), (".", "*Tests.cs")],
            Language::Unknown => vec![],
        };

        for (dir, _pattern) in test_patterns {
            let search_dir = if dir.is_empty() { 
                std::path::PathBuf::from(project_path)
            } else { 
                base_path.join(dir) 
            };
            if !search_dir.exists() {
                continue;
            }

            let test_files: Vec<String> = WalkDir::new(&search_dir)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.contains("test") || name.contains("spec")
                })
                .map(|e| e.path().display().to_string())
                .collect();

            if !test_files.is_empty() {
                locations.push(TestLocation {
                    path: search_dir.display().to_string(),
                    exists: true,
                    is_directory: search_dir.is_dir(),
                    test_files,
                });
            }
        }

        locations
    }

    pub fn suggest_test_location(language: &Language, source_file: &str, existing_tests: &[TestLocation]) -> String {
        let source_path = Path::new(source_file);
        let _source_dir = source_path.parent().unwrap_or(Path::new("."));
        let source_stem = source_path.file_stem().unwrap_or_default().to_string_lossy();

        if !existing_tests.is_empty() {
            let first_loc = &existing_tests[0];
            return format!("{}/{}.test.{}", 
                first_loc.path, 
                source_stem,
                match language {
                    Language::Rust => "rs",
                    Language::JavaScript => "js",
                    Language::TypeScript => "ts",
                    Language::Python => "py",
                    Language::Go => "go",
                    Language::Java => "java",
                    Language::CSharp => "cs",
                    _ => "txt",
                }
            );
        }

        let default_dir = match language {
            Language::Rust => "tests",
            Language::JavaScript | Language::TypeScript => "__tests__",
            Language::Python => "tests",
            Language::Go => "",
            Language::Java => "src/test/java",
            Language::CSharp => "Tests",
            _ => ".",
        };

        let ext = match language {
            Language::Rust => "rs",
            Language::JavaScript => "js",
            Language::TypeScript => "ts",
            Language::Python => "py",
            Language::Go => "go",
            Language::Java => "java",
            Language::CSharp => "cs",
            _ => "txt",
        };

        if default_dir.is_empty() {
            format!("{}_test.{}", source_stem, ext)
        } else {
            format!("{}/{}_test.{}", default_dir, source_stem, ext)
        }
    }

    pub async fn generate_tests(
        ai: &AiClient,
        source_file: &str,
        options: TestOptions,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = Path::new(source_file);
        if !path.exists() {
            return Err(format!("File not found: {}", source_file).into());
        }

        let content = fs::read_to_string(path)?;
        
        let language = Self::detect_language(source_file);
        if !language.is_supported() {
            return Err(format!(
                "Unsupported language. Supported: Rust, JavaScript, TypeScript, Python, Go, Java, C#"
            ).into());
        }

        let project_path = path.parent().unwrap_or(Path::new("."));
        let framework = if let Some(fw) = &options.framework {
            match fw.to_lowercase().as_str() {
                "jest" => TestFramework::Jest,
                "vitest" => TestFramework::Vitest,
                "pytest" => TestFramework::Pytest,
                "unittest" => TestFramework::Unittest,
                "junit4" => TestFramework::JUnit4,
                "junit5" => TestFramework::JUnit5,
                "xunit" => TestFramework::xUnit,
                "nunit" => TestFramework::NUnit,
                "tokio" => TestFramework::TokioTest,
                "native" | "rust" => TestFramework::RustNative,
                _ => TestFramework::Unknown,
            }
        } else {
            Self::detect_framework(&language, &project_path.to_string_lossy())
        };

        let system_prompt = Self::get_system_prompt(&language, &framework, &options.test_type);

        let prompt = format!(
            "Generate {} tests for the following source file.\n\
            Language: {}\n\
            Framework: {}\n\n\
            SOURCE FILE:\n\
            ```{}\
            \n{}\n\
            ```",
            options.test_type,
            language.display_name(),
            framework.display_name(),
            language.display_name().to_lowercase(),
            content
        );

        let tests = ai.generate_text(&prompt, &system_prompt).await?;
        
        Ok(tests)
    }

    fn get_system_prompt(language: &Language, framework: &TestFramework, test_type: &str) -> String {
        let base = format!(
            "You are an expert test engineer. Generate comprehensive, idiomatic {} {} tests.\n\n\
            Requirements:\n\
            1. Use {} testing framework\n\
            2. Cover edge cases and error conditions\n\
            3. Follow best practices: naming, assertions, setup/teardown\n\
            4. Output ONLY the test file content - no explanations, no markdown code blocks\n\
            5. If {} tests already exist in the file, append new tests rather than duplicating",
            test_type,
            language.display_name(),
            framework.display_name(),
            test_type
        );

        let language_specific = match language {
            Language::Rust => "\n\nFor Rust: Use proper error handling, avoid unwrap() in tests, use assert! and assert_eq! macros.",
            Language::JavaScript | Language::TypeScript => "\n\nFor JS/TS: Use describe/it blocks, expect assertions, handle async properly with async/await.",
            Language::Python => "\n\nFor Python: Use descriptive test names starting with test_, use pytest assertions, handle exceptions with pytest.raises.",
            Language::Go => "\n\nFor Go: Use testify package if available, t.Fatal/t.Error for failures, table-driven tests when appropriate.",
            Language::Java => "\n\nFor Java: Use JUnit annotations (@Test, @Before, @After), assertions from Assert class.",
            Language::CSharp => "\n\nFor C#: Use [Fact] or [Theory] attributes, Assert class for assertions.",
            _ => "",
        };

        format!("{}{}", base, language_specific)
    }

    #[allow(dead_code)]
    pub fn analyze_existing_tests(test_files: &[String]) -> Vec<String> {
        let mut functions = Vec::new();
        
        for file in test_files.iter().take(3) {
            if let Ok(content) = fs::read_to_string(file) {
                let file_path = Path::new(file);
                let ext = file_path.extension().unwrap_or_default().to_string_lossy();
                let lang = Language::from_extension(&ext);
                
                let patterns = match lang {
                    Language::Rust => vec!["fn ", "async fn "],
                    Language::JavaScript | Language::TypeScript => vec!["function ", "const ", "let ", "async "],
                    Language::Python => vec!["def "],
                    Language::Go => vec!["func "],
                    Language::Java => vec!["public ", "private ", "protected "],
                    Language::CSharp => vec!["public ", "private ", "void "],
                    _ => vec!["fn ", "function ", "def ", "func "],
                };

                for line in content.lines() {
                    for pattern in &patterns {
                        if line.contains(pattern) && line.contains('(') {
                            if let Some(name) = line.split(pattern).nth(1) {
                                let func_name = name.split('(').next().unwrap_or("").trim();
                                if !func_name.is_empty() && !func_name.starts_with('_') {
                                    functions.push(func_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        functions.dedup();
        functions
    }

    pub async fn run(
        tui: &Tui,
        ai: &AiClient,
        source_file: &str,
        options: TestOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(source_file);
        if !path.exists() {
            return Err(format!("File not found: {}", source_file).into());
        }

        let language = Self::detect_language(source_file);
        if !language.is_supported() {
            println!("\n{} Unsupported file type. Supported languages:", "✖".red());
            println!("  - Rust (.rs)");
            println!("  - JavaScript (.js)");
            println!("  - TypeScript (.ts)");
            println!("  - Python (.py)");
            println!("  - Go (.go)");
            println!("  - Java (.java)");
            println!("  - C# (.cs)");
            return Ok(());
        }

        let project_path = path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string();
        let framework = Self::detect_framework(&language, &project_path);
        let existing_tests = Self::find_existing_tests(&language, &project_path);
        
        println!("\n{} Analyzing source file...", "🔍".cyan());
        
        let content = fs::read_to_string(path)?;
        let _source_functions = Self::analyze_source_functions(&content, &language);

        println!("{} Detected: {} ({})", "✓".green(), language.display_name(), framework.display_name());

        let suggested_location = if let Some(ref output) = options.output_path {
            output.clone()
        } else {
            Self::suggest_test_location(&language, source_file, &existing_tests)
        };

        println!("\n{} Suggested test location: {}", "📁".cyan(), suggested_location.clone().yellow());
        
        if !existing_tests.is_empty() {
            println!("{} Found {} existing test location(s):", "📂".cyan(), existing_tests.len());
            for loc in existing_tests.iter().take(2) {
                println!("  - {} ({} files)", loc.path, loc.test_files.len());
            }
        }

        if options.preview_only {
            println!("\n{} Generating preview...", "⏳".cyan());
            let spinner = tui.start_thinking("AI is generating tests...");
            let tests = Self::generate_tests(ai, source_file, options.clone()).await?;
            tui.stop_spinner(spinner);
            
            println!("\n{}\n", "─".repeat(60).green());
            println!("{}", tests);
            println!("{}\n", "─".repeat(60).green());
            return Ok(());
        }

        println!("\n{}", "─".repeat(60).cyan());
        
        let choices = vec![
            format!("Save to {} (create/overwrite)", &suggested_location),
            "Save to custom location".to_string(),
            "Preview only (show in terminal)".to_string(),
            "Cancel".to_string(),
        ];

        let selection = Select::new()
            .with_prompt("Choose action:")
            .items(&choices)
            .default(0)
            .interact()?;

        let final_location = match selection {
            0 => suggested_location.clone(),
            1 => {
                let input: String = Input::new()
                    .with_prompt("Enter output path:")
                    .interact_text()?;
                input
            }
            2 => {
                let spinner = tui.start_thinking("AI is generating tests...");
                let tests = Self::generate_tests(ai, source_file, options).await?;
                tui.stop_spinner(spinner);
                
                println!("\n{}\n", "─".repeat(60).green());
                println!("{}", tests);
                println!("{}\n", "─".repeat(60).green());
                return Ok(());
            }
            _ => return Ok(()),
        };

        let spinner = tui.start_thinking("AI is generating tests...");
        let tests = Self::generate_tests(ai, source_file, options).await?;
        tui.stop_spinner(spinner);

        fs::write(&final_location, &tests)?;
        
        println!("\n{} Tests saved to: {}", "✔".green(), final_location.yellow());
        
        Ok(())
    }

    fn analyze_source_functions(content: &str, language: &Language) -> Vec<String> {
        let mut functions = Vec::new();
        
        let patterns = match language {
            Language::Rust => vec!["fn ", "async fn "],
            Language::JavaScript | Language::TypeScript => vec!["function ", "const ", "let "],
            Language::Python => vec!["def "],
            Language::Go => vec!["func "],
            Language::Java => vec!["public ", "private "],
            Language::CSharp => vec!["public ", "private ", "protected "],
            _ => vec!["fn ", "function ", "def ", "func "],
        };

        for line in content.lines() {
            for pattern in &patterns {
                if line.contains(pattern) && line.contains('(') {
                    if let Some(name) = line.split(pattern).nth(1) {
                        let func_name = name.split('(').next().unwrap_or("").trim();
                        if !func_name.is_empty() && !func_name.starts_with('_') && func_name.len() < 50 {
                            functions.push(func_name.to_string());
                        }
                    }
                }
            }
        }

        functions.dedup();
        functions
    }
}
