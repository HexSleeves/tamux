---
name: zipline-reloaded
description: "Zipline Reloaded — event-driven backtesting engine. Minute and daily data, custom factors, pipeline API, risk and performance analytics. Forked from Quantopian's Zipline for continued development."
tags: [backtesting, quant-finance, event-driven, factor-models, pipeline, trading, zorai]
---
## Overview

Zipline Reloaded is an event-driven backtesting engine with minute/daily data, custom factors, pipeline API, and built-in risk/performance analytics.

## Installation

```bash
uv pip install zipline-reloaded
```

## Strategy Example

```python
from zipline.api import order_target, record, symbol
from zipline import run_algorithm

def initialize(context):
    context.asset = symbol("AAPL")
    context.has_position = False

def handle_data(context, data):
    price = data.current(context.asset, "price")
    sma_20 = data.history(context.asset, "price", 20, "1d").mean()
    sma_50 = data.history(context.asset, "price", 50, "1d").mean()

    if sma_20 > sma_50 and not context.has_position:
        order_target(context.asset, 100)
        context.has_position = True
    elif sma_20 < sma_50 and context.has_position:
        order_target(context.asset, 0)
        context.has_position = False
