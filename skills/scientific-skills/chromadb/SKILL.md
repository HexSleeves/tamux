---
name: chromadb
description: "Chroma — AI-native embedding database. In-process, lightweight vector store with automatic embedding, metadata filtering, and full-text search. Simplest path from prototype to production RAG."
tags: [chromadb, vector-database, embeddings, rag, semantic-search, python, zorai]
---
## Overview

Chroma is an AI-native embedding database optimized for RAG workflows. In-process, lightweight, with automatic embedding via sentence-transformers, metadata filtering, semantic search, and no separate server required. Simplest path from prototype to production.

## Installation

```bash
uv pip install chromadb
```

## Persistent Usage

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_db")
collection = client.create_collection(name="documents")

collection.add(
    documents=["Paris is the capital of France.", "Berlin is the capital of Germany."],
    metadatas=[{"country": "France"}, {"country": "Germany"}],
    ids=["doc1", "doc2"],
)

results = collection.query(
    query_texts=["What is the capital of France?"],
    n_results=3,
    where={"country": "France"},
)
print(results["documents"])
```

## References
- [Chroma docs](https://docs.trychroma.com/)
- [Chroma GitHub](https://github.com/chroma-core/chroma)