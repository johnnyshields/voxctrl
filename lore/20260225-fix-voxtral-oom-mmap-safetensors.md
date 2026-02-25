# Fix Voxtral OOM: mmap safetensors file

**Date**: 2025-02-25
**Type**: Bug fix (OOM during model loading)

## Problem

Loading the Voxtral model (8.9 GB `consolidated.safetensors`) failed with
"memory allocation of 113246208 bytes failed". Root cause:

1. `fs::read()` loaded the entire 8.9 GB file into `Vec<u8>` on the heap
2. Each tensor was converted BF16 -> f32 (2x expansion), accumulating ~17.8 GB
3. Peak memory during loading: ~25.3 GB (file + tensors coexisting)
4. Heap fragmentation caused a 113 MB allocation to fail even with 64 GB RAM

## Solution

Memory-map the safetensors file instead of `fs::read()`. The OS pages in data
on demand, eliminating the 8.9 GB heap allocation. Peak drops to ~17.8 GB.

### Changes in `voxtral-mini-realtime-rs` fork

- **Cargo.toml**: Added `memmap2 = "0.9"` dependency
- **src/models/weights.rs**:
  - Added `BytesBacking` enum (Owned / Mapped variants)
  - `from_file()` now uses `memmap2::Mmap::map()` instead of `fs::read()`
  - `from_bytes()` uses the `Owned` variant (preserves WASM compatibility)
  - No API changes — same `OwnedSafeTensors` public interface

### Changes in voxctrl

- **crates/voxctrl-stt/Cargo.toml**: Pinned to fork commit `a1218a8`

## Verification

- `cargo build` in voxctrl workspace: compiles
- `cargo test` in voxtral-mini-realtime-rs: all 113 tests pass
- No API changes needed in voxctrl — `from_file()` signature unchanged
