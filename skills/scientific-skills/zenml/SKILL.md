---
name: zenml
description: "ZenML — ML pipeline orchestration. Connect ML tools (MLflow, W&B, Airflow, Kubeflow) into portable pipelines. Caching, versioning, and cloud-agnostic stack management for production ML workflows."
tags: [ml-pipeline-orchestration, reproducible-pipelines, stack-management, pipeline-caching, zenml]
---
## Overview

ZenML connects ML tools into portable pipelines with caching, versioning, and cloud-agnostic stack management.

## Installation

```bash
uv pip install zenml
```

## Pipeline

```python
from zenml import pipeline, step

@step
def load_data():
    return [1, 2, 3, 4, 5]

@step
def train_model(data):
    print(f"Training on {len(data)} samples")

@pipeline
def training_pipeline():
    data = load_data()
    train_model(data)

training_pipeline()
```
