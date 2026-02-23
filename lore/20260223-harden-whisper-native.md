# Harden: Whisper-Native Transcription Code

**Date**: 2026-02-23
**Scope**: Cleanup pass on whisper-native after rapid iteration fixed multiple bugs

## Changes

1. **Demote debug logs** — `[whisper-dbg]` internal decode-step logs from `info!` to `debug!` in `whisper_native.rs`
2. **Consolidate testbed transcription** — Extract `transcribe_via_server_or_direct()` helper in `model_table.rs` to deduplicate server+fallback logic
3. **Extract audio device resolution** — `resolve_device_and_config()` helper in `audio.rs` to deduplicate device+config lookup between `start_capture` and `start_test_capture`
4. **Unit tests for resample/audio_stats** — Pure function tests in `whisper_native.rs`
5. **Unit tests for suppress masks** — Test dimensions and -inf positions for `suppress_mask` and `begin_suppress_mask`
6. **Remove stale doc comment** — Delete outdated "Returns (stream, level_receiver)" doc on `start_test_capture`

## Files Modified

- `crates/voxctrl-stt/src/whisper_native.rs` — #1, #4, #5
- `crates/voxctrl/src/ui/model_table.rs` — #2
- `crates/voxctrl-core/src/audio.rs` — #3, #6
