---
name: google-adk
description: "Google Agent Development Kit (ADK). Code-first Python toolkit for building, evaluating, and deploying AI agents. Multi-agent orchestration, tool integration, built-in evaluation, and deployment to Vertex AI."
tags: [google-adk, agent-framework, multi-agent, vertex-ai, google, python, zorai]
---
## Overview

Google Agent Development Kit (ADK) is a code-first Python toolkit for building, evaluating, and deploying AI agents with Vertex AI integration.

## Installation

```bash
uv pip install google-adk
```

## Simple Agent

```python
from google.adk.agents import Agent
from google.adk.tools import FunctionTool

def get_weather(location):
    return f"Weather in {location} is sunny."

agent = Agent(
    name="weather_agent",
    model="gemini-2.0-flash",
    instruction="You are a helpful weather assistant.",
    tools=[FunctionTool(get_weather)],
)

response = agent.run("What is the weather in Paris?")
print(response.content)
```
