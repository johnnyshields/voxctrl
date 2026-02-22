# Plan: Model Manager + Download Consent for voxctrl

**Date**: 2026-02-22
**Status**: Implementation in progress

## Problem

Native STT backends (whisper-native, voxtral-native) auto-download multi-GB models
from HuggingFace on first use with no user consent. Users need:
1. Prompt before downloading — native dialog asking permission
2. Tabbed model manager window — egui-based GUI to browse, download, delete models

## Architecture

```
Arc<Mutex<ModelRegistry>>  (shared state)
        │
        ├── main.rs: scan cache on startup, consent check before pipeline creation
        ├── tray.rs: "Manage Models..." menu item
        ├── ui/: egui window (separate thread via eframe)
        └── models/: catalog, cache scanner, download, consent
```

## New Modules

### `src/models/` — Model management backend
- `mod.rs` — ModelRegistry, ModelEntry, DownloadStatus types; shared state API
- `catalog.rs` — Hardcoded catalog of known models (repo IDs, sizes, backend mapping)
- `cache_scanner.rs` — Scan HF Hub cache to determine download status
- `consent.rs` — rfd::MessageDialog consent prompt + ensure_model_available()
- `downloader.rs` — Download wrapper with registry status updates

### `src/ui/` — egui model manager window (behind `ui-model-manager` feature)
- `mod.rs` — open_model_manager() spawns eframe on separate thread
- `model_table.rs` — ModelManagerApp implementing eframe::App

## Implementation Phases

1. **Phase 1 — Foundation**: models/ module + consent (catalog, cache scanner, registry, consent, downloader)
2. **Phase 2 — Tray + wiring**: Update tray.rs (menu items + IDs), update main.rs (registry lifecycle, MenuEvent, consent)
3. **Phase 3 — egui window**: ui/mod.rs, ui/model_table.rs, wire "Manage Models..." to open_model_manager()

## New Dependencies

- eframe 0.30 (optional, behind ui-model-manager feature)
- egui 0.30 (optional, behind ui-model-manager feature)
- rfd 0.15 (optional, for consent dialogs)
- dirs 5 (for cache path discovery)
