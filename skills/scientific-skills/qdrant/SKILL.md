---
name: qdrant
description: "Qdrant — vector similarity search engine. Payload filtering, quantized indexing, multi-tenant, and horizontal scaling. REST and gRPC API. Docker-native deployment for production RAG and recommendation."
tags: [qdrant, vector-database, similarity-search, embeddings, rag, infrastructure, zorai]
---
## Overview

Qdrant is a vector similarity search engine with payload filtering, quantized indexing, and horizontal scaling. REST and gRPC API.

## Quick Start

```bash
docker run -p 6333:6333 qdrant/qdrant
```

## Python Client

```python
from qdrant_client import QdrantClient, models

client = QdrantClient("localhost", port=6333)
client.create_collection("docs", vectors_config=models.VectorParams(size=384, distance=models.Distance.COSINE))
client.upsert("docs", points=[models.PointStruct(id=1, vector=[0.1]*384, payload={"text": "Paris is capital"})])
results = client.search("docs", query_vector=[0.1]*384, limit=5)
```
