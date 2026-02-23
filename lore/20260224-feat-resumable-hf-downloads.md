# Resumable HuggingFace Downloads with Content-Length Validation

**Date**: 2026-02-24
**Branch**: `feat-voxtral-download/supervisor`

## Problem

Model downloads from HuggingFace silently produce corrupt/partial files. When the
CDN drops the connection mid-transfer, `reader.read()` returns 0 (EOF), and the code
treats it as success. The partial file passes the 1 KB minimum check, gets left at
the final path, and `scan_cache()` marks it "Downloaded" because `has_all_files()`
only checks file existence.

## Solution

Rewrite `download_model_files()` in `model_table.rs` with four improvements:

1. **Atomic partial files** — download to `{filename}.partial`, rename only on success.
   Incomplete downloads never appear as complete to the cache scanner.
2. **HTTP Range resume** — check for existing `.partial` file, send `Range: bytes=N-`
   header. Handle 206 (append) and 200 (restart from scratch).
3. **Content-Length validation** — compare bytes written vs `Content-Length` header
   after read loop. Detect premature EOF instead of silently accepting it.
4. **Retry with backoff** — 3 attempts per file, 2s/5s delays. Partial files enable
   automatic resume on retry.

## Files Changed

| File | Change |
|------|--------|
| `crates/voxctrl/src/ui/model_table.rs` | Rewrite `download_model_files()` |
