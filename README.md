<p align="center">
  <img src="img1.png" width="150" alt="gogit Logo">
</p>

# gogit CLI

<p align="center">
  <img src="img.png" width="100%" alt="gogit Cover">
</p>

**gogit** is a command-line interface tool that integrates AI capabilities into your Git workflow. It leverages AI to generate commit messages, draft pull request descriptions, review code, and semantically search your Git history.

---

- **🤖 AI Agent Chat**: Interactive chat mode with tool support (web search, file read/write, shell execution).
- **🎯 AI Model Browser**: Easily browse and switch between dozens of free OpenRouter models.
- **✨ AI Commit**: Generate meaningful commit messages from your staged changes.
- **🚀 Smart PRs**: Automatically generate pull request descriptions.
- **🕵️ Code Review**: Get AI-powered feedback on your code changes.
- **🛠️ AI Refactor**: Precision refactoring using natural language.
- **🔍 Semantic Search**: Search your git history using natural language.
- **📑 Document Generator**: Keep your README and documentation up to date.
- **🗺️ Repo Navigator**: Ask questions about your codebase.
- **Sweep and Cleanup**: Identify and remove stale branches.

## 🚀 Getting Started

1. **Install**:

   ```bash
   chmod +x install.sh
   ./install.sh
   ```

2. **Setup OpenRouter**:

   - Get an API key from [openrouter.ai](https://openrouter.ai/).
   - Set `OPENROUTER_API_KEY` in your environment or `.env` file.
   - (Optional) Get a **Native Gemini Key** from [Google AI Studio](https://aistudio.google.com/) and set `GEMINI_API_KEY` for 100% reliable tool use.
   - (Optional) Set `TAVILY_API_KEY` for better web search in chat mode.

3. **Configure**:
   ```bash
   gogit models # Select your preferred AI model
   ```

## 🤖 AI Agent Chat

The new `gogit chat` command puts a powerful autonomous agent at your fingertips.

- **📂 File Registry**: `read_file`, `write_file`, `list_directory`, `get_file_tree`.
- **⚙️ Git Integration**: `git_status`, `git_diff`, `git_add`, `git_commit`, `git_log`, `git_branch`.
- **🔍 Smart Search**: `search_code`, `web_search` (Tavily/DDG).
- **🌐 Documentation**: `read_url` (direct URL extraction).
- **💻 Safe Execution**: `run_command`, `git_commit` (requires interactive approval).

```bash
gogit chat "Explain the project structure and suggest improvements"
```

* **✨ AI Commit**: Generate and commit smart, descriptive commit messages automatically.
* **🚀 AI Pull Request**: Create detailed PR descriptions with summaries and key changes.
* **🕵️ AI Code Review**: Receive architectural and security-focused feedback on your changes.
* **🛠️ AI Fix**: Precision AI refactoring and bug fixing based on your instructions.
* **🔍 AI Search**: Semantic search through your git history using natural language.
* **📑 AI Documentation**: Automatically generate and update your project README.
* **⚔️ AI Conflict Resolution**: Let AI help you resolve complex merge conflicts intelligently.
* **🌱 AI Smart Branch**: Create new branches with contextually relevant names.
* **🏷️ AI NL Alias**: Translate English requests into powerful Git commands/aliases.
* **👥 Team Power-Ups**: Generate professional release notes and analyze stale branches.

---

   - Rust toolchain (Cargo) installed.
   - Git installed and configured.
   - An API key for OpenRouter. Set `OPENROUTER_API_KEY` in your environment or `.env` file. Get one at [openrouter.ai](https://openrouter.ai).

### 1. Prerequisites
- **Rust toolchain** (Cargo) installed.
- **Git** installed and configured.
- **Gemini API Key**: Set `GEMINI_API_KEY` in your environment or `.env` file.

### 2. Standard Installation
The recommended way to install `gogit` is using our installer script:

```bash
chmod +x install.sh
./install.sh
```
This builds the binary and installs it to ~/.local/bin/.

3. Build from Source
```Bash

git clone [https://github.com/buka-pitch/gogit.git](https://github.com/buka-pitch/gogit.git)
cd gogit
cargo build --release
cp target/release/gogit ~/.local/bin/
```

4. Install Git Hooks
```Bash

gogit hook install
```

📖 Usage
The gogit CLI offers several commands to integrate AI into your workflow.

<p align="center"> <video src="./video.webm" controls="controls" width="100%" style="border-radius: 8px;"> Your browser does not support the video tag. </video> </p>

1. Commit Generation
Automatically generate a commit message for your staged changes.

Bash

gogit commit
2. Pull Request Description
```Bash

gogit pr
```
3. Code Review
Get an AI-powered review of your staged code for bugs and security issues.

```Bash

gogit review
```
4. Refactoring
Use AI to refactor a specific file based on an instruction.

```Bash

gogit fix src/main.rs "Improve error handling by using Result and ? operator"
```

5. Git History Search
Perform a semantic search across your Git history using natural language.

```Bash

gogit search "user authentication"
```

6. AI Conflict Resolution
Check if merging would result in conflicts and let AI resolve them.

```Bash

gogit check --base main
(... sections 7 through 12 follow the same format ...)

⚙️ Configuration
gogit is configured via ~/.config/gogit/config.toml.

Ini, TOML

# Automatically analyze commits since last tag and save to RELEASE_NOTES.md

gogit release

### 10. AI Repo Navigator (Explain)

Get answers to your questions about the codebase.

# Ask about the architecture or where a feature is implemented

gogit explain "How is the encryption handled in the backend?"

### 11. Smart Stale Branch Analysis

Intelligently identify redundant branches that are safe to delete.

# Analyze branches relative to main

gogit stale

### 12. Documentation Management

Manage your project's documentation, including updating the README or generating doc-comments.

# Update README.md (Interactive: uses staged changes or prompts for full scan)

gogit doc

### 13. AI Model Management

Browse and selection from available free AI models.

# List and choose a free model

gogit models

---

**Configuration:**

`gogit` is configured via a `config.toml` file located in `~/.config/gogit/config.toml`.

Example `config.toml`:

```toml
model = "openai/gpt-4o-mini" # Or browse with 'gogit models'
api_key = "sk-or-v1-..."     # Optional: use OPENROUTER_API_KEY env var instead
commit_prompt = "Generate a concise commit message summarizing these changes."
pr_prompt = "Create a detailed PR description highlighting the problem, solution, and any potential impacts."
```

### 🎯 Free Models via OpenRouter

`gogit` now supports OpenRouter, giving you access to dozens of models! Some popular free options:

- `google/gemini-2.0-flash-exp:free`
- `meta-llama/llama-3.1-70b-instruct:free`
- `mistralai/mistral-7b-instruct:free`

Use `gogit models` to see the full, up-to-date list.

---

**Git Hook Management:**

Install or uninstall Git hooks for automated workflows.

# Install the necessary Git hooks

gogit hook install

# Uninstall the Git hooks

gogit hook uninstall

## 🗺️ Roadmap & Community

We're building the future of Git workflows together! Here are some things we're looking to implement:

- Support for more AI providers (Claude, GPT-4, Llama).
- Rich interactive TUI for conflict resolution.
- Performance optimizations for large monorepos.
- More project-specific context (e.g., analyzing `package.json`, `requirements.txt`).

```
