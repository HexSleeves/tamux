---
name: feast
description: "Feast — open-source feature store. Online and offline serving, point-in-time joins, feature validation, and streaming ingestion. Standardizes feature management across training and production."
tags: [feast, feature-store, mlops, feature-engineering, online-serving, infrastructure, zorai]
---
## Overview

Feast is an open-source feature store for production ML. Serves features for training (offline via batch queries) and inference (online via low-latency API) with point-in-time correctness, feature validation, streaming ingestion, and a registry for governance.

## Installation

```bash
uv pip install feast
```

## Define Features

```python
from feast import Entity, FeatureView, FileSource, ValueType
from datetime import timedelta

driver = Entity(name="driver_id", value_type=ValueType.INT64)
source = FileSource(path="data/driver_stats.parquet", timestamp_field="event_timestamp")
fv = FeatureView(
    name="driver_hourly_stats",
    entities=[driver],
    ttl=timedelta(hours=2),
    source=source,
)
```

## Serve

```bash
feast apply
feast serve  # online serving at localhost:6566
```

## References
- [Feast docs](https://docs.feast.dev/)
- [Feast GitHub](https://github.com/feast-dev/feast)