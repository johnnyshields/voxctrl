pub mod voxtral;

use std::path::Path;

use anyhow::Result;

pub trait TranscriptionBackend {
    fn transcribe(&self, wav_path: &Path) -> Result<String>;
    fn is_available(&self) -> bool;
    fn name(&self) -> &str;
}
