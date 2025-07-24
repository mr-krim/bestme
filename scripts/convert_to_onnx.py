#!/usr/bin/env python3
"""
Convert Hugging Face models to ONNX format for use with BestMe.

This script handles conversion of various model types including:
- T5 models (for grammar correction)
- GPT/Llama models (for text generation)
- BERT-style models (for classification)

Requirements:
    pip install transformers optimum onnx onnxruntime torch
"""

import os
import sys
import argparse
import logging
from pathlib import Path
from typing import Optional, Tuple

try:
    from transformers import (
        AutoTokenizer,
        AutoModelForSeq2SeqLM,
        AutoModelForCausalLM,
        AutoModelForTokenClassification,
        T5ForConditionalGeneration,
    )
    from optimum.onnxruntime import (
        ORTModelForSeq2SeqLM,
        ORTModelForCausalLM,
        ORTModelForTokenClassification,
    )
    from optimum.exporters.onnx import main_export
    import onnx
    from onnxruntime.quantization import quantize_dynamic, QuantType
    from onnxruntime.transformers.optimizer import optimize_model
except ImportError as e:
    print(f"Error: Missing required packages. Please install with:")
    print("pip install transformers optimum onnx onnxruntime torch")
    print(f"Import error: {e}")
    sys.exit(1)

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class ModelConverter:
    """Handles conversion of models to ONNX format."""
    
    SUPPORTED_MODELS = {
        # Grammar correction models
        "vennify/t5-base-grammar-correction": "seq2seq",
        "grammarly/coedit-base": "seq2seq",
        
        # General language models
        "microsoft/Phi-3-mini-4k-instruct": "causal",
        "meta-llama/Llama-3.2-1B": "causal",
        "meta-llama/Llama-3.2-3B": "causal",
        
        # Custom patterns
        "t5-": "seq2seq",
        "gpt": "causal",
        "llama": "causal",
        "phi": "causal",
    }
    
    def __init__(self, cache_dir: Optional[Path] = None):
        self.cache_dir = cache_dir or Path.home() / ".cache" / "bestme" / "models"
        self.cache_dir.mkdir(parents=True, exist_ok=True)
    
    def detect_model_type(self, model_name: str) -> str:
        """Detect the type of model based on its name."""
        # Check exact matches first
        if model_name in self.SUPPORTED_MODELS:
            return self.SUPPORTED_MODELS[model_name]
        
        # Check patterns
        model_lower = model_name.lower()
        for pattern, model_type in self.SUPPORTED_MODELS.items():
            if pattern in model_lower:
                return model_type
        
        # Default to seq2seq for unknown models
        logger.warning(f"Unknown model type for {model_name}, defaulting to seq2seq")
        return "seq2seq"
    
    def convert_model(
        self,
        model_name: str,
        output_dir: Path,
        quantize: bool = False,
        optimize: bool = True,
        opset_version: int = 14,
    ) -> Tuple[Path, Path]:
        """
        Convert a Hugging Face model to ONNX format.
        
        Args:
            model_name: HuggingFace model ID
            output_dir: Directory to save ONNX files
            quantize: Whether to quantize the model
            optimize: Whether to optimize the ONNX graph
            opset_version: ONNX opset version
            
        Returns:
            Tuple of (model_path, tokenizer_path)
        """
        output_dir.mkdir(parents=True, exist_ok=True)
        model_type = self.detect_model_type(model_name)
        
        logger.info(f"Converting {model_name} ({model_type}) to ONNX...")
        
        # Export to ONNX using optimum
        onnx_path = output_dir / "model.onnx"
        
        # Use optimum's export functionality
        logger.info("Exporting model to ONNX...")
        main_export(
            model_name_or_path=model_name,
            output=str(output_dir),
            task=self._get_task_name(model_type),
            opset=opset_version,
            device="cpu",  # Export on CPU
            cache_dir=str(self.cache_dir),
        )
        
        # Save tokenizer
        logger.info("Saving tokenizer...")
        tokenizer = AutoTokenizer.from_pretrained(model_name, cache_dir=self.cache_dir)
        tokenizer.save_pretrained(output_dir)
        tokenizer_path = output_dir / "tokenizer.json"
        
        # Optimize ONNX model
        if optimize:
            logger.info("Optimizing ONNX model...")
            self._optimize_onnx_model(onnx_path, model_type)
        
        # Quantize if requested
        if quantize:
            logger.info("Quantizing model to int8...")
            quantized_path = output_dir / "model_quantized.onnx"
            self._quantize_model(onnx_path, quantized_path)
            # Keep both versions
            logger.info(f"Quantized model saved to {quantized_path}")
        
        logger.info(f"Conversion complete! Model saved to {onnx_path}")
        return onnx_path, tokenizer_path
    
    def _get_task_name(self, model_type: str) -> str:
        """Get the task name for optimum export."""
        task_map = {
            "seq2seq": "text2text-generation",
            "causal": "text-generation",
            "token_classifier": "token-classification",
        }
        return task_map.get(model_type, "text2text-generation")
    
    def _optimize_onnx_model(self, model_path: Path, model_type: str):
        """Optimize ONNX model for inference."""
        try:
            # Load and optimize
            if model_type == "seq2seq":
                opt_model = optimize_model(
                    str(model_path),
                    model_type="t5",
                    num_heads=12,  # Will be auto-detected
                    hidden_size=768,  # Will be auto-detected
                    optimization_options=None,
                )
            else:
                opt_model = optimize_model(
                    str(model_path),
                    model_type="gpt2",
                    optimization_options=None,
                )
            
            # Save optimized model
            opt_path = model_path.parent / "model_optimized.onnx"
            opt_model.save_model_to_file(str(opt_path))
            
            # Replace original with optimized
            os.replace(opt_path, model_path)
            logger.info("Model optimization complete")
            
        except Exception as e:
            logger.warning(f"Optimization failed: {e}. Using unoptimized model.")
    
    def _quantize_model(self, input_path: Path, output_path: Path):
        """Quantize ONNX model to int8."""
        try:
            quantize_dynamic(
                str(input_path),
                str(output_path),
                weight_type=QuantType.QInt8,
                optimize_model=True,
            )
        except Exception as e:
            logger.error(f"Quantization failed: {e}")
            raise
    
    def verify_model(self, model_path: Path) -> bool:
        """Verify that the ONNX model is valid."""
        try:
            # Load and check model
            model = onnx.load(str(model_path))
            onnx.checker.check_model(model)
            
            # Get model info
            input_names = [i.name for i in model.graph.input]
            output_names = [o.name for o in model.graph.output]
            
            logger.info(f"Model verified successfully!")
            logger.info(f"Inputs: {input_names}")
            logger.info(f"Outputs: {output_names}")
            
            return True
        except Exception as e:
            logger.error(f"Model verification failed: {e}")
            return False


def main():
    parser = argparse.ArgumentParser(
        description="Convert Hugging Face models to ONNX format for BestMe"
    )
    parser.add_argument(
        "model_name",
        help="HuggingFace model ID (e.g., 'microsoft/Phi-3-mini-4k-instruct')"
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        help="Output directory (default: models/<model_name>)"
    )
    parser.add_argument(
        "-q", "--quantize",
        action="store_true",
        help="Quantize model to int8"
    )
    parser.add_argument(
        "--no-optimize",
        action="store_true",
        help="Skip ONNX optimization"
    )
    parser.add_argument(
        "--opset",
        type=int,
        default=14,
        help="ONNX opset version (default: 14)"
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify the converted model"
    )
    
    args = parser.parse_args()
    
    # Set output directory
    if args.output:
        output_dir = args.output
    else:
        model_safe_name = args.model_name.replace("/", "_")
        output_dir = Path("models") / model_safe_name
    
    # Convert model
    converter = ModelConverter()
    
    try:
        model_path, tokenizer_path = converter.convert_model(
            args.model_name,
            output_dir,
            quantize=args.quantize,
            optimize=not args.no_optimize,
            opset_version=args.opset,
        )
        
        # Verify if requested
        if args.verify:
            if converter.verify_model(model_path):
                logger.info("✅ Model verification passed!")
            else:
                logger.error("❌ Model verification failed!")
                sys.exit(1)
        
        print(f"\n✅ Conversion successful!")
        print(f"Model: {model_path}")
        print(f"Tokenizer: {tokenizer_path}")
        
    except Exception as e:
        logger.error(f"Conversion failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()