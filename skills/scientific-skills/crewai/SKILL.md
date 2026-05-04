---
name: crewai
description: "CrewAI — multi-agent AI framework. Role-based agents with defined goals, tools, and memory. Hierarchical and sequential task execution. Human input delegation and process orchestration."
tags: [crewai, multi-agent, agent-framework, collaboration, llm, orchestration, zorai]
---
## Overview

CrewAI enables role-based multi-agent AI systems with defined goals, tools, and memory. Sequential or hierarchical execution.

## Installation

```bash
uv pip install crewai
```

## Multi-Agent Crew

```python
from crewai import Agent, Task, Crew

researcher = Agent(
    role="Research Analyst",
    goal="Find latest AI developments",
    backstory="Expert at finding information",
)

writer = Agent(
    role="Technical Writer",
    goal="Write clear summary",
    backstory="Skilled at explaining tech topics",
)

task = Task(
    description="Search for latest AI agent frameworks",
    expected_output="List of frameworks with features",
    agent=researcher,
)

crew = Crew(agents=[researcher, writer], tasks=[task])
result = crew.kickoff()
```
