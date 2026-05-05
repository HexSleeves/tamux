---
name: pymc-markets
description: "Bayesian inference for financial markets using PyMC. Stochastic volatility models, regime-switching, Bayesian portfolio optimization, factor models, and Markov chain Monte Carlo for risk estimation."
tags: [bayesian, pymc, stochastic-volatility, portfolio-optimization, risk, markets, zorai]
---
## Overview

PyMC provides Bayesian inference for financial modeling using probabilistic programming. Implements stochastic volatility models, regime-switching, Bayesian portfolio optimization, factor models, and MCMC risk estimation with the NUTS sampler.

## Installation

```bash
uv pip install pymc arviz
```

## Stochastic Volatility

```python
import pymc as pm
import numpy as np
import arviz as az

returns = np.random.randn(500) * 0.02

with pm.Model() as sv:
    sigma = pm.InverseGamma("sigma", alpha=2, beta=1)
    log_vol = pm.GaussianRandomWalk("log_vol", sigma=sigma, shape=len(returns))
    obs = pm.Normal("obs", mu=0, sigma=pm.math.exp(log_vol / 2), observed=returns)
    trace = pm.sample(1000, tune=1000)

az.plot_trace(trace)
```

## References
- [PyMC docs](https://www.pymc.io/)
- [Bayesian Methods for Hackers](https://github.com/CamDavidsonPilon/Probabilistic-Programming-and-Bayesian-Methods-for-Hackers)