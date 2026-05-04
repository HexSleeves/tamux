---
name: nvidia-nemo-guardrails
description: "NVIDIA NeMo Guardrails — programmable guardrails for LLM applications. Colang-based dialog management, topical rails (fact-checking, moderation), safety rails, and security rails for production AI."
tags: [nemo-guardrails, llm-safety, nvidia, colang, governance, moderation, zorai]
---
## Overview

NVIDIA NeMo Guardrails provides programmable guardrails with Colang-based dialog management, topical rails, safety rails, and security rails for production AI.

## Installation

```bash
uv pip install nemoguardrails
```

## Colang Config

```colang
define user ask about politics
  "What do you think about [topic]?"

define bot refuse politics
  "I am not able to discuss political topics."

define flow
  user ask about politics
  bot refuse politics
```
