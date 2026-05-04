---
name: mlflow
description: "MLflow — open-source MLOps platform. Experiment tracking, model registry, packaging, deployment, and evaluation. Multi-cloud ML workflows with reproducible runs and artifact logging."
tags: [mlflow, mlops, experiment-tracking, model-registry, deployment, python, zorai]
---
## Overview

MLflow is the open-source MLOps platform for experiment tracking, model registry, packaging, deployment, and evaluation.

## Installation

```bash
uv pip install mlflow
```

## Experiment Tracking

```python
import mlflow

mlflow.set_experiment("my_project")
with mlflow.start_run():
    mlflow.log_param("learning_rate", 0.001)
    mlflow.log_metric("accuracy", 0.95)
    mlflow.log_artifact("model.pkl")
    mlflow.pytorch.log_model(model, "model")
```

## Model Registry

```python
mlflow.register_model("runs:/RUN_ID/model", "MyModel")
```
