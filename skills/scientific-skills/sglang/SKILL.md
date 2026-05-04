---
name: sglang
description: "Structured Generation Language for LLM serving. RadixAttention prefix caching, constrained decoding (JSON, grammar), OpenAI-compatible API, and multi-turn optimization. Fast inference with structured output guarantees."
tags: [structured-generation, constrained-decoding, llm-serving, agent-inference, sglang]
---
## Overview

SGLang provides structured LLM serving with RadixAttention prefix caching, constrained decoding (JSON/grammar), and OpenAI-compatible API.

## Installation

```bash
uv pip install sglang[all]
```

## Server

```bash
python -m sglang.launch_server --model-path Qwen/Qwen2.5-7B-Instruct --port 30000
```

## Client

```python
from openai import OpenAI
client = OpenAI(base_url="http://localhost:30000/v1", api_key="none")
response = client.chat.completions.create(
    model="default", messages=[{"role": "user", "content": "Hello!"}],
)
```
