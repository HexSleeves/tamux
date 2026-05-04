---
name: peft
description: "Parameter-Efficient Fine-Tuning (PEFT) library. LoRA, QLoRA, AdaLoRA, IA3, Prefix Tuning, P-Tuning, Prompt Tuning. Fine-tune large models with minimal memory overhead. Hugging Face ecosystem integration."
tags: [peft, lora, qlora, fine-tuning, llm, huggingface, parameter-efficient, zorai]
---
## Overview

PEFT enables efficient adaptation of large pretrained models by fine-tuning only a small number of extra parameters. LoRA, QLoRA, AdaLoRA, IA3, Prefix Tuning, and Prompt Tuning. Reduces GPU memory by 4-16x vs full fine-tuning.

## Installation

```bash
uv pip install peft
```

## LoRA Fine-Tuning

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import LoraConfig, get_peft_model, TaskType

model_id = "Qwen/Qwen2.5-3B-Instruct"
model = AutoModelForCausalLM.from_pretrained(model_id)
tokenizer = AutoTokenizer.from_pretrained(model_id)

peft_config = LoraConfig(
    r=16, lora_alpha=32,
    target_modules=["q_proj", "k_proj", "v_proj", "o_proj"],
    lora_dropout=0.05, bias="none",
    task_type=TaskType.CAUSAL_LM,
)
model = get_peft_model(model, peft_config)
model.print_trainable_parameters()

# Train with standard HF Trainer
```

## QLoRA (Quantized)

```python
from transformers import BitsAndBytesConfig
import torch

bnb_config = BitsAndBytesConfig(
    load_in_4bit=True,
    bnb_4bit_quant_type="nf4",
    bnb_4bit_compute_dtype=torch.bfloat16,
)
model = AutoModelForCausalLM.from_pretrained(
    model_id, quantization_config=bnb_config, device_map="auto"
)
model = get_peft_model(model, peft_config)
```

## Saving/Merging

```python
model.save_pretrained("my-lora-adapter")
from peft import PeftModel
base = AutoModelForCausalLM.from_pretrained(model_id)
merged = PeftModel.from_pretrained(base, "my-lora-adapter").merge_and_unload()
```
