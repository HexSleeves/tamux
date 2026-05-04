---
name: openai-agents-sdk
description: "OpenAI Agents SDK — build agentic workflows with handoffs, guardrails, and tool integration. Single-agent to multi-agent orchestration. Tracing and observability. Python-first SDK from OpenAI."
tags: [openai, agents-sdk, agent-framework, handoffs, guardrails, orchestration, zorai]
---
## Overview

OpenAI Agents SDK provides agentic workflows with handoffs, guardrails, tool integration, and built-in tracing.

## Installation

```bash
uv pip install openai-agents
```

## Basic Agent

```python
from agents import Agent, Runner, function_tool

@function_tool
def search_web(query):
    return f"Results for: {query}"

agent = Agent(
    name="research_assistant",
    instructions="Search the web for answers.",
    tools=[search_web],
)

result = Runner.run_sync(agent, "Find latest AI research.")
print(result.final_output)
```
