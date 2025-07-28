pub mod registry;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub size_bytes: u64,
    pub format: ModelFormat,
    pub source: ModelSource,
    pub capabilities: Vec<Capability>,
    pub performance: PerformanceProfile,
    pub requirements: ModelRequirements,
    pub architecture: String,
    pub parameters: String,
    pub performance_class: String,
    pub context_window: usize,
    pub supports_gpu: bool,
    pub download_url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ModelFormat {
    ONNX,
    Safetensors,
    PyTorch,
    TensorFlow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    HuggingFace { repo: String, file: String },
    Direct { url: String },
    Local { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Capability {
    GrammarCorrection,
    Punctuation,
    IntentDetection,
    StyleTransformation,
    Summarization,
    Translation,
    QuestionAnswering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub avg_latency_ms: f64,
    pub tokens_per_second: f64,
    pub memory_usage_mb: u64,
    pub supports_batch: bool,
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirements {
    pub min_ram_gb: f32,
    pub min_vram_gb: Option<f32>,
    pub supports_cpu: bool,
    pub supports_gpu: bool,
    pub supported_backends: Vec<String>,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self {
            avg_latency_ms: 100.0,
            tokens_per_second: 50.0,
            memory_usage_mb: 1024,
            supports_batch: true,
            max_batch_size: 8,
        }
    }
}

// Pre-defined model configurations
pub fn get_phi3_mini_metadata() -> ModelMetadata {
    ModelMetadata {
        id: "phi-3-mini".to_string(),
        name: "microsoft/Phi-3-mini-4k-instruct".to_string(),
        display_name: "Phi-3 Mini (3.8B)".to_string(),
        description: "Microsoft's compact language model optimized for instruction following and text enhancement".to_string(),
        size_bytes: 7_600_000_000, // ~7.6GB
        format: ModelFormat::ONNX,
        source: ModelSource::HuggingFace {
            repo: "microsoft/Phi-3-mini-4k-instruct-onnx".to_string(),
            file: "onnx/model.onnx".to_string(),
        },
        capabilities: vec![
            Capability::GrammarCorrection,
            Capability::Punctuation,
            Capability::IntentDetection,
            Capability::StyleTransformation,
        ],
        performance: PerformanceProfile {
            avg_latency_ms: 45.0,
            tokens_per_second: 120.0,
            memory_usage_mb: 8192,
            supports_batch: true,
            max_batch_size: 16,
        },
        requirements: ModelRequirements {
            min_ram_gb: 8.0,
            min_vram_gb: Some(4.0),
            supports_cpu: true,
            supports_gpu: true,
            supported_backends: vec![
                "CUDA".to_string(),
                "CoreML".to_string(),
                "DirectML".to_string(),
                "CPU".to_string(),
            ],
        },
        architecture: "Transformer".to_string(),
        parameters: "3.8B".to_string(),
        performance_class: "High".to_string(),
        context_window: 4096,
        supports_gpu: true,
        download_url: Some("https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-onnx".to_string()),
        sha256: None,
    }
}

pub fn get_llama32_1b_metadata() -> ModelMetadata {
    ModelMetadata {
        id: "llama-3.2-1b".to_string(),
        name: "meta-llama/Llama-3.2-1B".to_string(),
        display_name: "Llama 3.2 (1B)".to_string(),
        description: "Meta's efficient 1B parameter model for fast text processing".to_string(),
        size_bytes: 2_000_000_000, // ~2GB
        format: ModelFormat::Safetensors,
        source: ModelSource::HuggingFace {
            repo: "meta-llama/Llama-3.2-1B".to_string(),
            file: "model.safetensors".to_string(),
        },
        capabilities: vec![
            Capability::GrammarCorrection,
            Capability::Punctuation,
            Capability::IntentDetection,
        ],
        performance: PerformanceProfile {
            avg_latency_ms: 20.0,
            tokens_per_second: 200.0,
            memory_usage_mb: 2048,
            supports_batch: true,
            max_batch_size: 32,
        },
        requirements: ModelRequirements {
            min_ram_gb: 4.0,
            min_vram_gb: Some(2.0),
            supports_cpu: true,
            supports_gpu: true,
            supported_backends: vec![
                "CUDA".to_string(),
                "CoreML".to_string(),
                "DirectML".to_string(),
                "CPU".to_string(),
            ],
        },
        architecture: "Transformer".to_string(),
        parameters: "1B".to_string(),
        performance_class: "Medium".to_string(),
        context_window: 2048,
        supports_gpu: true,
        download_url: Some("https://huggingface.co/meta-llama/Llama-3.2-1B".to_string()),
        sha256: None,
    }
}

pub fn get_grammar_t5_metadata() -> ModelMetadata {
    ModelMetadata {
        id: "grammar-t5-base".to_string(),
        name: "vennify/t5-base-grammar-correction".to_string(),
        display_name: "Grammar T5 Base".to_string(),
        description: "T5 model fine-tuned specifically for grammar correction".to_string(),
        size_bytes: 900_000_000, // ~900MB
        format: ModelFormat::ONNX,
        source: ModelSource::HuggingFace {
            repo: "vennify/t5-base-grammar-correction".to_string(),
            file: "onnx/model.onnx".to_string(),
        },
        capabilities: vec![
            Capability::GrammarCorrection,
            Capability::Punctuation,
        ],
        performance: PerformanceProfile {
            avg_latency_ms: 30.0,
            tokens_per_second: 150.0,
            memory_usage_mb: 1024,
            supports_batch: true,
            max_batch_size: 16,
        },
        requirements: ModelRequirements {
            min_ram_gb: 2.0,
            min_vram_gb: Some(1.0),
            supports_cpu: true,
            supports_gpu: true,
            supported_backends: vec![
                "CUDA".to_string(),
                "CoreML".to_string(),
                "CPU".to_string(),
            ],
        },
        architecture: "T5".to_string(),
        parameters: "220M".to_string(),
        performance_class: "Low".to_string(),
        context_window: 512,
        supports_gpu: true,
        download_url: Some("https://huggingface.co/vennify/t5-base-grammar-correction".to_string()),
        sha256: None,
    }
}