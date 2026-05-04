---
name: lifelines
description: "Survival analysis in Python: Kaplan-Meier, Cox proportional hazard, Aalen additive, parametric models, and competing risks. Censored data handling for churn, clinical, and actuarial applications."
tags: [survival-analysis, kaplan-meier, cox-model, actuarial, churn, statistics, zorai]
---
## Overview

Lifelines implements survival analysis: Kaplan-Meier, Cox proportional hazard, Aalen additive, parametric models, and competing risks. Handles censored data.

## Installation

```bash
uv pip install lifelines
```

## Kaplan-Meier

```python
from lifelines import KaplanMeierFitter

T = [6, 12, 18, 24, 30, 36]
E = [1, 0, 1, 1, 0, 1]

kmf = KaplanMeierFitter()
kmf.fit(T, E, label="Treatment")
kmf.plot_survival_function()
print(kmf.median_survival_time_)
