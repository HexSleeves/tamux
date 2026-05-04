---
name: feast
description: "Feast — open-source feature store. Online and offline serving, point-in-time joins, feature validation, and streaming ingestion. Standardizes feature management across training and production."
tags: [feast, feature-store, mlops, feature-engineering, online-serving, infrastructure, zorai]
---
## Overview

Feast is an open-source feature store for ML with online/offline serving, point-in-time joins, and streaming ingestion.

## Installation

```bash
uv pip install feast
```

## Feature Definition

```python
from feast import Entity, FeatureView, FileSource, ValueType
from datetime import timedelta

driver = Entity(name="driver", value_type=ValueType.INT64)
driver_fv = FeatureView(
    name="driver_hourly_stats",
    entities=[driver],
    ttl=timedelta(hours=2),
    source=FileSource(path="data/driver_stats.parquet"),
)
```

## Serve

```bash
feast apply  # register features
feast serve  # online serving endpoint
```
