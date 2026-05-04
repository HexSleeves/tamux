---
name: langgraph
description: "LangGraph — orchestrate LLM agents as stateful graphs. Multi-agent coordination, persistent state, human-in-the-loop, streaming, checkpointing, and conditional control flow. Build complex agent workflows."
tags: [langgraph, agent-orchestration, state-machine, multi-agent, langchain, llm, zorai]
---
## Overview

LangGraph orchestrates LLM agents as stateful graphs with multi-agent coordination, persistent state, human-in-the-loop, streaming, and checkpointing.

## Installation

```bash
uv pip install langgraph
```

## Simple Agent Graph

```python
from typing import TypedDict
from langgraph.graph import StateGraph, END

class AgentState(TypedDict):
    messages: list
    next_step: str

def research_node(state):
    return {"messages": state["messages"], "next_step": "write"}

def write_node(state):
    return {"messages": state["messages"], "next_step": "review"}

graph = StateGraph(AgentState)
graph.add_node("research", research_node)
graph.add_node("write", write_node)
graph.set_entry_point("research")
graph.add_edge("research", "write")
graph.add_edge("write", END)
app = graph.compile()

result = app.invoke({"messages": [], "next_step": ""})
```
