---
name: freqtrade
description: "Open-source crypto trading bot. Strategy development in Python, backtesting, hyperparameter optimization, dry-run and live trading. Supports major exchanges via CCXT. Telegram integration for monitoring."
tags: [crypto, trading-bot, backtesting, automation, strategy, ccxt, zorai]
---
## Overview

Freqtrade is an open-source crypto trading bot with strategy development in Python, backtesting, hyperparameter optimization, and live trading via CCXT.

## Installation

```bash
git clone https://github.com/freqtrade/freqtrade.git
cd freqtrade
uv pip install -e .
```

## Basic Strategy

```python
from freqtrade.strategy import IStrategy

class MyStrategy(IStrategy):
    timeframe = "1h"
    minimal_roi = {"0": 0.01}
    stoploss = -0.05

    def populate_indicators(self, dataframe, metadata):
        dataframe["rsi"] = 100 - (100 / (1 + dataframe["close"] / dataframe["close"].shift(14)))
        return dataframe

    def populate_buy_trend(self, dataframe, metadata):
        dataframe.loc[(dataframe["rsi"] < 30) & (dataframe["volume"] > 0), "buy"] = 1
        return dataframe
