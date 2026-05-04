---
name: chromadb
description: "Chroma — AI-native embedding database. In-process, lightweight vector store with automatic embedding, metadata filtering, and full-text search. Simplest path from prototype to production RAG."
tags: [chromadb, vector-database, embeddings, rag, semantic-search, python, zorai]
---
## Overview

Chroma is an AI-native embedding database. Lightweight, in-process, with automatic embedding, metadata filtering, and semantic search.

## Installation

```bash
uv pip install chromadb
```

## Basic Usage

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_db")
collection = client.create_collection(name="docs")

collection.add(
    documents=["Paris is the capital of France.", "Berlin is the capital of Germany."],
    metadatas=[{"country": "France"}, {"country": "Germany"}],
    ids=["doc1", "doc2"],
)

results = collection.query(query_texts=["French capital"], n_results=1)
print(results["documents"])
```
