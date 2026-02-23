# Harden: Settings Auto-Save + Hot-Reload

## Changes

1. **Extract constants**: `CONFIG_POLL_INTERVAL_MS` (500ms) in main.rs,
   `FLASH_DURATION_MS` / `FLASH_DURATION_SECS` (1500ms / 1.5) in model_table.rs
2. **Guard invalid config**: Skip hot-reload apply when `load_config()` returns
   `Config::default()` but config.json exists with content (transient parse failure)
3. **Config PartialEq tests**: default equality, single-field inequality,
   serialize/deserialize round-trip
4. **SettingsSnapshot::changed_sections tests**: identical→empty, single field→correct
   section, multiple sections→all correct
5. **SharedPipeline concurrency test**: multi-threaded get()/swap() with no panics
