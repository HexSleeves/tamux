---
name: dowhy
description: "DoWhy (Microsoft) — causal inference library. Causal graph modeling, identification (back-door, front-door, IV), estimation (matching, IPW, double-ML), and refutation/robustness checks for causal claims."
tags: [dowhy, causal-inference, causal-graph, identification, estimation, microsoft, zorai]
---
## Overview

DoWhy (Microsoft/py-why) provides end-to-end causal inference with causal graph modeling (DAG specification), identification (back-door, front-door, IV), estimation (linear regression, matching, IV, double-ML), and refutation (placebo, bootstrap, random common cause).

## Installation

```bash
uv pip install dowhy
```

## Example

```python
from dowhy import CausalModel

model = CausalModel(
    data=df,
    treatment="treatment",
    outcome="outcome",
    common_causes=["age", "gender", "income"],
)

# Identify causal effect
identified = model.identify_effect(proceed_when_unidentifiable=True)

# Estimate
estimate = model.estimate_effect(identified, method_name="backdoor.linear_regression")
print(f"ATE: {estimate.value:.4f}")

# Refute
refute = model.refute_estimate(identified, estimate, method_name="placebo_treatment_refuter")
print(refute)
```

## References
- [DoWhy docs](https://www.pywhy.org/dowhy/)
- [DoWhy GitHub](https://github.com/py-why/dowhy)