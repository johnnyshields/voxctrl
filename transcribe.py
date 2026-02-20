#!/usr/bin/env python3
"""
DJI Mic Transcription - powered by faster-whisper (local, offline)

Captures audio from the DJI mic, detects speech via energy-based VAD,
and transcribes each utterance using a local Whisper model.

Output: C:\workspace\voxtral\output\transcription.log  (append, UTF-8)
Logs  : C:\workspace\voxtral\logs\transcribe.log
"""

import json
import logging
import os
import queue
import sys
import tempfile
import threading
import time
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Tuple

import numpy as np
import scipy.io.wavfile as wavfile
import sounddevice as sd
from faster_whisper import WhisperModel


# ── Config ────────────────────────────────────────────────────────────────────

CONFIG_PATH = Path(__file__).parent / "config.json"

_DEFAULTS = {
    "device_pattern":          "DJI",
    "sample_rate":             16000,
    "chunk_duration":          0.1,      # seconds per audio callback block
    "silence_threshold":       0.015,    # RMS energy below this = silence
    "silence_duration":        1.5,      # seconds of silence to end an utterance
    "max_utterance_duration":  30.0,
    "output_file":             "C:/workspace/voxtral/output/transcription.log",
    # faster-whisper settings
    "whisper_model":           "small",  # tiny / base / small / medium / large-v3
    "whisper_language":        None,     # None = auto-detect, "en" = English only
    "whisper_device":          "cpu",
    "whisper_compute_type":    "int8",   # int8 is fastest on CPU
}


def load_config() -> dict:
    cfg = dict(_DEFAULTS)
    if CONFIG_PATH.exists():
        with open(CONFIG_PATH, encoding="utf-8") as f:
            cfg.update(json.load(f))
    return cfg


# ── Logging ───────────────────────────────────────────────────────────────────

def setup_logging(log_dir: Path) -> logging.Logger:
    log_dir.mkdir(parents=True, exist_ok=True)
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)-8s %(message)s",
        handlers=[
            logging.StreamHandler(sys.stdout),
            logging.FileHandler(log_dir / "transcribe.log", encoding="utf-8"),
        ],
    )
    return logging.getLogger(__name__)


# ── Device detection ──────────────────────────────────────────────────────────

def find_input_device(pattern: str) -> Tuple[Optional[int], Optional[str]]:
    for idx, dev in enumerate(sd.query_devices()):
        if dev["max_input_channels"] > 0 and pattern.lower() in dev["name"].lower():
            return idx, dev["name"]
    return None, None


# ── Capture + VAD loop ────────────────────────────────────────────────────────

def run_capture(cfg: dict, model: WhisperModel, log: logging.Logger) -> None:
    device_idx, device_name = find_input_device(cfg["device_pattern"])
    if device_idx is None:
        log.warning("No device matching '%s' found - using system default input",
                    cfg["device_pattern"])
    else:
        log.info("Mic : [%d] %s", device_idx, device_name)

    sr            = int(cfg["sample_rate"])
    chunk_samples = int(sr * cfg["chunk_duration"])
    silence_thr   = float(cfg["silence_threshold"])
    max_silent    = int(cfg["silence_duration"] / cfg["chunk_duration"])
    max_chunks    = int(cfg["max_utterance_duration"] / cfg["chunk_duration"])
    language      = cfg.get("whisper_language") or None
    output_path   = Path(cfg["output_file"])

    output_path.parent.mkdir(parents=True, exist_ok=True)
    log.info("Output : %s", output_path)
    log.info("Silence: threshold=%.3f  duration=%.1fs", silence_thr, cfg["silence_duration"])
    log.info("Ready - listening for speech ...")

    audio_q: queue.Queue = queue.Queue()

    def audio_callback(indata, frames, time_info, status):
        if status:
            log.debug("Stream status: %s", status)
        audio_q.put(indata[:, 0].copy())

    def transcription_worker(audio_data: np.ndarray) -> None:
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
            wavfile.write(tmp.name, sr, audio_data)
            tmp_path = tmp.name
        try:
            t0 = time.monotonic()
            segments, info = model.transcribe(
                tmp_path,
                language=language,
                beam_size=5,
                vad_filter=True,          # whisper's own VAD as second pass
                vad_parameters={"min_silence_duration_ms": 300},
            )
            text = " ".join(seg.text.strip() for seg in segments).strip()
            elapsed = time.monotonic() - t0
            if text:
                ts   = datetime.now().strftime("%H:%M:%S")
                line = f"[{ts}] {text}"
                log.info(">> %s  (%.1fs)", text, elapsed)
                with open(output_path, "a", encoding="utf-8") as f:
                    f.write(line + "\n")
            else:
                log.debug("Empty transcription (%.1fs)", elapsed)
        except Exception as exc:
            log.error("Transcription error: %s", exc)
        finally:
            os.unlink(tmp_path)

    utterance:    List[np.ndarray] = []
    silent_count: int              = 0
    in_speech:    bool             = False

    with sd.InputStream(
        device=device_idx,
        samplerate=sr,
        channels=1,
        dtype="float32",
        blocksize=chunk_samples,
        callback=audio_callback,
    ):
        log.info("Audio stream open")
        while True:
            chunk  = audio_q.get()
            energy = float(np.sqrt(np.mean(chunk ** 2)))

            if energy > silence_thr:
                if not in_speech:
                    in_speech = True
                    log.debug("Speech start (rms=%.4f)", energy)
                utterance.append(chunk)
                silent_count = 0

            elif in_speech:
                utterance.append(chunk)
                silent_count += 1

                if silent_count >= max_silent or len(utterance) >= max_chunks:
                    audio_data = np.clip(
                        np.concatenate(utterance) * 32767, -32768, 32767
                    ).astype(np.int16)
                    threading.Thread(
                        target=transcription_worker,
                        args=(audio_data,),
                        daemon=True,
                    ).start()
                    utterance    = []
                    silent_count = 0
                    in_speech    = False


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    cfg     = load_config()
    log_dir = Path(cfg["output_file"]).parent.parent / "logs"
    log     = setup_logging(log_dir)

    log.info("Loading Whisper model '%s' (%s / %s) ...",
             cfg["whisper_model"], cfg["whisper_device"], cfg["whisper_compute_type"])
    t0 = time.monotonic()
    model = WhisperModel(
        cfg["whisper_model"],
        device=cfg["whisper_device"],
        compute_type=cfg["whisper_compute_type"],
    )
    log.info("Model loaded in %.1fs", time.monotonic() - t0)

    try:
        run_capture(cfg, model, log)
    except KeyboardInterrupt:
        log.info("Stopped by user")
    except Exception:
        log.exception("Fatal error in capture loop")
        sys.exit(1)


if __name__ == "__main__":
    main()
