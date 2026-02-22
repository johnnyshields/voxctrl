use std::sync::Arc;
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, StreamConfig};
use crate::{AppStatus, SharedState};
use crate::config::Config;

pub struct AudioCapture;

impl AudioCapture {
    pub fn start(state: Arc<SharedState>, cfg: &Config) -> Result<cpal::Stream> {
        let host = cpal::default_host();

        // Find input device matching cfg.device_pattern (case-insensitive substring).
        // Fall back to the system default input device if no match is found.
        let pattern = cfg.device_pattern.to_lowercase();
        let device = host
            .input_devices()
            .context("Failed to enumerate input devices")?
            .find(|d| {
                d.name()
                    .map(|n| n.to_lowercase().contains(&pattern))
                    .unwrap_or(false)
            })
            .or_else(|| host.default_input_device())
            .context("No input audio device found")?;

        let device_name = device.name().unwrap_or_else(|_| "<unknown>".to_string());
        log::info!("Audio device: {device_name}");

        // Build a StreamConfig: prefer cfg.sample_rate with 1 channel.
        // If the device does not support the requested rate, fall back to its default config.
        let desired_rate = SampleRate(cfg.sample_rate);
        let stream_config: StreamConfig = match device
            .supported_input_configs()
            .context("Cannot query device input configs")?
            .find(|c| {
                c.channels() >= 1
                    && c.min_sample_rate() <= desired_rate
                    && desired_rate <= c.max_sample_rate()
            }) {
            Some(range) => {
                let mut sc: StreamConfig = range.with_sample_rate(desired_rate).into();
                sc.channels = 1;
                sc
            }
            None => {
                let default = device
                    .default_input_config()
                    .context("No default input config available")?;
                log::warn!(
                    "Sample rate {} Hz not supported by '{device_name}'; \
                     falling back to device default {} Hz",
                    cfg.sample_rate,
                    default.sample_rate().0,
                );
                default.into()
            }
        };

        log::info!(
            "Stream config: {} Hz, {} ch",
            stream_config.sample_rate.0,
            stream_config.channels,
        );

        // Callback: push f32 samples into shared state only while Recording.
        let state_cb = Arc::clone(&state);
        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let is_recording = {
                        let status = state_cb.status.lock().unwrap();
                        *status == AppStatus::Recording
                    };
                    if is_recording {
                        let mut chunks = state_cb.chunks.lock().unwrap();
                        chunks.extend_from_slice(data);
                    }
                },
                |err| {
                    log::error!("Audio capture error: {err}");
                },
                None,
            )
            .context("Failed to build audio input stream")?;

        stream.play().context("Failed to start audio stream")?;
        Ok(stream)
    }
}
