---
name: clip
description: "OpenAI CLIP — contrastive language-image pre-training. Zero-shot image classification, image-text similarity, concept search, and cross-modal retrieval. Embed images and text into shared space."
tags: [clip, multimodal, image-text, zero-shot, embeddings, openai, zorai]
---
## Overview

OpenAI CLIP provides zero-shot image classification and image-text similarity by embedding images and text into a shared space.

## Installation

```bash
uv pip install open-clip-torch
```

## Zero-Shot Classification

```python
import clip
import torch
from PIL import Image

model, preprocess = clip.load("ViT-B/32")
image = preprocess(Image.open("photo.jpg")).unsqueeze(0)
text = clip.tokenize(["a dog", "a cat", "a car"])

with torch.no_grad():
    logits_per_image, _ = model(image, text)
    probs = logits_per_image.softmax(dim=-1)
print(probs)
```
