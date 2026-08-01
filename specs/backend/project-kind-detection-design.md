# Project kind detection (CLI vs installable app)

**Status:** implementing  
**Owner:** `crates/signet/src/scan`, `artifact`, `tui`

## Problem

Guided init always offered installable-app adapters (Tauri default). Scanning this repo also preferred nested `demo/fixture` Electron over the root Rust workspace.

## Design

1. **Detect `RustCli`** when a directory has a Cargo package with a binary (`src/main.rs` / `[[bin]]`) or a Cargo workspace root with no UI-app markers in that same directory.
2. **Prefer shallow / non-demo projects** when ranking detections: path depth + penalties for `demo|fixture|example|testdata` path segments.
3. **Suggest `framework = "cli"`** for Rust CLI / workspace tooling; adapter runs `cargo build --release` and discovers host binaries under `target/<profile>/`.
4. **Guided init** scans first, shows detected kind, and confirms (or lets the user override) instead of blank-defaulting to Tauri.

## Proof

- Unit: prefer RustCli at workspace root over Electron under `demo/fixture`
- Unit: select_adapter `"cli"`
- Manual: `signet scan` at Signet repo root → suggested framework `cli`
