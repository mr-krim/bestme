use anyhow::Result;
use log::{error, info};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{plugin::Plugin, Invoke, Runtime};

use bestme::audio::device::DeviceManager;
use bestme::audio::capture::{CaptureManager, AudioData};

use crate::plugin::TranscribeState;

// Structure to hold our audio state
pub struct AudioState {
    device_manager: Arc<Mutex<DeviceManager>>,
    capture_manager: Arc<Mutex<Option<CaptureManager>>>,
    transcribe_state: Option<Arc<TranscribeState>>,
    is_recording: bool,
    peak_level: Arc<Mutex<f32>>,
}

impl AudioState {
    pub fn new(device_manager: Arc<Mutex<DeviceManager>>) -> Self {
        Self {
            device_manager,
            capture_manager: Arc::new(Mutex::new(None)),
            transcribe_state: None,
            is_recording: false,
            peak_level: Arc::new(Mutex::new(0.0)),
        }
    }
    
    pub fn set_transcribe_state(&mut self, transcribe_state: Arc<TranscribeState>) {
        self.transcribe_state = Some(transcribe_state);
    }

    pub fn start_recording(&mut self, device_name: &str) -> Result<()> {
        info!("Starting audio recording with device: {}", device_name);

        // Get the device by name
        let device_opt = {
            let device_manager = self.device_manager.lock();
            let devices = device_manager.list_devices()?;
            devices.into_iter().find(|d| d.name() == device_name)
        };

        // Create and start the capture manager
        if let Some(device) = device_opt {
            let mut capture_manager = CaptureManager::new()?;
            capture_manager.set_device(device);
            
            // Set up a callback to receive peak level updates
            let peak_level_mutex = Arc::clone(&self.peak_level);
            
            capture_manager.on_peak_level(move |level| {
                let mut peak_level = peak_level_mutex.lock();
                *peak_level = level;
            });
            
            // If we have a transcribe state, create an audio channel
            let audio_sender = if let Some(transcribe_state) = &self.transcribe_state {
                Some(transcribe_state.create_audio_channel())
            } else {
                None
            };
            
            // Set up audio data callback for transcription
            if let Some(sender) = audio_sender {
                let sender_clone = sender.clone();
                capture_manager.on_audio_data(move |audio_data| {
                    // Send audio data to transcription component
                    let sender = sender_clone.clone();
                    tokio::spawn(async move {
                        if let Err(e) = sender.send(audio_data).await {
                            error!("Failed to send audio data: {}", e);
                        }
                    });
                });
            }
            
            // Start the audio capture
            capture_manager.start()?;
            
            // Update the internal state
            let mut state = self.capture_manager.lock();
            *state = Some(capture_manager);
            self.is_recording = true;
            
            Ok(())
        } else {
            error!("Device not found: {}", device_name);
            Err(anyhow::anyhow!("Device not found: {}", device_name))
        }
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        info!("Stopping audio recording");
        
        let mut capture_manager_lock = self.capture_manager.lock();
        if let Some(capture_manager) = capture_manager_lock.take() {
            capture_manager.stop()?;
            self.is_recording = false;
            
            // Reset peak level
            let mut peak_level = self.peak_level.lock();
            *peak_level = 0.0;
            
            Ok(())
        } else {
            error!("No active recording to stop");
            Err(anyhow::anyhow!("No active recording to stop"))
        }
    }

    pub fn get_peak_level(&self) -> f32 {
        *self.peak_level.lock()
    }
    
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }
}

// Define the Audio plugin
pub struct AudioPlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> AudioPlugin<R> {
    pub fn new() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![
                start_recording,
                stop_recording,
                get_peak_level,
                is_recording,
            ]),
        }
    }
}

impl<R: Runtime> Plugin<R> for AudioPlugin<R> {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
}

// Tauri command handlers
#[tauri::command]
async fn start_recording(
    device_name: String, 
    state: tauri::State<'_, AudioState>
) -> Result<(), String> {
    let mut state_mut = state.inner().lock();
    state_mut.start_recording(&device_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_recording(
    state: tauri::State<'_, AudioState>
) -> Result<(), String> {
    let mut state_mut = state.inner().lock();
    state_mut.stop_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_peak_level(state: tauri::State<'_, AudioState>) -> f32 {
    state.inner().get_peak_level()
}

#[tauri::command]
fn is_recording(state: tauri::State<'_, AudioState>) -> bool {
    state.inner().is_recording()
} 
