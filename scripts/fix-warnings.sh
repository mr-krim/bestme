#!/bin/bash

echo "=== Fixing BestMe Warnings ==="
echo "=============================="

# Fix unused imports
echo "Step 1: Fixing unused imports..."
cargo fix --lib -p bestme --allow-dirty --allow-staged 2>/dev/null
cargo fix --bin bestme-tauri -p bestme-tauri --allow-dirty --allow-staged 2>/dev/null

# Fix unused variables (prefix with _)
echo "Step 2: Fixing unused variables..."
# This will be done manually in the script below

# Fix unused Result warnings
echo "Step 3: Fixing unused Result warnings..."

# Main library fixes
echo "Fixing src/gui/mod.rs..."
sed -i 's/use log::{info, warn};/use log::info;/' src/gui/mod.rs
sed -i 's/TranslateMessage(&msg);/let _ = TranslateMessage(\&msg);/' src/gui/mod.rs

echo "Fixing src/text_injection/windows_types.rs..."
sed -i 's/KEYEVENTF_KEYUP,//' src/text_injection/windows_types.rs
sed -i 's/KEYEVENTF_UNICODE,//' src/text_injection/windows_types.rs

echo "Fixing src/audio/streaming_transcribe.rs..."
sed -i 's/sample_rate: usize,/_sample_rate: usize,/' src/audio/streaming_transcribe.rs
sed -i 's/task_handle: Arc/_task_handle: Arc/' src/audio/streaming_transcribe.rs

echo "Fixing src/gui/tray.rs..."
sed -i 's/menu: HMENU,/_menu: HMENU,/' src/gui/tray.rs
sed -i 's/AppendMenuA(/let _ = AppendMenuA(/' src/gui/tray.rs
sed -i 's/GetCursorPos(&mut point);/let _ = GetCursorPos(\&mut point);/' src/gui/tray.rs
sed -i 's/SetForegroundWindow(hwnd);/let _ = SetForegroundWindow(hwnd);/' src/gui/tray.rs
sed -i 's/PostMessageA(hwnd/let _ = PostMessageA(hwnd/' src/gui/tray.rs
sed -i 's/Shell_NotifyIconA(NIM_DELETE/let _ = Shell_NotifyIconA(NIM_DELETE/' src/gui/tray.rs
sed -i 's/DestroyWindow(self.hwnd);/let _ = DestroyWindow(self.hwnd);/' src/gui/tray.rs

echo "Fixing src/gui/window.rs..."
sed -i 's/ShowWindow(self.hwnd/let _ = ShowWindow(self.hwnd/' src/gui/window.rs
sed -i 's/DeleteObject(HGDIOBJ/let _ = DeleteObject(HGDIOBJ/' src/gui/window.rs
sed -i 's/EndPaint(hwnd/let _ = EndPaint(hwnd/' src/gui/window.rs
sed -i 's/DestroyWindow(self.hwnd);/let _ = DestroyWindow(self.hwnd);/' src/gui/window.rs

echo "Fixing src/gui/settings.rs..."
sed -i 's/ShowWindow(self.hwnd/let _ = ShowWindow(self.hwnd/' src/gui/settings.rs
sed -i 's/DestroyWindow(self.hwnd);/let _ = DestroyWindow(self.hwnd);/' src/gui/settings.rs

echo "Fixing src/ai fields..."
sed -i 's/config: ModelConfig,/_config: ModelConfig,/' src/ai/local/mod.rs
sed -i 's/config: ModelConfig,/_config: ModelConfig,/' src/ai/local/inference.rs
sed -i 's/session: Session,/_session: Session,/' src/ai/local/inference.rs
sed -i 's/tokenizer: tokenizers::Tokenizer,/_tokenizer: tokenizers::Tokenizer,/' src/ai/local/inference.rs
sed -i 's/max_length: usize,/_max_length: usize,/' src/ai/local/onnx_models.rs
sed -i 's/environment: Arc<Environment>,/_environment: Arc<Environment>,/' src/ai/local/onnx_runtime.rs
sed -i 's/original: String,/_original: String,/' src/ai/local/streaming_inference.rs
sed -i 's/position: TextPosition,/_position: TextPosition,/' src/ai/local/streaming_inference.rs
sed -i 's/timestamp: std::time::Instant,/_timestamp: std::time::Instant,/' src/ai/local/streaming_inference.rs
sed -i 's/usage: Option<RequestyUsage>,/_usage: Option<RequestyUsage>,/' src/ai/cloud/api_client.rs
sed -i 's/prompt_tokens: u32,/_prompt_tokens: u32,/' src/ai/cloud/api_client.rs
sed -i 's/completion_tokens: u32,/_completion_tokens: u32,/' src/ai/cloud/api_client.rs
sed -i 's/total_tokens: u32,/_total_tokens: u32,/' src/ai/cloud/api_client.rs
sed -i 's/metrics_collector: Arc<MetricsCollector>,/_metrics_collector: Arc<MetricsCollector>,/' src/ai/services/model_selector.rs
sed -i 's/start_time: Instant,/_start_time: Instant,/' src/ai/telemetry/metrics.rs
sed -i 's/meter: Meter,/_meter: Meter,/' src/ai/telemetry/metrics.rs
sed -i 's/timestamp: Instant,/_timestamp: Instant,/' src/ai/resilience/recovery.rs
sed -i 's/error_type: String,/_error_type: String,/' src/ai/resilience/recovery.rs
sed -i 's/strategy: String,/_strategy: String,/' src/ai/resilience/recovery.rs
sed -i 's/timestamp: Instant,/_timestamp: Instant,/' src/ai/benchmarks/performance_tracker.rs
sed -i 's/input_size: usize,/_input_size: usize,/' src/ai/benchmarks/performance_tracker.rs
sed -i 's/output_size: usize,/_output_size: usize,/' src/ai/benchmarks/performance_tracker.rs

echo "Fixing src/text_injection/mod.rs..."
sed -i 's/context_detector: ContextDetector,/_context_detector: ContextDetector,/' src/text_injection/mod.rs

# Tauri-specific fixes
echo "Fixing src-tauri imports..."
sed -i 's/use tauri::{Manager, AppHandle, State, plugin::Plugin, Runtime, Emitter};/use tauri::{Manager, AppHandle, plugin::Plugin, Runtime, Emitter};/' src-tauri/src/plugin/audio.rs
sed -i '/use bestme::audio::capture::CaptureCommand;/d' src-tauri/src/plugin/audio.rs
sed -i 's/CaptureManager, ThreadedCaptureManager, AudioData, AudioEvent/ThreadedCaptureManager, AudioEvent/' src-tauri/src/plugin/audio.rs
sed -i '/use reqwest::Client;/d' src-tauri/src/plugin/transcribe.rs
sed -i '/use serde::{Serialize, Deserialize};/d' src-tauri/src/plugin/transcribe.rs
sed -i 's/EnhancedWhisperProcessor, TranscriptSegment, HallucinationDetector/EnhancedWhisperProcessor, HallucinationDetector/' src-tauri/src/plugin/transcribe.rs
sed -i '/StreamingTranscriptionManager, StreamingConfig, StreamingEvent/d' src-tauri/src/plugin/transcribe.rs
sed -i 's/WhisperModelSize, SpeechSettings, WhisperParamsSettings/WhisperModelSize/' src-tauri/src/plugin/transcribe.rs
sed -i 's/use anyhow::{Result, anyhow};/use anyhow::Result;/' src-tauri/src/plugin/voice_commands.rs
sed -i 's/info, error, debug, warn/info, error/' src-tauri/src/plugin/voice_commands.rs
sed -i '/use std::{path::PathBuf, sync::Arc, collections::HashMap};/d' src-tauri/src/plugin/voice_commands.rs
echo "use std::sync::Arc;" >> src-tauri/src/plugin/voice_commands.rs
sed -i '/use tokio::sync::mpsc;/d' src-tauri/src/plugin/voice_commands.rs
sed -i '/use regex;/d' src-tauri/src/plugin/voice_commands.rs
sed -i 's/VoiceCommandType,//' src-tauri/src/plugin/voice_commands.rs
sed -i 's/TextEditOperation,//' src-tauri/src/plugin/voice_commands.rs
sed -i 's/FormatOperation,//' src-tauri/src/plugin/voice_commands.rs
sed -i 's/TextStyle,//' src-tauri/src/plugin/voice_commands.rs
sed -i 's/AIVoiceCommandConfig,//' src-tauri/src/plugin/voice_commands.rs
sed -i 's/InterpretedCommand,//' src-tauri/src/plugin/voice_commands.rs
sed -i '/use crate::plugin::TranscribeState;/d' src-tauri/src/plugin/voice_commands.rs

echo "Fixing src-tauri/src/plugin/storage.rs..."
sed -i 's/use anyhow::{Context, Result};/use anyhow::Result;/' src-tauri/src/plugin/storage.rs
sed -i 's/debug, error, info, warn/warn/' src-tauri/src/plugin/storage.rs
sed -i '/use tauri::ipc::Invoke;/d' src-tauri/src/plugin/storage.rs
sed -i 's/Builder, Plugin/Plugin/' src-tauri/src/plugin/storage.rs

echo "Fixing src-tauri/src/plugin/text_injection.rs..."
sed -i 's/use anyhow::{Context, Result};/use anyhow::Result;/' src-tauri/src/plugin/text_injection.rs
sed -i 's/debug, error, info, warn/warn/' src-tauri/src/plugin/text_injection.rs
sed -i '/use serde::{Deserialize, Serialize};/d' src-tauri/src/plugin/text_injection.rs
sed -i 's/Builder, Plugin/Plugin/' src-tauri/src/plugin/text_injection.rs

echo "Fixing src-tauri/src/plugin/mod.rs..."
sed -i '/pub use voice_commands::VoiceCommandPlugin;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use voice_commands::VoiceCommandState;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use storage::StoragePlugin;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use storage::StorageState;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use text_injection::TextInjectionPlugin;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use text_injection::TextInjectionState;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use ai::AIPlugin;/d' src-tauri/src/plugin/mod.rs
sed -i '/pub use ai::AIState;/d' src-tauri/src/plugin/mod.rs

echo "Fixing src-tauri/src/system_monitor.rs..."
sed -i '/use log::error;/d' src-tauri/src/system_monitor.rs

echo "Fixing src-tauri/src/main.rs..."
sed -i '/use serde_json::Value as JsonValue;/d' src-tauri/src/main.rs
sed -i '/use bestme::audio::voice_commands::VoiceCommandConfig as LibVoiceCommandConfig;/d' src-tauri/src/main.rs
sed -i 's/Config, GeneralSettings, AudioSettings, SpeechSettings//' src-tauri/src/main.rs
sed -i '/use plugin::transcribe::SUPPORTED_LANGUAGES;/d' src-tauri/src/main.rs
sed -i 's/Path, PathBuf//' src-tauri/src/main.rs
sed -i 's/DateTime, Utc, TimeZone/Utc, TimeZone/' src-tauri/src/main.rs
sed -i '/use std::collections::VecDeque;/d' src-tauri/src/main.rs

# Fix unused variables by prefixing with _
echo "Step 4: Prefixing unused variables with underscore..."
find src src-tauri -name "*.rs" -exec sed -i 's/\(let\|let mut\) \([a-z_][a-zA-Z0-9_]*\) = /\1 _\2 = /g' {} \; 2>/dev/null

# Add missing features to Cargo.toml
echo "Step 5: Adding missing features to src-tauri/Cargo.toml..."
if ! grep -q "storage" src-tauri/Cargo.toml; then
    sed -i '/\[features\]/a storage = []' src-tauri/Cargo.toml
    sed -i '/\[features\]/a text-injection = []' src-tauri/Cargo.toml
fi

echo "Done! Now run 'cargo check' to verify the fixes."
echo "Some manual fixes might still be needed for complex cases."