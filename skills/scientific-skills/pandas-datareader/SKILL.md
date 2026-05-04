---
name: pandas-datareader
description: "Multi-source financial data reader: FRED, World Bank, OECD, Eurostat, St. Louis Fed, Yahoo, Google, and more. Standard interface for economic and financial time series data ingestion."
tags: [financial-data, economic-data, fred, world-bank, time-series, python, zorai]
---
## Overview

Multi-source economic and financial data reader: FRED, World Bank, OECD, Eurostat, St. Louis Fed, Yahoo, and more. Standard interface for time series data ingestion across 200+ countries.

## Installation

```bash
uv pip install pandas-datareader
```

## FRED Data (Federal Reserve)

```python
import pandas_datareader.data as web
import datetime

start = datetime.datetime(2020, 1, 1)
end = datetime.datetime(2024, 12, 31)

gdp = web.DataReader("GDP", "fred", start, end)
unrate = web.DataReader("UNRATE", "fred", start, end)
cpi = web.DataReader("CPIAUCSL", "fred", start, end)
fedfunds = web.DataReader("FEDFUNDS", "fred", start, end)
