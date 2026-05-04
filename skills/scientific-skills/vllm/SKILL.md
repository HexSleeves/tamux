---
name: vllm
description: "Fast LLM inference engine. PagedAttention, continuous batching, tensor parallelism, speculative decoding, and prefix caching. OpenAI-compatible API server. Supports Llama, Mistral, Qwen, DeepSeek, and hundreds of models."
tags: [llm-serving, paged-attention, openai-compatible-server, high-throughput-inference, vllm]
---
## Overview

vLLM is a fast LLM inference engine with PagedAttention, continuous batching, tensor parallelism, speculative decoding, and prefix caching. OpenAI-compatible API.

## Installation

```bash
uv pip install vllm
```

## Python API

```python
from vllm import LLM, SamplingParams

llm = LLM(model="Qwen/Qwen2.5-7B-Instruct")
sampling_params = SamplingParams(temperature=0.7, max_tokens=512)

outputs = llm.generate(["What is the capital of France?"], sampling_params)
for output in outputs:
    print(output.outputs[0].text)
```

## Server Mode

```bash
vllm serve Qwen/Qwen2.5-7B-Instruct --port 8000
# OpenAI-compatible: curl http://localhost:8000/v1/chat/completions
```

## Multi-GPU

```python
llm = LLM(model="mistralai/Mistral-7B-v0.1", tensor_parallel_size=2)
```
