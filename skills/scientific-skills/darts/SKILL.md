---
name: darts
description: "Darts — time series forecasting library by Unit8. Unified API across ARIMA, Prophet, CatBoost, N-BEATS, TFT, TCN, Transformer, and RNN models. Backtesting, probabilistic forecasting, and covariate support."
tags: [darts, time-series, forecasting, deep-learning, probabilistic, backtesting, zorai]
---
## Overview

Darts (Unit8) provides unified time series forecasting across ARIMA, Prophet, N-BEATS, TFT, TCN, and Transformer models. Backtesting and probabilistic forecasts.

## Installation

```bash
uv pip install darts
```

## Basic Forecast

```python
from darts import TimeSeries
from darts.models import NBEATSModel

series = TimeSeries.from_csv("data.csv", time_col="date", value_cols="value")
model = NBEATSModel(input_chunk_length=24, output_chunk_length=12)
model.fit(series)
forecast = model.predict(12)
model.plot(forecast)
```
