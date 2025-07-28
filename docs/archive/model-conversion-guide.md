# Model Conversion Guide

This guide explains how to convert AI models for use with BestMe's text enhancement features.

## Prerequisites

Install required Python packages:
```bash
pip install transformers optimum onnx onnxruntime torch
```

## Quick Start

### 1. Convert a Model

```bash
# Convert Phi-3-mini model
python scripts/convert_to_onnx.py microsoft/Phi-3-mini-4k-instruct

# Convert with quantization (reduces size by ~75%)
python scripts/convert_to_onnx.py microsoft/Phi-3-mini-4k-instruct --quantize

# Convert Grammar-T5 model
python scripts/convert_to_onnx.py vennify/t5-base-grammar-correction
```

### 2. Verify Conversion

```bash
python scripts/convert_to_onnx.py <model_name> --verify
```

## Supported Models

### Pre-configured Models

| Model | Type | Use Case |
|-------|------|----------|
| `microsoft/Phi-3-mini-4k-instruct` | Causal LM | General text enhancement |
| `meta-llama/Llama-3.2-1B` | Causal LM | Fast inference |
| `vennify/t5-base-grammar-correction` | Seq2Seq | Grammar correction |
| `grammarly/coedit-base` | Seq2Seq | Text editing |

### Model Types

1. **Seq2Seq Models** (T5, BART)
   - Best for: Grammar correction, paraphrasing
   - Input: Text to correct
   - Output: Corrected text

2. **Causal LM** (GPT, Llama, Phi)
   - Best for: Text completion, generation
   - Input: Prompt/context
   - Output: Generated continuation

3. **Token Classifiers** (BERT-based)
   - Best for: Error detection, POS tagging
   - Input: Text
   - Output: Token labels

## Conversion Options

### Basic Usage
```bash
python scripts/convert_to_onnx.py <model_name> [options]
```

### Options
- `-o, --output`: Output directory (default: `models/<model_name>`)
- `-q, --quantize`: Create int8 quantized version
- `--no-optimize`: Skip graph optimization
- `--opset`: ONNX opset version (default: 14)
- `--verify`: Verify converted model

## Directory Structure

After conversion:
```
models/
└── microsoft_Phi-3-mini-4k-instruct/
    ├── model.onnx              # Main model file
    ├── model_quantized.onnx    # Quantized version (if --quantize)
    ├── tokenizer.json          # Tokenizer config
    ├── tokenizer_config.json   # Tokenizer settings
    └── special_tokens_map.json # Special tokens
```

## Integration with BestMe

1. **Place Models**: Put converted models in BestMe's models directory:
   ```bash
   ~/.local/share/bestme/models/
   ```

2. **Register Model**: Add to model registry in code:
   ```rust
   ModelMetadata {
       id: "custom-model",
       source: ModelSource::Local { 
           path: "~/.local/share/bestme/models/custom-model/model.onnx" 
       },
       // ... other metadata
   }
   ```

3. **Use in App**: Select model in BestMe's AI settings

## Performance Tips

### 1. Quantization
Reduces model size by ~75% with minimal accuracy loss:
```bash
python scripts/convert_to_onnx.py <model> --quantize
```

### 2. Model Selection
- **Large models** (>1GB): Better quality, slower
- **Small models** (<500MB): Faster, good for real-time
- **Quantized models**: Best balance

### 3. Hardware Optimization
- **GPU**: Use CUDA/Metal providers for 2-5x speedup
- **CPU**: Use quantized models for better performance
- **Memory**: Ensure 2x model size available

## Troubleshooting

### Common Issues

1. **Out of Memory**
   ```
   Solution: Use smaller model or quantization
   ```

2. **Slow Conversion**
   ```
   Solution: Normal for large models, be patient
   ```

3. **Import Errors**
   ```
   Solution: Install all requirements
   pip install -r requirements.txt
   ```

### Verification Checklist

✅ Model file exists (`model.onnx`)
✅ Tokenizer files present
✅ File sizes reasonable
✅ Verification passes
✅ Test inference works

## Advanced Usage

### Custom Models

1. **Fine-tuned Models**
   ```bash
   # Convert your fine-tuned model
   python scripts/convert_to_onnx.py ./my-finetuned-model
   ```

2. **Private Models**
   ```bash
   # Use HF token for private models
   export HF_TOKEN=your_token
   python scripts/convert_to_onnx.py private-org/model
   ```

### Batch Conversion

Convert multiple models:
```bash
#!/bin/bash
models=(
    "microsoft/Phi-3-mini-4k-instruct"
    "meta-llama/Llama-3.2-1B"
    "vennify/t5-base-grammar-correction"
)

for model in "${models[@]}"; do
    python scripts/convert_to_onnx.py "$model" --quantize
done
```

## Model Optimization

### Graph Optimization
Automatic optimizations applied:
- Operator fusion
- Constant folding
- Shape inference
- Redundancy elimination

### Quantization Types
- **Dynamic Int8**: Best for CPU (default)
- **Static Int8**: Requires calibration data
- **FP16**: For GPU inference

## Next Steps

1. Convert desired models
2. Test with BestMe
3. Compare performance
4. Choose best model for your use case

For more information, see the [AI Integration Guide](./ai-integration-guide.md).