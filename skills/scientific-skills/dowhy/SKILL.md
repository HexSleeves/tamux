---
name: dowhy
description: "DoWhy (Microsoft) — causal inference library. Causal graph modeling, identification (back-door, front-door, IV), estimation (matching, IPW, double-ML), and refutation/robustness checks for causal claims."
tags: [dowhy, causal-inference, causal-graph, identification, estimation, microsoft, zorai]
---
## Overview

DoWhy (Microsoft) provides causal inference with causal graph modeling, identification (back-door, front-door, IV), estimation, and refutation/robustness checks.

## Installation

```bash
uv pip install dowhy
```

## Basic Usage

```python
import dowhy
from dowhy import CausalModel

model = CausalModel(
    data=df,
    treatment="treatment",
    outcome="outcome",
    common_causes=["age", "gender"],
)

identified = model.identify_effect()
estimate = model.estimate_effect(identified, method_name="backdoor.linear_regression")
refute = model.refute_estimate(identified, estimate, method_name="placebo_treatment_refuter")
```
