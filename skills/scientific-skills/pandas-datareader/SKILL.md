---
name: pandas-datareader
description: "Multi-source financial data reader: FRED, World Bank, OECD, Eurostat, St. Louis Fed, Yahoo, Google, and more. Standard interface for economic and financial time series data ingestion."
tags: [financial-data, economic-data, fred, world-bank, time-series, python, zorai]
---
## Overview

pandas-datareader provides a unified interface for reading economic and financial time series from multiple sources: FRED (Federal Reserve), World Bank, OECD, Eurostat, Yahoo Finance, and St. Louis Fed. Standard data ingestion for macroeconomic research and quantitative analysis.

## Installation

```bash
uv pip install pandas-datareader
```

## FRED (Federal Reserve Economic Data)

```python
import pandas_datareader.data as web
import datetime

start = datetime.datetime(2020, 1, 1)
gdp = web.DataReader("GDP", "fred", start)
cpi = web.DataReader("CPIAUCSL", "fred", start)
fedfunds = web.DataReader("FEDFUNDS", "fred", start)
unemp = web.DataReader("UNRATE", "fred", start)
```

## World Bank

```python
gdp_pc = web.DataReader("NY.GDP.PCAP.CD", "worldbank", start=2015)
```

## References
- [pandas-datareader docs](https://pandas-datareader.readthedocs.io/)
- [FRED API](https://fred.stlouisfed.org/docs/api/fred/)