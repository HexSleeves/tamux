---
name: fredapi
description: "Federal Reserve Economic Data (FRED) API client. 800,000+ US and international economic time series: GDP, inflation, unemployment, interest rates, industrial production. Direct data access for macro research."
tags: [fred, economic-data, macro-economics, federal-reserve, time-series, api, zorai]
---
## Overview

FRED API provides access to 800,000+ US and international economic time series from the Federal Reserve Bank of St. Louis.

## Installation

```bash
uv pip install fredapi
```

## Basic Usage

```python
from fredapi import Fred

fred = Fred(api_key="YOUR_API_KEY")
gdp = fred.get_series("GDP")
unemployment = fred.get_series("UNRATE")
cpi = fred.get_series("CPIAUCSL")

# Search
results = fred.search("gross domestic product")
print(results[["id", "title", "frequency"]].head())
