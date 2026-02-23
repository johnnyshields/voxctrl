# Voxtral Local Tests + WSL2 Model Discovery Fix

**Date**: 2026-02-23
**Branch**: feat-voxtral-local-tests/supervisor

## Problem

Voxtral Mini 4B model is downloaded to Windows HF cache (`/mnt/c/Users/John/AppData/Local/huggingface/hub/`) but the cache scanner only checks the Linux path (`~/.cache/huggingface/hub/`), causing "Model not downloaded" errors on WSL2.

## Changes

1. **Fix cache scanner WSL2 detection** — scan `/mnt/c/Users/*/AppData/Local/huggingface/hub/` as fallback in `cache_scanner.rs`
2. **Create `test_voxtral.rs` example** — local smoke test mirroring `test_whisper.rs` pattern
3. **Add unit tests** — pending state, error handling, nonexistent dir fallback in `voxtral_native.rs`

## Files

- `crates/voxctrl-core/src/models/cache_scanner.rs` — WSL2 path scanning
- `crates/voxctrl-stt/examples/test_voxtral.rs` — new example
- `crates/voxctrl-stt/Cargo.toml` — example entry
- `crates/voxctrl-stt/src/voxtral_native.rs` — unit tests
