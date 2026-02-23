use std::path::PathBuf;
use voxctrl_core::stt::Transcriber;
use voxctrl_stt::voxtral_native::VoxtralNativeTranscriber;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let wav_path = std::env::args().nth(1).expect("Usage: test_voxtral <wav_path>");
    let wav_path = PathBuf::from(&wav_path);

    // Try standard HF cache, then Windows AppData path (WSL2)
    let model_dir = dirs::cache_dir()
        .map(|d| {
            d.join("huggingface/hub/models--mistralai--Voxtral-Mini-3B-2507/snapshots/main")
        })
        .filter(|d| d.join("consolidated.safetensors").exists())
        .or_else(|| {
            let p = PathBuf::from("/mnt/c/Users/John/AppData/Local/huggingface/hub/models--mistralai--Voxtral-Mini-3B-2507/snapshots/main");
            if p.join("consolidated.safetensors").exists() {
                Some(p)
            } else {
                None
            }
        });

    log::info!("Model dir: {:?}", model_dir);

    log::info!("Creating transcriber...");
    let transcriber = VoxtralNativeTranscriber::new(model_dir)?;

    log::info!("Transcribing {:?}...", wav_path);
    let result = transcriber.transcribe(&wav_path)?;

    println!("\n=== TRANSCRIPTION RESULT ===");
    println!("{result}");
    println!("============================\n");

    Ok(())
}
