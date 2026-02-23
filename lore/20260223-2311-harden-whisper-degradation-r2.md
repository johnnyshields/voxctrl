# Harden: Whisper Degradation R2

**Date**: 2026-02-23
**Scope**: `crates/voxctrl-stt/src/whisper_native.rs`
**Reviewing**: Commit `a83ed1f` (per-inference model rebuild + hallucination guards)

## Findings

| # | Opportunity | Effort | Impact |
|---|-------------|--------|--------|
| 1 | Remove redundant `drop(_verify)` — use `let _ =` idiom | Quick | Low |
| 2 | Fix repetition detector off-by-one (triggers at 4th token, not 3rd) | Quick | High |
| 3 | Add unit tests for hallucination guard logic (duration limit, repetition) | Easy | High |
| 4 | Add unit test for `model_to_repo` branching logic | Quick | Low |

## Key Issue: Repetition Detector Off-by-One (#2)

`consecutive_repeats` starts at 0 and increments on seeing a duplicate. With `MAX_TOKEN_REPEATS = 3`, the guard fires after the 4th consecutive same-token, not the 3rd as intended. Fix: rename to `MAX_CONSECUTIVE_DUPLICATES = 2` (allow 2 dupes = 3 total).

## Implementation (all 4 implemented)

1. **`drop(_verify)` → `let _ =`** — single-line cleanup
2. **Off-by-one fix** — renamed `MAX_TOKEN_REPEATS` → `MAX_CONSECUTIVE_DUPLICATES = 2`, fixed log message to report actual total (`consecutive_repeats + 1`)
3. **Hallucination guard tests** — `duration_token_limit` (0s, 0.5s, 2s, 15s, 30s) + repetition detector (no repeats, 1 dup continues, 2 dups halt, reset between runs, empty/single input)
4. **`model_to_repo` tests** — short name → `openai/whisper-{name}`, full repo ID passthrough

---

## Shared Module Analysis: Hallucination Guards

**Date**: 2026-02-23
**Decision**: Skip extraction — guards stay in `whisper_native.rs`

### Question

Should the hallucination guards (repetition detector, duration-proportional token limit, non-Latin detector) be extracted into a shared module for reuse by both Whisper and Voxtral backends?

### Analysis

**Whisper** has three guards in its greedy decode loop (token-by-token):

| Guard | Granularity | How it works |
|-------|-------------|--------------|
| Repetition detector | Per-token | Counts consecutive duplicate tokens; halts after `MAX_CONSECUTIVE_DUPLICATES` (2) |
| Duration-proportional token limit | Per-inference | Caps decode steps to `duration_secs * 15`, min 10, max 224 |
| Non-Latin detector | Per-token | Halts on CJK/Hangul/Hiragana/Katakana when `language_is_english` |

**Voxtral** has **no guards**. Its transcription is a single opaque call:
```rust
let token_ids = model.transcribe_streaming(mel_tensor, t_embed);
```
The upstream `voxtral_mini_realtime` crate handles the decode loop internally — there is no token-by-token hook point where guards could be injected.

### Why extraction doesn't make sense yet

1. **Token-level guards can't apply to Voxtral** — the decode loop is opaque. The repetition detector and non-Latin detector both operate per-token during greedy decoding, which only Whisper exposes.

2. **Post-hoc text guards are possible but unneeded** — `contains_non_latin()` could run on Voxtral's final output string, as could word-level repetition detection and output-length-vs-duration checks. But Voxtral hasn't exhibited hallucination issues in practice.

3. **whisper.cpp also lacks guards** — the other Whisper backend (`whisper_cpp.rs`) delegates to the C++ library internally, same opaque pattern as Voxtral. Extracting guards wouldn't help it either.

4. **No shared utilities exist today** — each backend is self-contained and feature-gated. Audio loading, resampling, and tokenization all use different libraries per backend. There's no existing shared module to build on.

5. **YAGNI** — creating an abstraction for one consumer (whisper-native) adds complexity without value.

### Revisit triggers

Extract shared guards if any of these occur:
- Voxtral starts hallucinating in production
- A new backend is added that also exposes a token-level decode loop
- `contains_non_latin` or duration-limit logic is duplicated elsewhere
