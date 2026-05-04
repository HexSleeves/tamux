---
name: llama-cpp
description: "LLM inference in C/C++ with Python bindings. GPU acceleration via CUDA/Metal/Vulkan, 2-8 bit quantization (GGUF), KV cache, and grammar-based sampling. Run Llama, Mistral, Gemma, Phi locally."
tags: [llama-cpp, gguf, quantization, local-llm, inference, python, zorai]
---
## Overview

llama.cpp provides LLM inference on CPU and GPU with GGUF quantization (2-8 bit), CUDA/Metal/Vulkan, and grammar-based sampling.

## Installation

```bash
uv pip install llama-cpp-python
# GPU: CMAKE_ARGS="-DGGML_CUDA=on" uv pip install llama-cpp-python
```

## Inference

```python
from llama_cpp import Llama
llm = Llama(model_path="qwen2.5-7b-q4_k_m.gguf", n_ctx=4096, n_gpu_layers=-1)
output = llm("Q: What is the capital of France? A:",
             max_tokens=64, temperature=0.7)
print(output["choices"][0]["text"])
```
