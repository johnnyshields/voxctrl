# Fix Whisper Transcription Degradation

**Date**: 2026-02-23
**Branch**: fix-whisper-degradation/supervisor
**Status**: Plan

## Problem

Whisper-native (candle) transcription degrades over repeated calls. First transcription is correct but subsequent ones produce dots, Chinese characters, or garbled output. Latency increases for degraded calls (14s vs instant).

## Root Cause Analysis

The `WhisperNativeTranscriber` reuses a single `Mutex<m::model::Whisper>` across all calls. While candle 0.8.4's `flush_kv_cache=true` on decode step 0 should properly reset the cross-attention KV cache, and self-attention doesn't cache, there may be subtle state accumulation. In contrast, `WhisperCppTranscriber` creates fresh `WhisperState` per call, avoiding this issue entirely.

Key code path: `whisper_native.rs:run_inference()` → locks model → `encoder.forward()` → greedy decode loop with `flush=true` on step 0 only.

## Fix

1. **Add `model.reset_kv_cache()` before each inference** — defensive cache clearing, negligible cost
2. **Add diagnostic logging** — encoder output stats, inference counter
3. **Add stability test** — repeated transcription consistency check

## Files

- `crates/voxctrl-stt/src/whisper_native.rs` — primary changes
