---
name: bentoml
description: "BentoML — model serving and deployment. Build prediction services from any ML framework with OpenAPI/Swagger. Containerize, deploy to Kubernetes, AWS, GCP, Azure. Adaptive batching and GPU support."
tags: [bentoml, model-serving, deployment, mlops, api, kubernetes, zorai]
---
## Overview

BentoML serves ML models with OpenAPI/Swagger endpoints. Deploy to Kubernetes, AWS, GCP, Azure with adaptive batching and GPU support.

## Model Service

```python
import bentoml
import numpy as np

@bentoml.service
class PredictService:
    def __init__(self):
        self.model = bentoml.sklearn.load_model("my_model:latest")
    @bentoml.api(batchable=True)
    def predict(self, data: np.ndarray) -> np.ndarray:
        return self.model.predict(data)
```

## Deploy

```bash
bentoml build
bentoml containerize my_service:latest
```
