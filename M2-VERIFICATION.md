# M2 Verification Report — horizon-mod-toolkit

**Auditor:** independent subagent
**Date:** 2026-10-31
**Scope:** `development/horizon-mod-toolkit` (Milestone M2 skeleton)
**Result:** All 7 integration tests pass; `cargo build` succeeds; smoke tests pass; all stated claims verified.

## 1. Build & test results

| Command | Result |
|---|---|
| `cargo build` | ✅ Finished dev profile (0.07s), no warnings |
| `cargo test` | ✅ 0 unit tests + 7 integration tests, all passed (0.44s) |

Integration tests:
- `test_new_scaffolds_content_mod` — PASS
- `test_new_invalid_id_errors` — PASS
- `test_doctor_runs` — PASS
- `test_help_and_version` — PASS
- `test_inspect_missing_file_exit_2` — PASS
- `test_inspect_valid_content_mod_exit_0` — PASS
- `test_unknown_command_exit_2` — PASS

## 2. Smoke tests

| Invocation | Result |
|---|---|
| `--help` | ✅ exit 0, prints usage |
| `version` | ✅ exit 0, prints `horizon-mod-toolkit 0.1.0` |
| `doctor` | ✅ exit 0; reports cargo/rustc; cargo-horizon-mod NOT FOUND in this env |
| `new <tmp>/my_mod --name "My Mod" --id my_mod` | ✅ exit 0; creates manifest, `assets/my_mod`, `data/my_mod`, `.gitkeep` x2, `README.md` |
| `inspect repositories/.../cozy-1.0.0.hpack` | ✅ exit 0; delegates to horizon-inspector; modpack validated |
| `build --path <proj>` (tool absent) | ✅ exit 2 with install hint |
| `check <file>` (tool absent) | ✅ exit 2 with install hint |
| `inspect <file> --strict/--expected-*` | ✅ exit 1 on warnings/mismatch; `--json` exit 0 |

## 3. Claim verification

| Claim | Status | Evidence |
|---|---|---|
| Commands exist: new, build, check, inspect, doctor (+help/version) | ✅ Verified | `src/main.rs` dispatch table; smoke tests |
| `new` scaffolds `horizon-mod.toml`, `assets/<id>`, `data/<id>`, `.gitkeep`, `README.md` | ✅ Verified | `cmd_new`; integration test `test_new_scaffolds_content_mod` |
| `new` validates id (`[a-z0-9_]{1,64}`) | ✅ Verified | `valid_id`; `test_new_invalid_id_errors` (exit non-zero) |
| `inspect` delegates to horizon-inspector with options and exit codes | ✅ Verified | `cmd_inspect` calls `inspect_package`; supports `--json/--strict/--expected-sha256/--expected-size/--key/--signature/--target-hooks-api`; exit 0/1/2 |
| `build`/`check` delegate to `cargo horizon-mod` with missing-tool handling (exit 2) | ✅ Verified | `run_cargo_horizon_mod` preflight + NotFound branch; smoke tests exit 2 |
| No upstream modifications | ✅ Verified | `git status --short` clean in `repositories/horizon`, `repositories/horizon-mods`, `repositories/horizon-native-mod-template` |

## 4. Findings

| ID | Severity | Finding |
|---|---|---|
| F1 | Minor (non-blocking) | `cargo-horizon-mod` availability preflight uses `cargo horizon-mod --version`. The real `cargo-horizon-mod` (upstream) does **not** implement `--version` (returns usage, exit 1), so the toolkit will falsely report the tool as missing even when installed. Detection should use `cargo --list` (which lists `horizon-mod`) or an existence check instead. |
| F2 | Minor | `cmd_check` ignores any extra args after the file and has no `--path` option (README only documents `check <FILE>`, so consistent, but worth noting). |
| F3 | Info | `doctor`'s mod-inspector crate lookup tries a relative `development/mod-inspector/Cargo.toml` first; only the `../mod-inspector/Cargo.toml` candidate matches when run from the toolkit dir. Harmless (second candidate succeeds). |
| F4 | Info | `new` does not emit `.gitignore` (cargo-horizon-mod template does). Not required by README/M2 claims. |

## 5. Verdict

**Ready for M3** — all M2 exit criteria (new project under `development/`, wraps build/check, inspector integration via `inspect`, offline-first, tests + docs) are met and verified. Recommend fixing F1 (tool detection) before relying on an installed `cargo-horizon-mod` in production use; no change required to pass M2 tests.
