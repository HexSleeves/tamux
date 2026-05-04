---
name: trl
description: "Transformer Reinforcement Learning library (TRL). Supervised fine-tuning (SFT), reward modeling, PPO, DPO, KTO, GRPO for RLHF. Process reward models and language model alignment."
tags: [rlhf-post-training, dpo-training, ppo-alignment, supervised-finetuning, trl]
---
## Overview

TRL provides Supervised Fine-Tuning (SFT), reward modeling, PPO, DPO, KTO, and GRPO for LLM alignment. Standard library from Hugging Face for RLHF pipelines.

## Installation

```bash
uv pip install trl
```

## SFT

```python
from trl import SFTTrainer
from transformers import AutoModelForCausalLM, AutoTokenizer

model = AutoModelForCausalLM.from_pretrained("Qwen/Qwen2.5-1.5B-Instruct")
tokenizer = AutoTokenizer.from_pretrained("Qwen/Qwen2.5-1.5B-Instruct")

trainer = SFTTrainer(
    model=model, tokenizer=tokenizer,
    train_dataset=dataset,
    args=dict(per_device_train_batch_size=4, learning_rate=2e-5, max_seq_length=2048),
)
trainer.train()
```

## DPO

```python
from trl import DPOTrainer

dpo_trainer = DPOTrainer(
    model=model, ref_model=ref_model, tokenizer=tokenizer,
    train_dataset=preference_dataset,
    args=dict(per_device_train_batch_size=4, max_length=2048),
)
dpo_trainer.train()
```
