# horizon-mod-toolkit

Mod Toolkit CLI for Horizon content mods and shader packs.

## Overview

`horizon-mod-toolkit` is a small, offline-first CLI that helps scaffold, build, check, and inspect Horizon content mods (`.hmod`) and shader packs (`.hshader`). It delegates building/checking to `cargo-horizon-mod` and inspection to `horizon-inspector`.

## Folder Layout

- `src/` - Rust source code for the CLI
- `mods/` - Place your `.hmod` content mods here
- `shaders/` - Place your `.hshader` shader packs here

## Quick Install

### Prerequisites

1. **Rust toolchain** — install from https://rustup.rs/
2. **`cargo-horizon-mod`** — required for `build` and `check`:
   ```sh
   cargo install cargo-horizon-mod
   ```
3. **`horizon-mod-inspector`** — required for `inspect` and `compat`. Clone it alongside
   this repo so the Cargo path dependency resolves:
   ```sh
   git clone https://github.com/CGRLabs/horizon-mod-toolkit.git
   git clone https://github.com/CGRLabs/horizon-inspector.git
   # The inspector checkout must sit next to the toolkit checkout:
   #   horizon-mod-toolkit/
   #   horizon-inspector/
   ```

### Build

From inside `horizon-mod-toolkit`:

```sh
cargo build --release
cargo test
```

The release binary is at `target/release/horizon-mod-toolkit`.

## Commands

- `new <PATH> [--name "Name"] [--id <id>]` - scaffold a content mod project
- `build [--path <dir>]` - invoke `cargo horizon-mod build` in the target dir
- `check <FILE>` - invoke `cargo horizon-mod check <FILE>`
- `inspect <FILE> [OPTIONS]` - delegate to `horizon-inspector`; supports `--json`, `--strict`, `--expected-sha256`, `--expected-size`, `--key`, `--signature`, `--target-hooks-api`
- `compat <FILE> [--target-hooks-api N]` - print compatibility summary and verdict
- `doctor` - print toolchain status
- `--help`, `--version`

## Exit codes

- `0` success
- `1` validation failure
- `2` usage error or missing tool

## Dependencies

- `cargo` and (for build/check) `cargo-horizon-mod` (`cargo install cargo-horizon-mod`).
- `horizon-inspector` crate (path dependency `../mod-inspector`).

## Build

```sh
cargo build
cargo test
```

## Example Workflow

```sh
# 1. Create a new content mod
horizon-mod-toolkit new my_mod --name "My Mod" --id my_mod

# 2. Add your data/ and assets/ folders to my_mod/

# 3. Build the .hmod
horizon-mod-toolkit build --path my_mod

# 4. Check it (validates against registry rules)
horizon-mod-toolkit check my_mod/target/horizon-mod/my_mod.hmod

# 5. Inspect it (security report, JSON output)
horizon-mod-toolkit inspect my_mod/target/horizon-mod/my_mod.hmod --json
```

## AI Agent Usage

This CLI is designed to be easy for AI agents to call. All commands return clear exit codes (`0` = success, `1` = validation failure, `2` = usage error). Use `--json` for machine-readable output from `inspect`.