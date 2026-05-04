---
name: captum
description: "Captum (PyTorch) — model interpretability and feature attribution. Integrated Gradients, DeepLIFT, SmoothGrad, Occlusion, SHAP approximation, and Layer-wise Relevance Propagation. For vision and text models."
tags: [captum, explainability, feature-attribution, integrated-gradients, pytorch, interpretability, zorai]
---
## Overview

Captum (PyTorch) provides model interpretability with Integrated Gradients, DeepLIFT, SmoothGrad, Occlusion, and Layer-wise Relevance Propagation.

## Installation

```bash
uv pip install captum
```

## Feature Attribution

```python
from captum.attr import IntegratedGradients
import torch

ig = IntegratedGradients(model)
input_tensor = torch.randn(1, 3, 224, 224)
baseline = torch.zeros(1, 3, 224, 224)
attributions, delta = ig.attribute(input_tensor, baseline, target=0, return_convergence_delta=True)
```
