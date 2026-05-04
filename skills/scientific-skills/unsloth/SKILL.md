---
name: unsloth
description: "Fast QLoRA/QLoRA fine-tuning with 2x faster training and 50% less memory. Supports Llama, Mistral, Gemma, Qwen, DeepSeek, Phi, Yi, Falcon. Flash Attention, 4-bit quantization. No quality loss."
tags: [qlora-finetuning, lora-finetuning, memory-efficient-tuning, quantized-llm-training, unsloth]
---
## Overview

Unsloth provides 2x faster QLoRA training with 50% less memory. Supports Llama, Mistral, Gemma, Qwen, DeepSeek, Phi with Flash Attention and 4-bit quantization.

## Installation

```bash
uv pip install unsloth
```

## QLoRA Fine-Tuning

```python
from unsloth import FastLanguageModel
import torch

model, tokenizer = FastLanguageModel.from_pretrained(
    model_name="unsloth/Qwen2.5-7B-Instruct-bnb-4bit",
    max_seq_length=4096,
    dtype=torch.bfloat16,
    load_in_4bit=True,
)

model = FastLanguageModel.get_peft_model(
    model,
    r=16,
    target_modules=["q_proj", "k_proj", "v_proj", "o_proj"],
    lora_alpha=16,
    lora_dropout=0,
    use_gradient_checkpointing="unsloth",
)
print(f"Trainable: {model.print_trainable_parameters()}")
```

## Inference

```python
FastLanguageModel.for_inference(model)
inputs = tokenizer(["Describe quantum computing."], return_tensors="pt").to("cuda")
outputs = model.generate(**inputs, max_new_tokens=256)
print(tokenizer.decode(outputs[0]))
```
