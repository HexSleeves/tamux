---
name: sdv
description: "Synthetic Data Vault (SDV) — generate synthetic tabular data. Single-table, multi-table, and sequential data synthesis. CTGAN, TVAE, CopulaGAN, GaussianCopula. Privacy metrics and evaluation."
tags: [sdv, synthetic-data, data-generation, privacy, ctgan, tabular-data, zorai]
---
## Overview

Synthetic Data Vault (SDV) generates synthetic tabular, multi-table, and sequential data with CTGAN, TVAE, CopulaGAN, and GaussianCopula models.

## Installation

```bash
uv pip install sdv
```

## Single Table

```python
from sdv.single_table import CTGANSynthesizer
from sdv.datasets.demo import load_demo

data, metadata = load_demo()
synthesizer = CTGANSynthesizer(metadata)
synthesizer.fit(data)
synthetic_data = synthesizer.sample(100)
```
