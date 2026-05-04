---
name: epidemiology
description: "Disease modeling and epidemiological analysis: SIR/SEIR compartmental models, R0 estimation, outbreak simulation, incidence/prevalence forecasting, and intervention impact modeling."
tags: [epidemiology, disease-modeling, simulation, public-health, sir, zorai]
---
## Overview

Disease modeling and epidemiological analysis: SIR/SEIR compartmental models, R0 estimation, outbreak simulation, incidence/prevalence forecasting, and intervention impact modeling.

## Installation

```bash
uv pip install scipy numpy matplotlib
```

## SIR Model

```python
import numpy as np
from scipy.integrate import solve_ivp

def sir_model(t, y, beta, gamma):
    S, I, R = y
    dS = -beta * S * I
    dI = beta * S * I - gamma * I
    dR = gamma * I
    return [dS, dI, dR]

beta, gamma = 0.3, 0.1
R0 = beta / gamma
print(f"R0 = {R0:.2f}")

sol = solve_ivp(sir_model, [0, 160], [0.99, 0.01, 0], args=(beta, gamma), dense_output=True)
