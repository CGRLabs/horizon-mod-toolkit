# horizon-mod-toolkit

Mod Toolkit CLI skeleton (Milestone M2) for Horizon content mods.

## Overview

`horizon-mod-toolkit` is a small, offline-first CLI that helps scaffold, build, check, and inspect Horizon content mods. It delegates building/checking to `cargo-horizon-mod` and inspection to `horizon-inspector`.

## Commands

- `new <PATH> [--name "Name"] [--id <id>]` — scaffold a content mod project.
- `build [--path <dir>]` — invoke `cargo horizon-mod build` in the target dir.
- `check <FILE>` — invoke `cargo horizon-mod check <FILE>`.
- `inspect <FILE> [OPTIONS]` — delegate to `horizon-inspector`; supports `--json`, `--strict`, `--expected-sha256`, `--expected-size`, `--key`, `--signature`, `--target-hooks-api`.
- `compat <FILE> [--target-hooks-api N]` — print compatibility summary and verdict.
- `doctor` — print toolchain status.
- `--help`, `--version`.

## Exit codes

- `0` success
- `1` validation failure
- `2` usage error or missing tool

## Dependencies

- `cargo` and (for build/check) `cargo-horizon-mod` (`cargo install cargo-horizon-mod`).
- `horizon-inspector` crate (path dependency `../mod-inspector`).

## Folder Layout

- `src/` - Rust source code for the CLI
- `mods/` - Place your `.hmod` content mods here
- `shaders/` - Place your `.hshader` shader packs here

## Build

```sh
cargo build
cargo test
```
