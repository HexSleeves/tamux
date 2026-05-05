---
name: ccxt
description: "Unified cryptocurrency exchange trading API. 100+ exchange clients (Binance, Coinbase, Kraken, Bybit, OKX). Market data, order management, websocket streaming, and arbitrage workflows."
tags: [crypto, exchange, trading, api, binance, coinbase, market-data, zorai]
---
## Overview

CCXT provides a unified API for 100+ cryptocurrency exchanges (Binance, Coinbase, Kraken, Bybit, OKX, KuCoin, Gate.io). Market data, order management, websocket streaming, and arbitrage detection through a single consistent interface.

## Installation

```bash
uv pip install ccxt
```

## Market Data

```python
import ccxt
exchange = ccxt.binance()
ticker = exchange.fetch_ticker("BTC/USDT")
print(f"{ticker['symbol']}: bid={ticker['bid']}, ask={ticker['ask']}, vol={ticker['baseVolume']}")

ohlcv = exchange.fetch_ohlcv("ETH/USDT", "1h", limit=100)
for ts, o, h, l, c, v in ohlcv:
    print(f"{ts}: O={o} H={h} L={l} C={c} V={v}")
```

## Trading

```python
exchange.create_market_buy_order("ETH/USDT", 0.1)
exchange.create_limit_sell_order("ETH/USDT", 0.1, 3000)
```

## References
- [CCXT docs](https://docs.ccxt.com/)
- [CCXT GitHub](https://github.com/ccxt/ccxt)