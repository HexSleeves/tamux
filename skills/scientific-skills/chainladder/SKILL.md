---
name: chainladder
description: "Property & casualty insurance loss reserving in Python. Chain ladder, Bornhuetter-Ferguson, Cape Cod, bootstrap simulation, and loss development pattern estimation. Actuarial triangle operations."
tags: [insurance, actuarial, loss-reserving, chain-ladder, p-and-c, claims, zorai]
---
## Overview

Chainladder implements property and casualty insurance loss reserving: chain ladder, Bornhuetter-Ferguson, Cape Cod, bootstrap simulation, and loss development pattern estimation.

## Installation

```bash
uv pip install chainladder
```

## Basic Chain Ladder

```python
import chainladder as cl

tri = cl.load_sample("usauto")
print(tri)

model = cl.ChainLadder()
result = model.fit(tri)
print(result.ultimate_)
print(result.ibnr_)
