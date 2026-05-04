---
name: riskfolio-lib
description: "Portfolio risk and optimization: mean-variance, risk parity, CVaR, CDaR, worst-case, and robust optimization. Factor models, Black-Litterman, NCO. Supports plotting and interactive dashboards."
tags: [portfolio-optimization, risk-parity, cvar, factor-models, risk-management, quant-finance, zorai]
---
## Overview

Riskfolio-Lib provides portfolio optimization with mean-variance, risk parity, CVaR, CDaR, worst-case, and robust methods. Factor models, Black-Litterman, NCO, and built-in visualization.

## Installation

```bash
uv pip install riskfolio-lib
```

## Portfolio Optimization

```python
import riskfolio as rp
import yfinance as yf

prices = yf.download(["AAPL", "MSFT", "GOOGL", "AMZN", "NVDA"], start="2022-01-01")["Close"]
returns = prices.pct_change().dropna()

port = rp.Portfolio(returns=returns)
port.assets_stats(method_mu="hist", method_cov="hist")

# Mean-variance optimization
w = port.optimization(model="Classic", rm="MV", obj="Sharpe", hist=True)
