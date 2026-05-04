---
name: dgl
description: "Deep Graph Library (DGL) — graph neural network framework. GCN, GAT, GraphSAGE, RGCN, and custom message-passing. Heterogeneous graphs, temporal graphs, and large-scale training with mini-batch sampling."
tags: [dgl, graph-neural-network, gnn, message-passing, deep-learning, python, zorai]
---
## Overview

Deep Graph Library (DGL) provides GNN frameworks: GCN, GAT, GraphSAGE, RGCN with heterogeneous, temporal, and large-scale graph support.

## Installation

```bash
uv pip install dgl
```

## GCN

```python
import dgl
import torch
import torch.nn as nn
import torch.nn.functional as F
from dgl.nn import GraphConv

class GCN(nn.Module):
    def __init__(self, in_feats, hidden, out_feats):
        super().__init__()
        self.conv1 = GraphConv(in_feats, hidden)
        self.conv2 = GraphConv(hidden, out_feats)

    def forward(self, g, features):
        x = F.relu(self.conv1(g, features))
        x = self.conv2(g, x)
        return x
```
