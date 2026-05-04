---
name: wandb
description: "Weights & Biases — ML experiment tracking and visualization. Log metrics, hyperparameters, model checkpoints, and artifacts. Collaborative dashboards, sweep hyperparameter search, and model registry."
tags: [experiment-tracking, hyperparameter-sweeps, model-observability, training-visualization, wandb]
---
## Overview

Weights and Biases tracks ML experiments with metrics, hyperparameters, model checkpoints, and collaborative dashboards.

## Installation

```bash
uv pip install wandb
wandb login
```

## Basic Tracking

```python
import wandb

wandb.init(project="my_project", config={"lr": 0.001, "epochs": 10})
for epoch in range(10):
    loss = train_one_epoch()
    wandb.log({"train_loss": loss, "epoch": epoch})
wandb.finish()
```

## Hyperparameter Sweep

```python
sweep_config = {"method": "bayes", "metric": {"name": "loss", "goal": "minimize"},
                "parameters": {"lr": {"min": 1e-5, "max": 1e-2}}}
sweep_id = wandb.sweep(sweep_config, project="my_project")
wandb.agent(sweep_id, function=train_model)
```
