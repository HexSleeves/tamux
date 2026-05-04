---
name: open-clip
description: "OpenCLIP — open-source implementation of CLIP trained on LAION-5B/OpenCLIP datasets. Multi-head attention pooling, SigLIP loss variants, and wide model zoo (ViT, ConvNeXt, EVA). Community-driven."
tags: [open-clip, multimodal, image-text, laion, zero-shot, embeddings, zorai]
---
## Overview

OpenCLIP is an open-source CLIP implementation trained on LAION-5B with larger model zoo (ViT, ConvNeXt, SigLIP).

## Installation

```bash
uv pip install open-clip-torch
```

## Usage

```python
import open_clip
import torch
from PIL import Image

model, _, preprocess = open_clip.create_model_and_transforms("ViT-H-14", pretrained="laion2b_s32b_b79k")
tokenizer = open_clip.get_tokenizer("ViT-H-14")

image = preprocess(Image.open("photo.jpg")).unsqueeze(0)
text = tokenizer(["a dog", "a cat"])
with torch.no_grad():
    image_features = model.encode_image(image)
    text_features = model.encode_text(text)
```
