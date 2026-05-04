---
name: guardrails-ai
description: "Guardrails AI — LLM output validation and guardrails. Define guardrails as XML/JSON specs, validate outputs against structural and semantic constraints, correct/retry on failure, and audit model behavior."
tags: [guardrails-ai, llm-safety, output-validation, guardrails, governance, python, zorai]
---
## Overview

Guardrails AI validates LLM outputs against structural and semantic constraints, with automatic retry/correction on failure.

## Installation

```bash
uv pip install guardrails-ai
```

## Output Validation

```python
import guardrails as gd

rail_spec = """
<rail version="0.1">
<output>
  <string name="summary" description="Brief summary" format="length: 1-100"/>
  <integer name="score" format="range: 1-10"/>
</output>
</rail>
"""

guard = gd.Guard.from_rail_string(rail_spec)
validated = guard.parse("Some long model output here...")
print(validated)
```
