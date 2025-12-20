<p align="center">
  <img src="img1.png" width="150" alt="gogit Logo">
</p>

# gogit CLI

<p align="center">
  <img src="img.png" width="100%" alt="gogit Cover">
</p>

**gogit** is a command-line interface tool that integrates AI capabilities into your Git workflow. It leverages AI to generate commit messages, draft pull request descriptions, review code, and semantically search your Git history.

---

## ✨ Key Features

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

## 🚀 Installation

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

model = "gemini-2.5-flash-lite"
commit_prompt = "Generate a concise commit message summarizing these changes."
pr_prompt = "Create a detailed PR description highlighting the problem and solution."

```
