<p align="center">
  <img src="img1.png" width="150" alt="gogit Logo" object-fit="cover">
</p>

# gogit CLI

<p align="center">
  <img src="img.png" width="100%" alt="gogit Cover" object-fit="cover">
</p>

gogit is a command-line interface tool that integrates AI capabilities into your Git workflow, enhancing productivity and code quality. It leverages AI to generate commit messages, draft pull request descriptions, review code for potential issues, refactor code, and semantically search your Git history.

## Features

- **AI-Powered Commit Messages:** Automatically generate descriptive commit messages based on your staged changes.
- **AI-Generated PR Descriptions:** Create comprehensive pull request descriptions to facilitate better code reviews and understanding.
- **AI Code Review:** Get AI-driven analysis of your staged code for potential bugs, security vulnerabilities, performance issues, and adherence to best practices.
- **AI-Assisted Refactoring:** Improve your code by providing natural language instructions for AI-powered refactoring of specific files.
- **Semantic Git History Search:** Search your Git history using natural language queries to find relevant commits and changes.
- **Merge Conflict Check:** Check for merge conflicts against any base branch without side effects before performing the actual merge.
- **Documentation Management:** Easily update your `README.md` file or generate doc-comments for your project with AI assistance.
- **Git Hook Management:** Install and uninstall Git hooks to automate AI actions during your Git workflow (e.g., `prepare-commit-msg`).
- **Configuration:** Customize AI models and prompts through a configuration file.

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

### 6. Check for Merge Conflicts

Check if merging your current branch into a base branch would result in any conflicts. This is a dry-run and does not modify your files.

# Check for conflicts against the 'main' branch

gogit check --base main

### 7. Documentation Management

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
