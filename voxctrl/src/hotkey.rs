use std::sync::Arc;

use anyhow::{Context, Result};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

use crate::backend::TranscriptionBackend;
use crate::config::Config;
use crate::{AppStatus, SharedState};

/// Register the Ctrl+Super+Space toggle hotkey.
/// Returns (manager, hotkey_id). The manager must be kept alive.
pub fn setup_hotkeys() -> Result<(GlobalHotKeyManager, u32)> {
    let manager = GlobalHotKeyManager::new().context("create hotkey manager")?;
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SUPER), Code::Space);
    let id = hotkey.id();
    manager.register(hotkey).context("register Ctrl+Super+Space")?;
    Ok((manager, id))
}

/// Process a global hotkey event from the winit event loop.
/// Toggle: Idle → Recording → Transcribing → Idle.
pub fn handle_hotkey_event(
    event: &GlobalHotKeyEvent,
    hotkey_id: Option<u32>,
    state: &Arc<SharedState>,
    cfg: &Config,
    backend: Arc<dyn TranscriptionBackend + Send + Sync>,
) {
    if Some(event.id) != hotkey_id {
        return;
    }

    let current = *state.status.lock().unwrap();
    match current {
        AppStatus::Idle => {
            // Start recording
            state.chunks.lock().unwrap().clear();
            *state.status.lock().unwrap() = AppStatus::Recording;
            log::info!("Recording started");
        }
        AppStatus::Recording => {
            // Stop recording → transcribe
            *state.status.lock().unwrap() = AppStatus::Transcribing;
            log::info!("Recording stopped, transcribing…");

            let chunks: Vec<f32> = state.chunks.lock().unwrap().drain(..).collect();

            if chunks.is_empty() {
                log::info!("No audio captured, returning to idle");
                *state.status.lock().unwrap() = AppStatus::Idle;
                return;
            }

            let state_clone = state.clone();
            let sample_rate = cfg.sample_rate;
            std::thread::Builder::new()
                .name("transcription".into())
                .spawn(move || {
                    if let Err(e) = transcribe_and_type(&chunks, sample_rate, &*backend) {
                        log::error!("Transcription error: {}", e);
                    }
                    *state_clone.status.lock().unwrap() = AppStatus::Idle;
                    log::info!("Back to idle");
                })
                .expect("spawn transcription thread");
        }
        AppStatus::Transcribing => {
            log::debug!("Ignoring hotkey — already transcribing");
        }
    }
}

/// Drain chunks → write WAV → transcribe → type result.
fn transcribe_and_type(
    chunks: &[f32],
    sample_rate: u32,
    backend: &dyn TranscriptionBackend,
) -> Result<()> {
    // Write WAV to temp file
    let tmp = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile()
        .context("create temp WAV")?;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(tmp.path(), spec).context("create WAV writer")?;
    for &sample in chunks {
        let s16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(s16)?;
    }
    writer.finalize().context("finalize WAV")?;

    // Transcribe
    let start = std::time::Instant::now();
    let text = backend.transcribe(tmp.path())?;
    let elapsed = start.elapsed().as_secs_f64();

    let preview = if text.len() > 80 {
        &text[..80]
    } else {
        &text
    };
    log::info!("Transcribed in {:.1}s: {}", elapsed, preview);

    if !text.is_empty() {
        crate::typing::type_text(&text)?;
    }

    Ok(())
}
