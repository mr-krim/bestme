//! Voice Activity Detection (VAD) module for efficient audio processing

use anyhow::Result;
use log::{debug, info};
use std::collections::VecDeque;

/// Voice Activity Detector for identifying speech segments
pub struct VoiceActivityDetector {
    /// Energy threshold for voice detection
    threshold: f32,
    
    /// Minimum duration of speech in samples
    min_speech_samples: usize,
    
    /// Maximum duration of silence in samples
    max_silence_samples: usize,
    
    /// Sample rate
    sample_rate: usize,
    
    /// Rolling energy buffer for smoothing
    energy_buffer: VecDeque<f32>,
    
    /// Buffer size for energy smoothing
    buffer_size: usize,
    
    /// Current state
    is_speaking: bool,
    
    /// Consecutive speech samples
    speech_count: usize,
    
    /// Consecutive silence samples
    silence_count: usize,
    
    /// Adaptive threshold enabled
    adaptive_threshold: bool,
    
    /// Noise floor for adaptive threshold
    noise_floor: f32,
}

impl VoiceActivityDetector {
    /// Create a new VAD instance
    pub fn new(
        threshold: f32,
        min_speech_duration_ms: u32,
        max_silence_duration_ms: u32,
        sample_rate: usize,
    ) -> Self {
        let min_speech_samples = (min_speech_duration_ms as f32 * sample_rate as f32 / 1000.0) as usize;
        let max_silence_samples = (max_silence_duration_ms as f32 * sample_rate as f32 / 1000.0) as usize;
        
        Self {
            threshold,
            min_speech_samples,
            max_silence_samples,
            sample_rate,
            energy_buffer: VecDeque::with_capacity(10),
            buffer_size: 10,
            is_speaking: false,
            speech_count: 0,
            silence_count: 0,
            adaptive_threshold: true,
            noise_floor: 0.01,
        }
    }
    
    /// Process audio samples and detect voice activity
    pub fn process(&mut self, audio: &[f32]) -> VADResult {
        let energy = self.calculate_energy(audio);
        
        // Update rolling buffer
        self.energy_buffer.push_back(energy);
        if self.energy_buffer.len() > self.buffer_size {
            self.energy_buffer.pop_front();
        }
        
        // Calculate smoothed energy
        let smoothed_energy = self.energy_buffer.iter().sum::<f32>() / self.energy_buffer.len() as f32;
        
        // Update adaptive threshold if enabled
        if self.adaptive_threshold && !self.is_speaking {
            self.update_noise_floor(smoothed_energy);
        }
        
        let effective_threshold = if self.adaptive_threshold {
            self.noise_floor * 3.0 // 3x noise floor
        } else {
            self.threshold
        };
        
        // Detect speech
        let is_speech = smoothed_energy > effective_threshold;
        
        if is_speech {
            self.speech_count += audio.len();
            self.silence_count = 0;
            
            if !self.is_speaking && self.speech_count >= self.min_speech_samples {
                self.is_speaking = true;
                debug!("Speech started (energy: {:.4}, threshold: {:.4})", smoothed_energy, effective_threshold);
                return VADResult::SpeechStart;
            }
        } else {
            self.silence_count += audio.len();
            self.speech_count = 0;
            
            if self.is_speaking && self.silence_count >= self.max_silence_samples {
                self.is_speaking = false;
                debug!("Speech ended (energy: {:.4}, threshold: {:.4})", smoothed_energy, effective_threshold);
                return VADResult::SpeechEnd;
            }
        }
        
        if self.is_speaking {
            VADResult::Speech
        } else {
            VADResult::Silence
        }
    }
    
    /// Calculate energy of audio samples
    fn calculate_energy(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = audio.iter().map(|&x| x * x).sum();
        (sum_squares / audio.len() as f32).sqrt()
    }
    
    /// Update noise floor for adaptive threshold
    fn update_noise_floor(&mut self, energy: f32) {
        // Exponential moving average
        let alpha = 0.1;
        self.noise_floor = alpha * energy + (1.0 - alpha) * self.noise_floor;
        self.noise_floor = self.noise_floor.max(0.001); // Minimum noise floor
    }
    
    /// Reset the VAD state
    pub fn reset(&mut self) {
        self.is_speaking = false;
        self.speech_count = 0;
        self.silence_count = 0;
        self.energy_buffer.clear();
        self.noise_floor = 0.01;
    }
    
    /// Check if currently detecting speech
    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }
    
    /// Set adaptive threshold
    pub fn set_adaptive_threshold(&mut self, enabled: bool) {
        self.adaptive_threshold = enabled;
    }
    
    /// Get current threshold (effective threshold if adaptive)
    pub fn get_threshold(&self) -> f32 {
        if self.adaptive_threshold {
            self.noise_floor * 3.0
        } else {
            self.threshold
        }
    }
}

/// VAD processing result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VADResult {
    /// No speech detected
    Silence,
    /// Speech detected
    Speech,
    /// Speech just started
    SpeechStart,
    /// Speech just ended
    SpeechEnd,
}

/// Advanced VAD using machine learning (placeholder for future implementation)
pub struct AdvancedVAD {
    // This would use a small neural network for more accurate detection
    // For now, we'll use the energy-based VAD
    basic_vad: VoiceActivityDetector,
}

impl AdvancedVAD {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            basic_vad: VoiceActivityDetector::new(0.02, 250, 2000, sample_rate),
        }
    }
    
    pub fn process(&mut self, audio: &[f32]) -> VADResult {
        // In the future, this would use a neural network
        // For now, delegate to basic VAD
        self.basic_vad.process(audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vad_creation() {
        let vad = VoiceActivityDetector::new(0.02, 250, 2000, 16000);
        assert!(!vad.is_speaking());
        assert_eq!(vad.min_speech_samples, 4000); // 250ms at 16kHz
        assert_eq!(vad.max_silence_samples, 32000); // 2s at 16kHz
    }
    
    #[test]
    fn test_energy_calculation() {
        let vad = VoiceActivityDetector::new(0.02, 250, 2000, 16000);
        
        // Silence
        let silence = vec![0.0; 100];
        assert_eq!(vad.calculate_energy(&silence), 0.0);
        
        // Loud signal
        let loud = vec![0.5; 100];
        let energy = vad.calculate_energy(&loud);
        assert!(energy > 0.4 && energy < 0.6);
    }
    
    #[test]
    fn test_speech_detection() {
        let mut vad = VoiceActivityDetector::new(0.1, 100, 500, 16000);
        
        // Process silence
        let silence = vec![0.01; 1600]; // 100ms of silence
        for _ in 0..5 {
            let result = vad.process(&silence);
            assert_eq!(result, VADResult::Silence);
        }
        
        // Process speech
        let speech = vec![0.3; 1600]; // 100ms of speech
        let mut detected_start = false;
        for i in 0..10 {
            let result = vad.process(&speech);
            if result == VADResult::SpeechStart {
                detected_start = true;
            }
            if i > 2 {
                assert!(vad.is_speaking());
            }
        }
        assert!(detected_start);
    }
}