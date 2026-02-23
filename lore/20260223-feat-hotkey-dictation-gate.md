# feat-hotkey-dictation-gate

**Date**: 2026-02-23
**Branch**: `feat-hotkey-dictation-gate/supervisor`

## Summary

Two changes to hotkey behavior:

### A. Main app hotkey safety fix (`hotkey.rs`)

`handle_hotkey_event` now filters by `HotKeyState::Pressed`, early-returning on
`Released` events. This prevents double-toggling on platforms where `global_hotkey`
delivers both Pressed and Released events.

### B. Testbed hotkey gating (`model_table.rs`)

When the dictation hotkey is active (not bypassed AND registered), Step 4 (STT) in
the testbed is gated by the hotkey:

1. **Event loop** (`update()`): Tracks `dict_hotkey_toggled` flag. After draining
   events, if the dictation hotkey was pressed and bypass is off, toggles recording
   (start or stop+transcribe).

2. **Step 4 UI**: When hotkey gating is active:
   - Not recording: disabled "Press hotkey..." button (instead of "Record")
   - Recording: disabled "Recording... press hotkey" button (instead of "Stop & Transcribe")
   - "Load File..." button remains always available

## Files Changed

| File | Change |
|------|--------|
| `crates/voxctrl/src/hotkey.rs` | Import `HotKeyState`, filter Pressed-only in `handle_hotkey_event` |
| `crates/voxctrl/src/ui/model_table.rs` | Hotkey-gated recording toggle + UI button changes in Step 4 |
