---
name: yfinance
description: "Yahoo Finance market data downloader. Stock prices, options chains, fundamentals, dividends, splits, earnings, institutional holders, and financial statements. Quick data ingestion for quant research and backtesting."
tags: [yahoo-finance, market-data, stocks, etfs, financial-data, api, zorai]
---
## Overview

yfinance downloads Yahoo Finance market data: stock prices, options chains, fundamentals, dividends, splits, earnings, and financial statements.

## Installation

```bash
uv pip install yfinance
```

## Price History

```python
import yfinance as yf

msft = yf.download("MSFT", start="2024-01-01", end="2024-12-31")
print(msft.head())

ticker = yf.Ticker("AAPL")
hist = ticker.history(period="1y")
```

## Fundamentals

```python
ticker = yf.Ticker("MSFT")
print(ticker.balance_sheet)
print(ticker.info["marketCap"], ticker.info["peRatio"])
```

## Multiple Tickers

```python
data = yf.download(["AAPL", "MSFT", "GOOGL"], start="2024-01-01")
print(data["Close"].head())
```
