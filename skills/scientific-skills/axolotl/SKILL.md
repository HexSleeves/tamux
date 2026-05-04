---
name: axolotl
description: "Streamlined fine-tuning framework for LLMs. Supports full fine-tune, LoRA, QLoRA, FSDP, DeepSpeed, and multi-GPU. YAML config driven. Works with Llama, Mistral, Qwen, DeepSeek, and hundreds of HF models."
tags: [axolotl, fine-tuning, llm, lora, deep-speed, huggingface, zorai]
---
## Overview

Axolotl streamlines LLM fine-tuning with YAML-driven configuration. Supports full fine-tune, LoRA, QLoRA, FSDP, DeepSpeed, multi-GPU.

## Installation

```bash
git clone https://github.com/OpenAccess-AI-Collective/axolotl.git
cd axolotl
uv pip install -e .
```

## YAML Config

```yaml
base_model: Qwen/Qwen2.5-7B-Instruct
adapter: lora
lora_r: 16
lora_alpha: 32
lora_target_modules:
  - q_proj
  - v_proj
sequence_len: 2048
batch_size: 2
learning_rate: 2e-5
num_epochs: 3
```

## Run

```bash
accelerate launch -m axolotl.cli.train config.yml
```
