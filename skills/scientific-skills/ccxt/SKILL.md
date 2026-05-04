---
name: ccxt
description: "Unified cryptocurrency exchange trading API. 100+ exchange clients (Binance, Coinbase, Kraken, Bybit, OKX). Market data, order management, websocket streaming, and arbitrage workflows."
tags: [crypto, exchange, trading, api, binance, coinbase, market-data, zorai]
---
## Overview

CCXT provides a unified API for 100+ cryptocurrency exchanges (Binance, Coinbase, Kraken, Bybit, OKX). Market data, order management, websocket streaming, and arbitrage workflows.

## Installation

```bash
uv pip install ccxt
```

## Market Data

```python
import ccxt

exchange = ccxt.binance()
exchange.load_markets()

ticker = exchange.fetch_ticker("BTC/USDT")
print(f"BTC: bid={ticker["bid"]}, ask={ticker["ask"]}")

# OHLCV
ohlcv = exchange.fetch_ohlcv("ETH/USDT", timeframe="1h", limit=100)
for candle in ohlcv:
    print(candle)  # [timestamp, open, high, low, close, volume]
