# gui + tui Feature Flags

**Date:** 2026-02-22

## Goal

Gate the GUI code (tray, global hotkey, winit event loop, egui model manager) behind a `gui` feature flag. Add a `tui` feature flag for a terminal-based alternative using ratatui. Both default-on. At runtime, GUI takes precedence unless `--tui` is passed.

## Architecture

- Shared startup (config, models, pipeline, audio capture) stays in `main()`
- Recording state machine (toggle Idle→Recording→Transcribing) extracted into `src/recording.rs`
- GUI and TUI each get their own event loop
- Module declarations gated by `#[cfg(feature = "...")]`

## Files Changed

| File | Action |
|------|--------|
| Cargo.toml | Add gui/tui features, make GUI deps optional, add ratatui+crossterm |
| src/recording.rs | Create — extracted toggle + transcribe logic |
| src/hotkey.rs | Delegate to recording::toggle_recording |
| src/tui.rs | Create — ratatui scaffold |
| src/main.rs | Rewrite — shared startup + pick_ui_mode + run_gui + dispatch |

## Runtime Behavior

| Features | CLI flag | Result |
|----------|----------|--------|
| gui + tui | (none) | GUI |
| gui + tui | --tui | TUI |
| gui only | --tui | warn, fallback GUI |
| tui only | (any) | TUI |
