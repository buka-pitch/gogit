<p align="center">
  <img src="img1.png" width="150" alt="gogit Logo" object-fit="cover">
</p>

# gogit CLI

<p align="center">
  <img src="img.png" width="100%" alt="gogit Cover" object-fit="cover">
</p>

gogit is a command-line interface tool that integrates AI capabilities into your Git workflow, enhancing productivity and code quality. It leverages AI to generate commit messages, draft pull request descriptions, review code for potential issues, refactor code, and semantically search your Git history.

## Features

- **AI-Powered Conflict Resolution:** Not just a check! Automatically resolve merge conflicts using AI that understands the context of both branches.
- **Smart Branching:** Describe your next task in plain English, and let AI suggest and create a perfectly named branch for you.
- **Natural Language Git Aliases:** Translate complex Git needs (e.g., "show last 5 commits by Bob") into native Git commands and save them as aliases.
- **AI-Powered Commit Messages:** Automatically generate descriptive commit messages based on your staged changes.
- **AI-Generated PR Descriptions:** Create comprehensive pull request descriptions with smart title generation.
- **AI Code Review:** Get AI-driven analysis of your staged code for potential bugs, security vulnerabilities, and logic issues.
- **AI-Assisted Refactoring:** Improve your code by providing natural language instructions for AI-powered refactoring.
- **Semantic Git History Search:** Search your Git history using natural language queries to find relevant changes.
- **Documentation Management:** Easily update your `README.md` file or generate doc-comments with AI assistance.
- **Git Hook Management:** Automate AI actions during your Git workflow (e.g., `prepare-commit-msg`).

## Installation

1.  **Prerequisites:**

    - Rust toolchain (Cargo) installed.
    - Git installed and configured.
    - An API key for the AI model provider (e.g., Gemini). Set the environment variable `GEMINI_API_KEY`.

2.  **Build from Source:**
    git clone https://github.com/your-username/gogit.git
    cd gogit
    cargo build --release
    The compiled binary will be located at `target/release/gogit`. You can then move this binary to your system's PATH for easy access.

3.  **Install Git Hooks (Optional but Recommended):**
    To enable features like automatic commit message generation, install the Git hooks:
    gogit hook install
    This will create a `prepare-commit-msg` hook in your `.git/hooks` directory.

## Usage

The `gogit` CLI offers several commands to integrate AI into your Git workflow.

### 1. Commit Generation

Automatically generate a commit message for your staged changes.

# Generate AI commit message for staged changes

gogit commit

### 2. Pull Request Description Generation

Generate a description for your pull request.

# Generate AI PR description for the current branch based on its diff with the base branch

gogit pr

### 3. Code Review

Get an AI-powered review of your staged code.

# Review staged changes for bugs and security issues

gogit review

### 4. Refactoring

Use AI to refactor a specific file based on an instruction.

# Refactor the 'src/main.rs' file to improve error handling

gogit fix src/main.rs "Improve error handling by using Result and ? operator"

# Refactor the 'utils.py' file to add type hints

gogit fix utils.py "Add type hints to all function signatures"

### 5. Git History Search

Perform a semantic search across your Git history using natural language.

# Find commits related to user authentication changes

gogit search "user authentication"

# Search for commits that involved performance optimizations

gogit search "performance improvements"

### 6. AI Conflict Resolution

Check if merging your current branch into a base branch would result in any conflicts, and optionally let AI resolve them for you.

# Check for conflicts against the 'main' branch

gogit check --base main

# If conflicts are found, gogit will offer to resolve them using AI.

### 7. Smart Branching

Create a new branch with an AI-suggested name based on your task.

# Describe your task to create a branch

gogit branch "fix the navigation bar bug on mobile"

### 8. Natural Language Aliases

Translate your Git requests into commands and optionally save them.

# Get a command for a specific need

gogit alias "list all files changed in the last 2 days"

### 9. Documentation Management

Manage your project's documentation, including updating the README or generating doc-comments.

# Update README.md (Interactive: uses staged changes or prompts for full scan)

gogit doc

### 7. Git Hook Management

Install or uninstall Git hooks for automated workflows.

# Install the necessary Git hooks

gogit hook install

# Uninstall the Git hooks

gogit hook uninstall

---

**Configuration:**

`gogit` is configured via a `config.toml` file located in `~/.config/gogit/config.toml`.

Example `config.toml`:

```toml
model = "gemini-2.5-flash-lite" # Or your preferred model
commit_prompt = "Generate a concise commit message summarizing these changes."
pr_prompt = "Create a detailed PR description highlighting the problem, solution, and any potential impacts."
```
