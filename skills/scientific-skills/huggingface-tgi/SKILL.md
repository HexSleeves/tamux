---
name: huggingface-tgi
description: "HuggingFace Text Generation Inference (TGI). High-performance LLM serving with continuous batching, tensor parallelism, watermarking, and OpenAI-compatible API. Native HF model hub integration."
tags: [tgi, llm-inference, huggingface, serving, text-generation, api, zorai]
---
## Overview

HuggingFace TGI provides high-performance LLM serving with continuous batching, tensor parallelism, and OpenAI-compatible API.

## Docker

```bash
docker run --gpus all -p 8080:80 -v $HOME/models:/data \
  ghcr.io/huggingface/text-generation-inference:latest \
  --model-id Qwen/Qwen2.5-7B-Instruct
```

## Client

```python
from openai import OpenAI
client = OpenAI(base_url="http://localhost:8080/v1", api_key="none")
response = client.chat.completions.create(
    model="tgi", messages=[{"role": "user", "content": "Hello!"}],
)
```
