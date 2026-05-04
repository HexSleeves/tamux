---
name: quantlib-python
description: "QuantLib Python bindings for quantitative finance. Pricing and risk analytics for fixed income, equity, FX, credit derivatives, and structured products. Yield curves, options, swaps, bonds, and Monte Carlo simulation."
tags: [quantitative-finance, derivatives, options, fixed-income, risk-management, quantlib, zorai]
---
## Overview

QuantLib Python provides pricing and risk analytics for fixed income, equity, FX, and credit derivatives. Yield curves, options, swaps, bonds, and Monte Carlo simulation.

## Installation

```bash
uv pip install QuantLib-Python
```

## Bond Pricing

```python
import QuantLib as ql

ql.Settings.instance().evaluationDate = ql.Date(15, 6, 2024)
issue = ql.Date(15, 6, 2023)
maturity = ql.Date(15, 6, 2028)
schedule = ql.Schedule(issue, maturity, ql.Period(ql.Semiannual),
                       ql.UnitedStates(ql.UnitedStates.GovernmentBond),
                       ql.Unadjusted, ql.Unadjusted, ql.DateGeneration.Backward, False)

bond = ql.FixedRateBond(2, 100.0, schedule, [0.05], ql.ActualActual())
ytm = bond.bondYield(95.0, ql.ActualActual(), ql.Compounded, ql.Semiannual)
print(f"YTM: {ytm:.4%}")
