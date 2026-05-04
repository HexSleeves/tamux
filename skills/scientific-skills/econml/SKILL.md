---
name: econml
description: "EconML (Microsoft) — heterogeneous treatment effect estimation. Double ML, Causal Forest, Deep IV, and metalearners (S-Learner, T-Learner, X-Learner). Orthogonal learning for causal effects from observational data."
tags: [econml, causal-inference, heterogeneous-treatment-effects, causal-forest, microsoft, econometrics, zorai]
---
## Overview

EconML (Microsoft) estimates heterogeneous treatment effects with Double ML, Causal Forest, Deep IV, and metalearners (S-Learner, T-Learner, X-Learner).

## Installation

```bash
uv pip install econml
```

## Double ML

```python
from econml.dml import LinearDML
from sklearn.ensemble import GradientBoostingRegressor

est = LinearDML(model_y=GradientBoostingRegressor(), model_t=GradientBoostingRegressor())
est.fit(Y=outcome, T=treatment)
treatment_effects = est.effect(X=features)
```
