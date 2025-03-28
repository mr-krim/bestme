use anyhow::Result;
use log::{error, info, debug};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{Manager, AppHandle, State, plugin};
use tokio::sync::mpsc;
use std::marker::PhantomData;

use bestme::audio::device::DeviceManager;
use bestme::audio::capture::{CaptureManager, ThreadedCaptureManager, AudioData, AudioEvent};

use crate::plugin::TranscribeState;

// Structure to hold our audio state
pub struct AudioState {
    device_manager: Arc<Mutex<DeviceManager>>,
    capture_manager: Arc<Mutex<Option<ThreadedCaptureManager>>>,
    event_receiver: Arc<Mutex<Option<mpsc::Receiver<AudioEvent>>>>,
    transcribe_state: Option<Arc<TranscribeState>>,
    is_recording: Arc<Mutex<bool>>,
    peak_level: Arc<Mutex<f32>>,
    selected_device: Arc<Mutex<Option<String>>>,
}

impl AudioState {
    pub fn new(device_manager: Arc<Mutex<DeviceManager>>) -> Self {
        Self {
            device_manager,
            capture_manager: Arc::new(Mutex::new(None)),
            event_receiver: Arc::new(Mutex::new(None)),
            transcribe_state: None,
            is_recording: Arc::new(Mutex::new(false)),
            peak_level: Arc::new(Mutex::new(0.0)),
            selected_device: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn set_transcribe_state(&mut self, transcribe_state: Arc<TranscribeState>) {
        self.transcribe_state = Some(transcribe_state);
    }

    pub fn start_recording(&self, device_name: &str) -> Result<()> {
        info!("Starting audio recording with device: {}", device_name);

        // Get the device
        let device = {
            let device_manager = self.device_manager.lock();
            let devices = device_manager.list_devices()
                .map_err(|e| anyhow::anyhow!("Failed to list devices: {}", e))?;
            
            let device = devices.into_iter()
                .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", device_name))?;
            
            device
        };
        
        // Get or create the capture manager
        let manager = {
            let mut cm = self.capture_manager.lock();
            
            if cm.is_none() {
                let (manager, receiver) = ThreadedCaptureManager::create_from_capture_manager()?;
                
                // Store the event receiver
                {
                    let mut er = self.event_receiver.lock();
                    *er = Some(receiver);
                }
                
                // Process events
                self.process_audio_events();
                
                *cm = Some(manager);
            }
            
            // Get a reference to create new manager
            let cm_ref = cm.as_ref().unwrap();
            ThreadedCaptureManager {
                command_sender: cm_ref.get_command_sender()
            }
        };
        
        // Set the device
        manager.set_device(device)?;
        
        // Set up peak level callback
        let peak_level = Arc::clone(&self.peak_level);
        manager.on_peak_level(move |level| {
            let mut peak = peak_level.lock();
            *peak = level;
        })?;
        
        // Set up audio data callback if we have a transcribe state
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
        
        // Start recording
        manager.start()?;
        
        // Update recording state
        {
            let mut recording = self.is_recording.lock();
            *recording = true;
        }
        
        // Store selected device
        {
            let mut selected_device = self.selected_device.lock();
            *selected_device = Some(device_name.to_string());
        }
        
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<()> {
        info!("Stopping audio recording");
        
        // Get the capture manager
        let manager = {
            let cm = self.capture_manager.lock();
            
            match cm.as_ref() {
                Some(manager) => ThreadedCaptureManager { 
                    command_sender: manager.get_command_sender()
                },
                None => return Err(anyhow::anyhow!("No active recording to stop")),
            }
        };
        
        // Stop recording
        manager.stop()?;
        
        // Update recording state
        {
            let mut recording = self.is_recording.lock();
            *recording = false;
        }
        
        // Reset peak level
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
    
    // Process audio events from the event receiver
    fn process_audio_events(&self) {
        let event_receiver = {
            let mut er = self.event_receiver.lock();
            er.take()
        };
        
        if let Some(mut receiver) = event_receiver {
            let peak_level = Arc::clone(&self.peak_level);
            let is_recording = Arc::clone(&self.is_recording);
            
            // Start a task to process events
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    match event {
                        AudioEvent::Level(level) => {
                            // Update peak level
                            let mut peak = peak_level.lock();
                            *peak = level;
                        },
                        AudioEvent::LevelChanged(level) => {
                            // Legacy compatibility for level changes
                            let mut peak = peak_level.lock();
                            *peak = level;
                        },
                        AudioEvent::Data(_) => {
                            // Event already processed by the callback
                        },
                        AudioEvent::Error(err) => {
                            error!("Audio error: {}", err);
                        },
                        AudioEvent::Stopped => {
                            let mut recording = is_recording.lock();
                            *recording = false;
                        },
                        AudioEvent::Started => {
                            // Just log the event
                            debug!("Audio recording started");
                        }
                    }
                }
            });
        }
    }

    // Initialize the AudioState
    pub fn initialize(&self) -> Result<()> {
        // Try to use default device
        let default_device = {
            let device_manager = self.device_manager.lock();
            device_manager.get_default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device found"))?
        };
        
        // Create a threaded capture manager
        let (capture_manager, receiver) = ThreadedCaptureManager::create_from_capture_manager()?;
        
        // Set the device
        capture_manager.set_device(default_device.clone())?;
        
        // Set up a callback for peak level updates
        let peak_level = Arc::clone(&self.peak_level);
        capture_manager.on_peak_level(move |level| {
            let mut peak = peak_level.lock();
            *peak = level;
        })?;
        
        // Set up audio data callback if we have a transcribe state
        if let Some(ts) = &self.transcribe_state {
            let sender = ts.create_audio_channel();
            let sender_clone = sender.clone();
            
            capture_manager.on_audio_data(move |audio_data| {
                let sender = sender_clone.clone();
                tokio::spawn(async move {
                    if let Err(e) = sender.send(audio_data).await {
                        error!("Failed to send audio data: {}", e);
                    }
                });
            })?;
        }
        
        // Store the capture manager
        {
            let mut cm = self.capture_manager.lock();
            *cm = Some(capture_manager);
        }
        
        // Store the event receiver and start processing events
        {
            let mut er = self.event_receiver.lock();
            *er = Some(receiver);
        }
        
        // Start event processing
        self.process_audio_events();
        
        Ok(())
    }
    
    // Set the audio device
    pub fn set_device(&self, device_id: &str) -> Result<()> {
        // Get the device
        let device = {
            let device_manager = self.device_manager.lock();
            device_manager.get_device_by_id(device_id)
                .ok_or_else(|| anyhow::anyhow!("Device not found with ID: {}", device_id))?
        };
        
        // Get the capture manager
        let manager = {
            let cm = self.capture_manager.lock();
            
            match cm.as_ref() {
                Some(manager) => ThreadedCaptureManager { 
                    command_sender: manager.get_command_sender()
                },
                None => {
                    return self.initialize();
                },
            }
        };
        
        // Set the device
        manager.set_device(device)?;
        
        // Store selected device
        {
            let mut selected_device = self.selected_device.lock();
            *selected_device = Some(device_id.to_string());
        }
        
        Ok(())
    }
}

// Audio plugin for Tauri 2.0
#[derive(Default)]
pub struct AudioPlugin {
    _phantom: PhantomData<()>,
}

impl AudioPlugin {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl tauri::Plugin for AudioPlugin {
    fn name(&self) -> &'static str {
        "audio"
    }
    
    fn initialize(&mut self, app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing audio plugin");
        
        // Register the plugin with the app
        app.plugin(
            tauri::plugin::Builder::new("audio")
                .js_init_script(include_str!("./audio_init.js"))
                .setup(|app, _| {
                    // Initialize the audio state if needed
                    let audio_state = app.state::<Arc<Mutex<AudioState>>>();
                    if let Err(e) = audio_state.lock().initialize() {
                        error!("Failed to initialize audio state: {}", e);
                    }
                    
                    Ok(())
                })
                .build(),
        )?;
        
        Ok(())
    }
}

// Helper to create a ThreadedCaptureManager
impl ThreadedCaptureManager {
    pub fn create_from_capture_manager() -> Result<(Self, mpsc::Receiver<AudioEvent>)> {
        let (command_sender, command_receiver) = mpsc::channel(10);
        let (event_sender, event_receiver) = mpsc::channel(10);
        
        // Create and spawn the manager
        let manager = CaptureManager::new(command_receiver, event_sender)?;
        tokio::spawn(async move {
            manager.run().await;
        });
        
        Ok((Self { command_sender }, event_receiver))
    }
    
    pub fn get_command_sender(&self) -> mpsc::Sender<CaptureCommand> {
        self.command_sender.clone()
    }
}

// Tauri 2.0 command handlers
#[tauri::command]
pub async fn start_recording(
    device_name: String, 
    state: tauri::State<'_, Arc<Mutex<AudioState>>>
) -> Result<(), String> {
    state.inner().lock().start_recording(&device_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> Result<(), String> {
    state.inner().lock().stop_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_level(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> f32 {
    state.inner().lock().get_peak_level()
}

#[tauri::command]
pub async fn is_recording(state: tauri::State<'_, Arc<Mutex<AudioState>>>) -> bool {
    state.inner().lock().is_recording()
}

#[tauri::command]
pub async fn get_audio_devices(
    state: tauri::State<'_, Arc<Mutex<AudioState>>>
) -> Result<Vec<(String, String)>, String> {
    let state = state.inner().lock();
    let device_manager = state.device_manager.lock();
    
    let devices = device_manager.list_devices()
        .map_err(|e| e.to_string())?;
    
    Ok(devices.into_iter()
        .filter_map(|d| {
            let id = d.id().to_string();
            let name = match d.name() {
                Ok(name) => name,
                Err(_) => return None,
            };
            Some((id, name))
        })
        .collect())
}

#[tauri::command]
pub async fn set_device(
    device_id: String,
    state: tauri::State<'_, Arc<Mutex<AudioState>>>
) -> Result<(), String> {
    state.inner().lock().set_device(&device_id)
        .map_err(|e| e.to_string())
} 
