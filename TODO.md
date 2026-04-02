# Project TODO

## High Priority

- [ ] Add `rust-toolchain.toml` and pin Rust to `1.88.0` so local development matches CI and dependency requirements.
- [ ] Fix local build reliability and verify the project builds cleanly on a fresh Linux setup.
- [ ] Add automated tests for AI provider behavior in `src/ai.rs`.
- [ ] Add tests for tool execution flow in `src/agent.rs`.
- [ ] Add tests for tool argument parsing and safety checks in `src/tools.rs`.
- [ ] Add tests for config load/save behavior in `src/config.rs`.
- [ ] Improve Ollama tool-calling support validation and error messages for unsupported models.
- [ ] Document which providers/models are expected to support tool calling.

## Architecture

- [ ] Refactor `src/ai.rs` into smaller provider-focused modules.
- [ ] Split AI code into files such as `src/ai/types.rs`, `src/ai/openrouter.rs`, `src/ai/gemini.rs`, and `src/ai/ollama.rs`.
- [ ] Introduce explicit provider capability flags such as `supports_tools` and `supports_streaming_tools`.
- [ ] Reduce branching complexity in the AI client by moving provider-specific request/response mapping into dedicated modules.

## Tooling And Safety

- [ ] Replace loose JSON parsing in tool handlers with typed argument structs where practical.
- [ ] Improve validation errors for malformed tool arguments.
- [ ] Add stronger path safety checks for file-writing tools.
- [ ] Improve `run_command` safety with better timeout enforcement and clearer command restrictions.
- [ ] Review shell execution paths for dangerous or ambiguous behavior.

## CLI And UX

- [ ] Add a `gogit doctor` command to check Rust version, installed binary path, provider setup, and common environment issues.
- [ ] Add a `gogit self-update` or release-install helper workflow for easier upgrades.
- [ ] Improve chat UX by showing clearer tool-call progress and tool-result summaries.
- [ ] Show active provider/model more clearly during chat sessions.
- [ ] Improve model selection UX with labels for local vs remote models and tool-calling compatibility.

## CI/CD

- [ ] Add `cargo fmt --check` to GitHub Actions.
- [ ] Add `cargo clippy` to GitHub Actions.
- [ ] Add a dedicated CI workflow for pull requests and branch validation.
- [ ] Expand release automation to support more Linux targets such as `aarch64`.
- [ ] Consider adding macOS and Windows release builds later.
- [ ] Add required status checks for build, test, lint, and formatting in GitHub.

## Documentation

- [ ] Clean up and reorganize `README.md` to reduce duplicated sections and setup drift.
- [ ] Add a clear installation section for downloading release binaries.
- [ ] Add a provider setup guide covering OpenRouter, Gemini, and Ollama.
- [ ] Add a tool-calling compatibility section with known caveats.
- [ ] Add troubleshooting steps for stale installed binaries vs workspace binaries.

## Nice To Have

- [ ] Add release badges and binary download badges to the README.
- [ ] Add richer release notes and changelog automation.
- [ ] Add telemetry-free diagnostics for debugging provider issues locally.
- [ ] Add optional verbose debug logging for AI request/response flows.
