---
name: quantlib-python
description: "QuantLib Python bindings for quantitative finance. Pricing and risk analytics for fixed income, equity, FX, credit derivatives, and structured products. Yield curves, options, swaps, bonds, and Monte Carlo simulation."
tags: [quantitative-finance, derivatives, options, fixed-income, risk-management, quantlib, zorai]
---
## Overview

QuantLib Python provides pricing and risk analytics for fixed income, equity, FX, and credit derivatives. It is the standard open-source quant finance library, covering yield curves, options, swaps, bonds, caps/floors, swaptions, and structured products.

## Installation

```bash
uv pip install QuantLib-Python
```

## Bond Pricing

```python
import QuantLib as ql

ql.Settings.instance().evaluationDate = ql.Date(15, 6, 2024)
schedule = ql.Schedule(
    ql.Date(15, 6, 2023), ql.Date(15, 6, 2028),
    ql.Period(ql.Semiannual),
    ql.UnitedStates(ql.UnitedStates.GovernmentBond),
    ql.Unadjusted, ql.Unadjusted,
    ql.DateGeneration.Backward, False)

bond = ql.FixedRateBond(2, 100.0, schedule, [0.05], ql.ActualActual(ql.ActualActual.ISMA))
ytm = bond.bondYield(95.0, ql.ActualActual(), ql.Compounded, ql.Semiannual)
print(f"YTM: {ytm:.4%}")
```

## References
- [QuantLib docs](https://www.quantlib.org/)
- [QuantLib-Python docs](https://quantlib-python-docs.readthedocs.io/)