# Repository Guidelines

This is a CLI tool for YouTrack.

## Project Structure & Module Organization
- `src/main.rs`: CLI entrypoint and command routing.
- `src/app/`: app-layer command args, parsing, rendering, and context helpers.
- `src/youtrack/mod.rs`: high-level YouTrack client logic used by CLI commands.
- `src/config/`: config loading and environment overrides (`YOUTRACK_URL`, `YOUTRACK_TOKEN`).
- `src/output/`: table/JSON rendering helpers.
- `src/cli/`: interactive setup flow.
- `api/`: generated Rust API client (`api/src/apis`, `api/src/models`, `api/docs`).
