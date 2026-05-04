---
name: lancedb
description: "LanceDB — serverless vector database for AI. Columnar storage on Lance format, zero-copy access, multimodal search (text + images + audio), and direct DataFrame integration. No separate server."
tags: [lancedb, vector-database, embedded, multimodal, embeddings, python, zorai]
---
## Overview

LanceDB is a serverless vector database with zero-copy access, no separate server needed, and direct DataFrame integration.

## Installation

```bash
uv pip install lancedb
```

## Basic Usage

```python
import lancedb
import numpy as np

db = lancedb.connect("./my_db")
table = db.create_table("vectors", data=[
    {"vector": np.random.rand(384).tolist(), "text": "Paris is capital", "id": 1},
])
results = table.search(np.random.rand(384).tolist()).limit(5).to_pandas()
```
