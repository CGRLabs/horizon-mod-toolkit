//! Horizon Mod Toolkit CLI.
//!
//! A small, offline-first toolkit for scaffolding and managing Horizon content mods.

use horizon_inspector::{InspectorConfig, inspect_package};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = "horizon-mod-toolkit";

fn print_usage() {
    eprintln!(
        r#"{BIN_NAME} v{VERSION} - Horizon Mod Toolkit

USAGE:
    {BIN_NAME} <COMMAND> [OPTIONS]

COMMANDS:
    new <PATH> [--name "Name"] [--id <id>] [--kind content|native]  Scaffold a content or native mod project
    build [--path <dir>]                     Build a content mod (invokes cargo horizon-mod build)
    check <FILE>                             Check a content mod package (invokes cargo horizon-mod check)
    inspect <FILE> [OPTIONS]                 Inspect a mod package (delegates to horizon-inspector)
    compat <FILE> [--target-hooks-api N]     Print compatibility summary for a package
    revocation-check <FILE> --revocation-list <PATH>  Check a package against a revoked.toml
    doctor                                   Print toolchain status
    help                                     Show this help
    version                                  Show version

INSPECT OPTIONS:
    --json                                  Output JSON
    --strict                                Treat warnings as errors
    --expected-sha256 <HEX>                 Verify SHA-256
    --expected-size <BYTES>                 Verify file size
    --key <PUBLIC_KEY>                      Minisign public key
    --signature <PATH>                      Signature file path
    --target-hooks-api <LEVEL>               Target hooks API level
    --revocation-list <PATH>                Check against a local revoked.toml

EXIT CODES:
    0   Success
    1   Validation failure
    2   Usage error or missing tool
"#
    );
}

fn print_version() {
    println!("{BIN_NAME} {VERSION}");
}

// ---------- helpers ----------

fn run_cargo_horizon_mod(args: &[&str], cwd: &Path) -> ExitCode {
    // Check cargo availability first.
    let cargo_check = Command::new("cargo").arg("--version").output();
    if cargo_check.is_err() {
        eprintln!("error: cargo not found on PATH. Install Rust toolchain first.");
        return ExitCode::from(2);
    }

    // Preflight cargo-horizon-mod availability.
    // `cargo --list` lists installed cargo subcommands; if `horizon-mod`
    // shows up, the tool is on PATH and usable. Fall back to checking for
    // the standalone `cargo-horizon-mod` binary.
    let tool_ok = Command::new("cargo")
        .args(["--list"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|line| line.trim_start().starts_with("horizon-mod "))
        })
        .unwrap_or(false)
        || match Command::new("cargo-horizon-mod").arg("--version").output() {
            Ok(_) => true, // binary exists (even if --version isn't implemented upstream)
            Err(e) => e.kind() != std::io::ErrorKind::NotFound,
        };
    if !tool_ok {
        eprintln!(
            "error: cargo-horizon-mod is not installed or not on PATH.\n\
             Install it with: cargo install cargo-horizon-mod"
        );
        return ExitCode::from(2);
    }

    let status = Command::new("cargo")
        .args(std::iter::once("horizon-mod").chain(args.iter().copied()))
        .current_dir(cwd)
        .status();

    match status {
        Ok(s) => {
            if s.success() {
                ExitCode::SUCCESS
            } else {
                // propagate code if possible, else 1
                s.code()
                    .and_then(|c| u8::try_from(c).ok())
                    .map(ExitCode::from)
                    .unwrap_or(ExitCode::from(1))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!(
                    "error: cargo-horizon-mod is not installed or not on PATH.\n\
                     Install it with: cargo install cargo-horizon-mod\n\
                     (or ensure the cargo-horizon-mod binary is available)"
                );
            } else {
                eprintln!("error: failed to run cargo: {e}");
            }
            ExitCode::from(2)
        }
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn derive_id(dir_name: &str) -> String {
    let mut s = String::new();
    for c in dir_name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            s.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' || c == '.' {
            s.push('_');
        }
    }
    // collapse consecutive underscores
    let mut out = String::new();
    let mut prev_us = false;
    for c in s.chars() {
        if c == '_' {
            if prev_us {
                continue;
            }
            prev_us = true;
        } else {
            prev_us = false;
        }
        out.push(c);
    }
    // trim leading/trailing underscores
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "my_mod".to_string()
    } else if out.len() > 64 {
        out[..64].to_string()
    } else {
        out
    }
}

fn title_case_id(id: &str) -> String {
    id.split('_')
        .filter(|p| !p.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => {
                    let mut s = first.to_uppercase().to_string();
                    s.push_str(&chars.as_str().to_lowercase());
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn cmd_new(args: &[String]) -> ExitCode {
    let mut path: Option<String> = None;
    let mut name: Option<String> = None;
    let mut id: Option<String> = None;
    let mut kind: String = "content".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --name requires an argument");
                    return ExitCode::from(2);
                }
                name = Some(args[i].clone());
            }
            "--id" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --id requires an argument");
                    return ExitCode::from(2);
                }
                id = Some(args[i].clone());
            }
            "--kind" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --kind requires an argument (content|native)");
                    return ExitCode::from(2);
                }
                kind = args[i].to_ascii_lowercase();
                if kind != "content" && kind != "native" {
                    eprintln!("error: unknown kind '{}'", kind);
                    return ExitCode::from(2);
                }
            }
            other if !other.starts_with('-') => {
                if path.is_some() {
                    eprintln!("error: multiple paths specified");
                    return ExitCode::from(2);
                }
                path = Some(other.to_string());
            }
            other => {
                eprintln!("error: unrecognized option '{other}'");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let path_str = match path {
        Some(p) => p,
        None => {
            eprintln!("error: new requires a <PATH>");
            return ExitCode::from(2);
        }
    };

    let target = PathBuf::from(&path_str);
    let dir_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_mod");

    let mod_id = match id {
        Some(i) => i,
        None => derive_id(dir_name),
    };

    if !valid_id(&mod_id) {
        eprintln!(
            "error: invalid mod id '{mod_id}' (must match [a-z0-9_]{{1,64}})"
        );
        return ExitCode::from(2);
    }

    let mod_name = name.unwrap_or_else(|| title_case_id(&mod_id));

    // Refuse to clobber existing manifest
    let manifest_name = if kind == "native" { "horizon-native.json" } else { "horizon-mod.toml" };
    let manifest_path = target.join(manifest_name);
    if manifest_path.exists() {
        eprintln!(
            "error: {} already exists; refusing to overwrite",
            manifest_path.display()
        );
        return ExitCode::from(2);
    }

    if let Err(e) = fs::create_dir_all(&target) {
        eprintln!("error: cannot create directory '{}': {e}", target.display());
        return ExitCode::from(2);
    }

    if kind == "native" {
        // Native mod scaffolding
        let native_manifest = format!(
            r#"{{
  "schema": 1,
  "id": "{id}",
  "version": "0.1.0",
  "game": "26.3",
  "horizon": "0000000000000000000000000000000000000000",
  "mixins": "0000000000000000000000000000000000000000",
  "hooks": 1
}}"#,
            id = mod_id
        );
        if let Err(e) = fs::write(&manifest_path, native_manifest) {
            eprintln!("error: cannot write manifest: {e}");
            return ExitCode::from(2);
        }
        let mixin_dir = target.join("mixins").join(&mod_id);
        let src_dir = mixin_dir.join("src");
        if let Err(e) = fs::create_dir_all(&src_dir) {
            eprintln!("error: cannot create directory '{}': {e}", src_dir.display());
            return ExitCode::from(2);
        }
        let mixin_toml = format!(
            r#"[mod]
id = "{id}"
version = "0.1.0"

[compatibility]
horizon_revision = "0000000000000000000000000000000000000000"
"#,
            id = mod_id
        );
        let _ = fs::write(mixin_dir.join("horizon-mixin.toml"), mixin_toml);
        let _ = fs::write(src_dir.join("lib.rs"), "// Native mixin implementation\n");
        let readme = format!("# {name}\n\nA Horizon native mod (unsandboxed).\n\n## Build\n\nRequires Linux, Rust 1.95.0, Python 3.12+.\nStage with tools/horizon.py from horizon-native-mod-template.\n", name = mod_name);
        let _ = fs::write(target.join("README.md"), readme);
        println!("Created native mod at {}", target.display());
        println!("  id:   {}", mod_id);
        println!("  name: {}", mod_name);
        println!("  note: pin horizon/mixins revisions to real 40-hex commits before building");
        return ExitCode::SUCCESS;
    }

    let toml_content = format!(
        r#"id = "{id}"
name = "{name}"
version = "0.1.0"
description = ""
authors = []
# A content mod: blocks, items and mob effects from data/ and assets/, no code (API 0.2).
api = "0.2"
side = "both"
# registries: "May add blocks, items and mob effects to your worlds".
capabilities = ["registries"]
"#,
        id = mod_id,
        name = mod_name
    );

    if let Err(e) = fs::write(&manifest_path, toml_content) {
        eprintln!("error: cannot write manifest: {e}");
        return ExitCode::from(2);
    }

    // assets/<id>/ and data/<id>/
    let assets_dir = target.join("assets").join(&mod_id);
    let data_dir = target.join("data").join(&mod_id);
    for d in [&assets_dir, &data_dir] {
        if let Err(e) = fs::create_dir_all(d) {
            eprintln!("error: cannot create directory '{}': {e}", d.display());
            return ExitCode::from(2);
        }
        let gitkeep = d.join(".gitkeep");
        let _ = fs::write(&gitkeep, "");
    }

    let readme = format!(
        "# {name}\n\nA Horizon content mod.\n\n## Build\n\n```\ncargo horizon-mod build\n```\n",
        name = mod_name
    );
    let _ = fs::write(target.join("README.md"), readme);

    println!("Created content mod at {}", target.display());
    println!("  id:   {}", mod_id);
    println!("  name: {}", mod_name);
    ExitCode::SUCCESS
}

fn cmd_build(args: &[String]) -> ExitCode {
    let mut path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--path" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --path requires an argument");
                    return ExitCode::from(2);
                }
                path = Some(args[i].clone());
            }
            other => {
                eprintln!("error: unrecognized option '{other}'");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let cwd = path
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    run_cargo_horizon_mod(&["build"], &cwd)
}

fn cmd_check(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("error: check requires a <FILE>");
        return ExitCode::from(2);
    }
    // First positional is file; rest ignored (or could pass through)
    let file = &args[0];
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Pass file path to cargo horizon-mod check
    run_cargo_horizon_mod(&["check", file], &cwd)
}

fn cmd_inspect(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("error: inspect requires a <FILE>");
        return ExitCode::from(2);
    }

    // First positional file; collect options from rest
    let mut target_file: Option<String> = None;
    let mut json_output = false;
    let mut strict_mode = false;
    let mut config = InspectorConfig::default();

    let mut i = 0;
    // We'll treat first non-option as file, but file may appear anywhere.
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--json" => json_output = true,
            "--strict" => strict_mode = true,
            "--expected-sha256" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --expected-sha256 requires an argument");
                    return ExitCode::from(2);
                }
                config.expected_sha256 = Some(args[i].clone());
            }
            "--expected-size" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --expected-size requires an argument");
                    return ExitCode::from(2);
                }
                match args[i].parse::<u64>() {
                    Ok(v) => config.expected_size = Some(v),
                    Err(e) => {
                        eprintln!("error: invalid byte count: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            "--key" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --key requires an argument");
                    return ExitCode::from(2);
                }
                config.public_key = Some(args[i].clone());
            }
            "--signature" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --signature requires an argument");
                    return ExitCode::from(2);
                }
                config.signature_path = Some(PathBuf::from(&args[i]));
            }
            "--target-hooks-api" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --target-hooks-api requires an argument");
                    return ExitCode::from(2);
                }
                match args[i].parse::<u32>() {
                    Ok(v) => config.target_hooks_api = Some(v),
                    Err(e) => {
                        eprintln!("error: invalid level: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            "--revocation-list" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --revocation-list requires a file path");
                    return ExitCode::from(2);
                }
                config.revocation_list = Some(PathBuf::from(&args[i]));
            }
            other if !other.starts_with('-') => {
                if target_file.is_some() {
                    eprintln!("error: multiple target files specified");
                    return ExitCode::from(2);
                }
                target_file = Some(other.to_string());
            }
            other => {
                eprintln!("error: unrecognized option '{other}'");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let target = match target_file {
        Some(t) => t,
        None => {
            eprintln!("error: no package file specified");
            return ExitCode::from(2);
        }
    };

    let target_path = Path::new(&target);
    if !target_path.exists() {
        eprintln!("error: target not found: {}", target_path.display());
        return ExitCode::from(2);
    }

    let report = inspect_package(target_path, &config);

    if json_output {
        match report.to_json() {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        print!("{}", report.to_terminal_text());
    }

    if !report.is_valid {
        ExitCode::from(1)
    } else if strict_mode && report.stats.warning_count > 0 {
        eprintln!("error: package inspection passed with warnings and --strict was specified");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_compat(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("error: compat requires a <FILE>");
        return ExitCode::from(2);
    }
    let mut target_file: Option<String> = None;
    let mut config = InspectorConfig::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target-hooks-api" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --target-hooks-api requires an integer level");
                    return ExitCode::from(2);
                }
                match args[i].parse::<u32>() {
                    Ok(v) => config.target_hooks_api = Some(v),
                    Err(e) => {
                        eprintln!("error: invalid level: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            other if !other.starts_with('-') => {
                if target_file.is_some() {
                    eprintln!("error: multiple target files specified");
                    return ExitCode::from(2);
                }
                target_file = Some(other.to_string());
            }
            other => {
                eprintln!("error: unrecognized option '{other}'");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let target = match target_file {
        Some(t) => t,
        None => {
            eprintln!("error: no package file specified");
            return ExitCode::from(2);
        }
    };
    let target_path = Path::new(&target);
    if !target_path.exists() {
        eprintln!("error: target not found: {}", target_path.display());
        return ExitCode::from(2);
    }

    let report = inspect_package(target_path, &config);

    println!("Package:     {}", report.file.file_name);
    println!("Kind:        {}", report.package_kind.as_str());
    println!("Security:    {}", report.security_tier.description());
    println!("Valid:       {}", report.is_valid);
    if let Some(id) = &report.identity.id {
        println!("ID:          {}", id);
    }
    if let Some(ver) = &report.identity.version {
        println!("Version:     {}", ver);
    }
    if let Some(game) = &report.compatibility.game {
        println!("Game:        Minecraft Java {}", game);
    }
    if let Some(api) = &report.compatibility.api {
        println!("API line:    {}", api);
    }
    if let Some(side) = &report.compatibility.side {
        println!("Side:        {}", side);
    }
    if let Some(rev) = &report.compatibility.horizon_revision {
        println!("Horizon rev: {}", rev);
    }
    if let Some(rev) = &report.compatibility.mixins_revision {
        println!("Mixins rev:  {}", rev);
    }
    if let Some(hooks) = report.compatibility.hooks_api {
        println!("Hooks API:   {}", hooks);
    }
    if !report.capabilities.is_empty() {
        println!("Capabilities:{}", report.capabilities.join(", "));
    }

    // Hooks compatibility verdict
    if let Some(target_level) = config.target_hooks_api {
        if let Some(hooks) = report.compatibility.hooks_api {
            if hooks > target_level {
                println!("Compat:      INCOMPATIBLE (hooks {} > target {})", hooks, target_level);
            } else {
                println!("Compat:      OK (hooks {} <= target {})", hooks, target_level);
            }
        } else {
            println!("Compat:      hooks not declared (pinned revision only)");
        }
    } else {
        println!("Compat:      (pass --target-hooks-api to check native hooks)");
    }

    if !report.is_valid {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_revocation_check(args: &[String]) -> ExitCode {
    // Require --revocation-list
    if !args.iter().any(|a| a == "--revocation-list") {
        eprintln!("error: revocation-check requires --revocation-list <PATH>");
        return ExitCode::from(2);
    }
    cmd_inspect(args)
}

fn cmd_doctor() -> ExitCode {
    println!("Horizon Mod Toolkit doctor");
    println!();

    // cargo
    match Command::new("cargo").arg("--version").output() {
        Ok(o) => println!("cargo: {}", String::from_utf8_lossy(&o.stdout).trim()),
        Err(_) => println!("cargo: NOT FOUND"),
    }
    match Command::new("rustc").arg("--version").output() {
        Ok(o) => println!("rustc: {}", String::from_utf8_lossy(&o.stdout).trim()),
        Err(_) => println!("rustc: NOT FOUND"),
    }
    // cargo-horizon-mod
    let chm_found = Command::new("cargo")
        .args(["--list"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|line| line.trim_start().starts_with("horizon-mod "))
        })
        .unwrap_or(false)
        || match Command::new("cargo-horizon-mod").arg("--version").output() {
            Ok(_) => true,
            Err(e) => e.kind() != std::io::ErrorKind::NotFound,
        };
    if chm_found {
        // cargo-horizon-mod is installed; show its usage line (no args -> usage to stderr).
        match Command::new("cargo").args(["horizon-mod"]).output() {
            Ok(o) => {
                let usage = String::from_utf8_lossy(&o.stderr).trim().to_string();
                if !usage.is_empty() {
                    println!("cargo-horizon-mod: {usage}");
                } else {
                    println!("cargo-horizon-mod: found on PATH");
                }
            }
            Err(e) => println!("cargo-horizon-mod: found on PATH but not runnable ({e})"),
        }
    } else {
        println!("cargo-horizon-mod: NOT FOUND (install with: cargo install cargo-horizon-mod)");
    }

    // check a few locations
    let candidates = [
        PathBuf::from("development/mod-inspector/Cargo.toml"),
        PathBuf::from("../mod-inspector/Cargo.toml"),
        PathBuf::from("../../development/mod-inspector/Cargo.toml"),
    ];
    let mut found = false;
    for c in &candidates {
        if c.exists() {
            found = true;
            println!("mod-inspector crate: found ({})", c.display());
            break;
        }
    }
    if !found {
        println!("mod-inspector crate: NOT FOUND");
    }

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        return ExitCode::from(2);
    }

    // Global help/version
    if args.iter().any(|a| a == "--help" || a == "-h") && args.len() == 1 {
        print_usage();
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--version" || a == "-v") && args.len() == 1 {
        print_version();
        return ExitCode::SUCCESS;
    }

    match args[0].as_str() {
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        "version" | "--version" | "-v" => {
            print_version();
            ExitCode::SUCCESS
        }
        "new" => cmd_new(&args[1..]),
        "build" => cmd_build(&args[1..]),
        "check" => cmd_check(&args[1..]),
        "inspect" => cmd_inspect(&args[1..]),
        "compat" => cmd_compat(&args[1..]),
        "revocation-check" => cmd_revocation_check(&args[1..]),
        "doctor" => cmd_doctor(),
        other => {
            eprintln!("error: unknown command '{other}'. Use --help for usage.");
            ExitCode::from(2)
        }
    }
}
