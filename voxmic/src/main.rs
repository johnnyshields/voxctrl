//! voxmic — Cross-platform tray dictation app
//!
//! Hold Ctrl+Win to record, release to transcribe via Voxtral, type at cursor.
//! Tray icon: green = idle, red = recording, amber = transcribing.

mod audio;
mod backend;
mod config;
mod hotkey;
mod tray;
mod typing;

use std::sync::{Arc, Mutex};

use anyhow::Result;
use global_hotkey::GlobalHotKeyEvent;
use tray_icon::TrayIconEvent;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};

// ── Shared types (used by all modules via `crate::`) ────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Idle,
    Recording,
    Transcribing,
}

pub struct SharedState {
    pub status: Mutex<AppStatus>,
    pub chunks: Mutex<Vec<f32>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(AppStatus::Idle),
            chunks: Mutex::new(Vec::new()),
        }
    }
}

// ── Application handler for winit event loop ────────────────────────────────

struct App {
    state: Arc<SharedState>,
    #[allow(dead_code)]
    tray: Option<tray_icon::TrayIcon>,
    #[allow(dead_code)]
    hotkey_manager: Option<global_hotkey::GlobalHotKeyManager>,
    hotkey_id: Option<u32>,
    cfg: config::Config,
    backend: Arc<dyn backend::TranscriptionBackend + Send + Sync>,
    _audio_stream: Option<cpal::Stream>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // No windows to create — we are tray-only
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
        // No windows
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process tray icon events
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            log::trace!("Tray event: {:?}", event);
        }

        // Process global hotkey events
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            hotkey::handle_hotkey_event(
                &event,
                self.hotkey_id,
                &self.state,
                &self.cfg,
                self.backend.clone(),
            );
        }
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("─── voxmic starting ───");

    // Load config
    let cfg = config::load_config();
    log::info!("Config loaded: backend={}", cfg.backend);

    // Create shared state
    let state = Arc::new(SharedState::new());

    // Create backend
    let voxtral = backend::voxtral::VoxtralBackend::new();
    let backend: Arc<dyn backend::TranscriptionBackend + Send + Sync> = Arc::new(voxtral);

    // Build winit event loop (must be on main thread for macOS)
    let event_loop = EventLoop::new()?;

    // Set up tray icon (must be created after EventLoop on some platforms)
    let tray = tray::build_tray()?;
    log::info!("Tray icon created");

    // Set up global hotkey
    let (hotkey_manager, hotkey_id) = hotkey::setup_hotkeys()?;
    log::info!("Global hotkey registered (Ctrl+Win)");

    // Start audio capture (always-open stream)
    let audio_stream = audio::AudioCapture::start(state.clone(), &cfg)?;
    log::info!("Audio stream open (always-on)");

    // Run the event loop
    let mut app = App {
        state,
        tray: Some(tray),
        hotkey_manager: Some(hotkey_manager),
        hotkey_id: Some(hotkey_id),
        cfg,
        backend,
        _audio_stream: Some(audio_stream),
    };

    log::info!("Tray icon running — green=idle  red=recording  amber=transcribing");
    event_loop.run_app(&mut app)?;

    log::info!("─── voxmic stopped ───");
    Ok(())
}
