# feat-hotkey-ui — Hotkey Capture Widget + Test Tab

**Date:** 2026-02-23
**Branch:** feat-hotkey-ui/supervisor

## Summary

Two hotkey UX improvements in the Settings GUI:

1. **Key capture widget** replaces the plain text input for hotkey configuration. Shows a button that enters "Press keys..." capture mode, detects key combos via egui's input API, and builds a shortcut string. Includes a "Super/Win" checkbox workaround since egui 0.30 on Windows/Linux doesn't expose the Super key in its modifier state.

2. **Test tab** with a live hotkey indicator. Registers the configured hotkey via `GlobalHotKeyManager`, shows yellow when ready, green when the hotkey is actively pressed. Handles registration conflicts gracefully (the main voxctrl process may already hold the hotkey).

## Files modified

- `src/hotkey.rs` — Make `parse_shortcut` public
- `src/ui/model_table.rs` — Capture widget, Test tab, new state fields and helper functions

## Key design decisions

- **Super key workaround**: egui's `Modifiers` on Windows/Linux maps `command` to `ctrl`, discarding Super. A separate checkbox toggles "Super+" in the shortcut string.
- **Solo implementation**: Both features share the same file and struct. No benefit from parallelization.
- **Tab-based hotkey lifecycle**: Test tab registers the hotkey on entry, unregisters on exit, avoiding permanent conflicts with the main app.
