use anyhow::Result;
use log::{error, info, debug, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Manager, AppHandle, State, plugin::Plugin, Runtime, Emitter};
use bestme::audio::capture::CaptureCommand;
use tokio::sync::mpsc;
use std::marker::PhantomData;

use bestme::audio::device::DeviceManager;
use bestme::audio::capture::{CaptureManager, ThreadedCaptureManager, AudioData, AudioEvent};
use bestme::config::ConfigManager;

use crate::plugin::TranscribeState;

// Structure to hold our audio state
pub struct AudioState {
    config_manager: Arc<Mutex<ConfigManager>>,
    device_manager: Arc<Mutex<DeviceManager>>,
    capture_manager: Arc<Mutex<Option<ThreadedCaptureManager>>>,
    event_receiver: Arc<Mutex<Option<mpsc::Receiver<AudioEvent>>>>,
    transcribe_state: Option<Arc<TranscribeState>>,
    is_recording: Arc<Mutex<bool>>,
    peak_level: Arc<Mutex<f32>>,
    selected_device: Arc<Mutex<Option<String>>>,
    app_handle: Option<AppHandle<tauri::Wry>>,
}

impl AudioState {
    pub fn new(config_manager: Arc<Mutex<ConfigManager>>, device_manager: Arc<Mutex<DeviceManager>>) -> Self {
        Self {
            config_manager,
            device_manager,
            capture_manager: Arc::new(Mutex::new(None)),
            event_receiver: Arc::new(Mutex::new(None)),
            transcribe_state: None,
            is_recording: Arc::new(Mutex::new(false)),
            peak_level: Arc::new(Mutex::new(0.0)),
            selected_device: Arc::new(Mutex::new(None)),
            app_handle: None,
        }
    }
    
    pub fn set_transcribe_state(&mut self, transcribe_state: Arc<TranscribeState>) {
        self.transcribe_state = Some(transcribe_state);
    }

    pub fn start_recording(&self) -> Result<()> {
        info!("Attempting to start audio recording using configured device...");

        // Create or get the capture manager
        {
            let mut cm_lock = self.capture_manager.lock();
            
            if cm_lock.is_none() {
                info!("Creating new ThreadedCaptureManager instance.");
                let (manager, receiver) = ThreadedCaptureManager::create_threaded(self.config_manager.clone())?;
                
                {
                    let mut er = self.event_receiver.lock();
                    *er = Some(receiver);
                }
                
                self.process_audio_events(); 
                
                // Set up callbacks before storing
                let peak_level = Arc::clone(&self.peak_level);
                manager.on_peak_level(move |level| {
                    let mut peak = peak_level.lock();
                    *peak = level;
                })?;
                
                if let Some(transcribe_state) = &self.transcribe_state {
                    let audio_sender = transcribe_state.create_audio_channel();
                    let audio_sender_clone = audio_sender.clone();
                    
                    manager.on_audio_data(move |audio_data| {
                        let sender = audio_sender_clone.clone(); 
                        tokio::spawn(async move {
                            if let Err(e) = sender.send(audio_data).await {
                                error!("Failed to send audio data: {}", e);
                            }
                        });
                    })?;
                }
                
                // Start the manager
                manager.start()?;
                
                *cm_lock = Some(manager);
            } else {
                info!("Using existing ThreadedCaptureManager instance.");
                // Just start the existing manager
                if let Some(manager) = cm_lock.as_ref() {
                    manager.start()?;
                }
            }
        }
        
        {
            let mut recording = self.is_recording.lock();
            *recording = true;
        }
        
        {
            let config = self.config_manager.lock().get_config().clone();
            let device_id_used = config.audio.input_device;
            let mut selected_device_lock = self.selected_device.lock();
            *selected_device_lock = device_id_used; 
            if selected_device_lock.is_some() {
                 info!("Recording started with device ID: {}", selected_device_lock.as_ref().unwrap());
            } else {
                 info!("Recording started with default device.");
            }
        }
        
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<()> {
        info!("Stopping audio recording");
        
        {
            let cm = self.capture_manager.lock();
            
            match cm.as_ref() {
                Some(manager) => manager.stop()?,
                None => return Err(anyhow::anyhow!("No active recording to stop")),
            }
        }
        
        {
            let mut recording = self.is_recording.lock();
            *recording = false;
        }
        
        {
            let mut peak = self.peak_level.lock();
            *peak = 0.0;
        }
        
        Ok(())
    }

    pub fn get_peak_level(&self) -> f32 {
        *self.peak_level.lock()
    }
    
    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock()
    }
    
    fn process_audio_events(&self) {
        let event_receiver_option = {
            let mut er_lock = self.event_receiver.lock();
            er_lock.take()
        };
        
        if let Some(mut receiver) = event_receiver_option {
            let peak_level = Arc::clone(&self.peak_level);
            let is_recording = Arc::clone(&self.is_recording);
            let app_handle = self.app_handle.clone();
            
            tokio::spawn(async move {
                info!("Audio event processing task started.");
                while let Some(event) = receiver.recv().await {
                    // Handle the event
                    match &event {
                        AudioEvent::Data(_) => continue, // Skip data events for emission
                        _ => {}
                    }
                    
                    if let Some(handle) = &app_handle {
                        let (event_name, payload) = match &event {
                             AudioEvent::Level(l) | AudioEvent::LevelChanged(l) => 
                                ("audio:level", serde_json::json!({ "level": l })),
                             AudioEvent::Error(e) => 
                                ("audio:error", serde_json::json!({ "error": e })),
                             AudioEvent::Stopped => 
                                ("audio:stopped", serde_json::json!({})),
                             AudioEvent::Started => 
                                ("audio:started", serde_json::json!({})),
                             AudioEvent::Data(_) => unreachable!(),
                        };
                        if let Err(e) = handle.emit(event_name, payload) {
                             error!("Failed to emit audio event '{}': {}", event_name, e);
                        }
                    } else {
                        warn!("AppHandle not available in AudioState for emitting events.");
                    }

                    match event {
                        AudioEvent::Level(level) | AudioEvent::LevelChanged(level) => {
                            let mut peak = peak_level.lock();
                            *peak = level;
                        },
                        AudioEvent::Error(err) => {
                            error!("Audio capture error: {}", err);
                            let mut recording = is_recording.lock();
                            *recording = false;
                        },
                        AudioEvent::Stopped => {
                            info!("Audio capture stopped event received.");
                            let mut recording = is_recording.lock();
                            *recording = false;
                        },
                        AudioEvent::Started => {
                            debug!("Audio capture started event received.");
                        },
                        AudioEvent::Data(_) => {
                        }
                    }
                }
                info!("Audio event processing task finished.");
            });
        } else {
             warn!("Attempted to process audio events, but receiver was already taken or never existed.");
        }
    }

    pub fn initialize(&self) -> Result<()> {
        info!("Initializing AudioState and capture manager...");
        let (capture_manager, receiver) = ThreadedCaptureManager::create_threaded(self.config_manager.clone())?;
        
        {
             let mut cm_lock = self.capture_manager.lock();
             *cm_lock = Some(capture_manager);
             let mut er_lock = self.event_receiver.lock();
             *er_lock = Some(receiver);
        }

        self.process_audio_events();

        info!("AudioState initialized.");
        Ok(())
    }
    
    // Command sender is no longer directly accessible from ThreadedCaptureManager
    // Use the manager's public methods instead

    pub fn set_app_handle(&mut self, handle: AppHandle<tauri::Wry>) {
        self.app_handle = Some(handle);
    }
}

// Audio plugin for Tauri 2.0
#[derive(Default)]
pub struct AudioPlugin<R: Runtime> {
    _phantom: PhantomData<fn() -> R>,
}

impl<R: Runtime> AudioPlugin<R> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<R: Runtime> Plugin<R> for AudioPlugin<R> {
    fn name(&self) -> &'static str {
        "audio"
    }
    
    fn initialize(&mut self, app: &AppHandle<R>, _: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing AudioPlugin...");
        let audio_state = app.state::<Arc<Mutex<AudioState>>>();
        // Skip setting app handle for now - type mismatch between generic R and concrete Wry
        // TODO: Refactor AudioState to be generic over Runtime
        // audio_state.lock().set_app_handle(app.clone());

        info!("AudioPlugin initialized.");
        Ok(())
    }
}

// Helper function to create a ThreadedCaptureManager
// TODO: This function is not currently used and needs to be updated to the new API
/*
fn create_threaded_capture_manager() -> Result<(ThreadedCaptureManager, mpsc::Receiver<AudioEvent>)> {
    let (command_sender, command_receiver) = mpsc::channel(10);
    let (event_sender, event_receiver) = mpsc::channel(10);
    
    // Create and spawn the manager
    let manager = CaptureManager::new(command_receiver, event_sender)?;
    tokio::spawn(async move {
        manager.run().await;
    });
    
    Ok((ThreadedCaptureManager { command_sender }, event_receiver))
}
*/

// Tauri 2.0 command handlers
#[tauri::command]
pub async fn start_recording(
    state: tauri::State<'_, Arc<Mutex<AudioState>>>
) -> Result<(), String> {
    state.inner().lock().start_recording().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> Result<(), String> {
    state.inner().lock().stop_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_level(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> Result<f32, String> {
    Ok(state.inner().lock().get_peak_level())
}

#[tauri::command]
pub async fn is_recording(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> Result<bool, String> {
    Ok(state.inner().lock().is_recording())
} 
