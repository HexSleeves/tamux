---
name: pymc-markets
description: "Bayesian inference for financial markets using PyMC. Stochastic volatility models, regime-switching, Bayesian portfolio optimization, factor models, and Markov chain Monte Carlo for risk estimation."
tags: [bayesian, pymc, stochastic-volatility, portfolio-optimization, risk, markets, zorai]
---
## Overview

Bayesian inference for financial markets using PyMC: stochastic volatility models, regime-switching, Bayesian portfolio optimization, and factor models.

## Installation

```bash
uv pip install pymc arviz
```

## Stochastic Volatility

```python
import pymc as pm
import numpy as np

returns = np.random.normal(0, 0.02, 500)

with pm.Model() as sv_model:
    sigma = pm.InverseGamma("sigma", alpha=2, beta=1)
    h = pm.GaussianRandomWalk("h", sigma=sigma, shape=len(returns))
    obs = pm.Normal("obs", mu=0, sigma=pm.math.exp(h / 2), observed=returns)
    trace = pm.sample(1000, tune=1000)
