---
name: onnx-runtime
description: "ONNX Runtime — cross-platform ML inference optimizer. Convert PyTorch, TensorFlow, scikit-learn models to ONNX. GPU, CPU, and mobile acceleration. Quantization, graph optimization, and custom ops."
tags: [onnx, model-optimization, inference, cross-platform, quantization, edge, zorai]
---
## Overview

ONNX Runtime provides cross-platform ML inference optimization with GPU, CPU, and mobile acceleration. Quantization and graph optimization.

## Installation

```bash
uv pip install onnxruntime onnxruntime-gpu
```

## Inference

```python
import onnxruntime as ort
import numpy as np

session = ort.InferenceSession("model.onnx")
input_name = session.get_inputs()[0].name
output = session.run(None, {input_name: np.random.randn(1, 3, 224, 224).astype(np.float32)})
print(output[0].shape)
```
