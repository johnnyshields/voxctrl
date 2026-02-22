use std::path::Path;

use anyhow::{Context, Result};

use super::TranscriptionBackend;

pub struct VoxtralBackend {
    base_url: String,
}

impl VoxtralBackend {
    pub fn new() -> Self {
        Self {
            base_url: "http://127.0.0.1:5200".to_string(),
        }
    }

    pub fn with_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl TranscriptionBackend for VoxtralBackend {
    fn transcribe(&self, wav_path: &Path) -> Result<String> {
        let wav_bytes =
            std::fs::read(wav_path).with_context(|| format!("reading WAV: {:?}", wav_path))?;

        let boundary = "----VoxmicBoundary";
        let mut body: Vec<u8> = Vec::new();

        // Part: file
        let part_header = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"audio.wav\"\r\nContent-Type: audio/wav\r\n\r\n"
        );
        body.extend_from_slice(part_header.as_bytes());
        body.extend_from_slice(&wav_bytes);
        body.extend_from_slice(b"\r\n");

        // Part: model (required by OpenAI-compatible API)
        let model_part = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\nvoxtral-mini\r\n"
        );
        body.extend_from_slice(model_part.as_bytes());

        // Closing boundary
        let closing = format!("--{boundary}--\r\n");
        body.extend_from_slice(closing.as_bytes());

        let content_type = format!("multipart/form-data; boundary={boundary}");
        let url = format!("{}/v1/audio/transcriptions", self.base_url);

        let response = ureq::post(&url)
            .set("Content-Type", &content_type)
            .send_bytes(&body)
            .context("POST to Voxtral /v1/audio/transcriptions")?;

        let json: serde_json::Value = response.into_json().context("parse Voxtral JSON")?;

        let text = json
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(text)
    }

    fn is_available(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        matches!(ureq::get(&url).call(), Ok(resp) if resp.status() == 200)
    }

    fn name(&self) -> &str {
        "Voxtral Mini"
    }
}
