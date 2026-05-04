---
name: milvus
description: "Milvus — cloud-native vector database for billion-scale similarity search. GPU-accelerated indexing, hybrid search, multi-vector, streaming, and time travel. Distributed deployment with Kubernetes."
tags: [milvus, vector-database, similarity-search, scale, embeddings, infrastructure, zorai]
---
## Overview

Milvus is a cloud-native vector database for billion-scale similarity search with GPU-accelerated indexing.

## Quick Start

```bash
docker compose -f milvus-standalone-docker-compose.yml up -d
```

## Python Client

```python
from pymilvus import connections, Collection, FieldSchema, CollectionSchema, DataType

connections.connect(host="localhost", port=19530)
schema = CollectionSchema([
    FieldSchema("id", DataType.INT64, is_primary=True),
    FieldSchema("embedding", DataType.FLOAT_VECTOR, dim=384),
])
collection = Collection("docs", schema)
collection.create_index("embedding", {"index_type": "IVF_FLAT", "metric_type": "L2"})
```
