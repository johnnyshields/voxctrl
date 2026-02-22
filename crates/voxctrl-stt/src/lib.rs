//! voxctrl-stt — Heavy ML inference backends for speech-to-text.
//!
//! Provides whisper-native (candle), whisper-cpp (whisper-rs), and
//! voxtral-native (burn) backends. These are split from voxctrl-core
//! to avoid recompiling heavy ML dependencies when GUI code changes.

#[cfg(feature = "stt-whisper-cpp")]
pub mod whisper_cpp;
#[cfg(feature = "stt-whisper-native")]
pub mod whisper_native;
#[cfg(feature = "stt-voxtral-native")]
pub mod voxtral_native;

use std::path::PathBuf;
use voxctrl_core::config::SttConfig;
use voxctrl_core::stt::Transcriber;

/// Factory function for heavy STT backends.
///
/// Returns `Some(Ok(transcriber))` if this crate handles the backend,
/// `Some(Err(..))` if it handles the backend but init failed,
/// or `None` for unknown backends (lets core handle them).
pub fn stt_factory(
    cfg: &SttConfig,
    model_dir: Option<PathBuf>,
) -> Option<anyhow::Result<Box<dyn Transcriber>>> {
    match cfg.backend.as_str() {
        "whisper-cpp" => {
            #[cfg(feature = "stt-whisper-cpp")]
            { Some(whisper_cpp::WhisperCppTranscriber::new(cfg).map(|t| Box::new(t) as _)) }
            #[cfg(not(feature = "stt-whisper-cpp"))]
            { Some(Err(anyhow::anyhow!("stt-whisper-cpp feature not compiled in"))) }
        }
        "whisper-native" => {
            #[cfg(feature = "stt-whisper-native")]
            { Some(whisper_native::WhisperNativeTranscriber::new(cfg).map(|t| Box::new(t) as _)) }
            #[cfg(not(feature = "stt-whisper-native"))]
            { Some(Err(anyhow::anyhow!("stt-whisper-native feature not compiled in"))) }
        }
        "voxtral-native" => {
            #[cfg(feature = "stt-voxtral-native")]
            { Some(voxtral_native::VoxtralNativeTranscriber::new(model_dir).map(|t| Box::new(t) as _)) }
            #[cfg(not(feature = "stt-voxtral-native"))]
            { let _ = model_dir; Some(Err(anyhow::anyhow!("stt-voxtral-native feature not compiled in"))) }
        }
        _ => None, // Unknown — let core handle it
    }
}
