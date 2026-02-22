use anyhow::Result;
use enigo::{Enigo, Keyboard, Settings};

pub fn type_text(text: &str) -> Result<()> {
    // Brief pause so the target window has time to settle focus after hotkey release.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("Failed to create Enigo instance: {e}"))?;

    enigo
        .text(text)
        .map_err(|e| anyhow::anyhow!("Failed to inject text: {e}"))?;

    log::info!("Typed {} chars", text.len());
    Ok(())
}
