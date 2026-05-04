---
name: nixtla
description: "Nixtla ecosystem — statsforecast (statistical), neuralforecast (deep learning), hierarchicalforecast, and MLForecast. Production time series forecasting with AutoARIMA, ETS, Theta, Transformers, and ensemble blending."
tags: [nixtla, time-series, forecasting, arima, deep-learning, hierarchical, zorai]
---
## Overview

Nixtla ecosystem provides statistical, deep learning, and hierarchical time series forecasting: AutoARIMA, ETS, Theta, Transformers, and ensemble blending.

## Installation

```bash
uv pip install statsforecast neuralforecast
```

## Statistical Forecast

```python
from statsforecast import StatsForecast
from statsforecast.models import AutoARIMA

sf = StatsForecast(df=df, models=[AutoARIMA(season_length=12)], freq="M")
forecasts = sf.forecast(h=6)
print(forecasts)
```
