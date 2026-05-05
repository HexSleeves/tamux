---
name: autogen
description: "AutoGen (Microsoft) — multi-agent conversation framework. Agent-to-agent chat, code generation & execution, tool use, group chat, and human-in-the-loop. Build collaborative AI systems with specialized agents."
tags: [autogen, multi-agent, conversation, microsoft, llm, agent-framework, zorai]
---
## Overview

AutoGen (Microsoft) enables multi-agent conversations where LLM agents collaborate on complex tasks. Supports code generation and execution, tool use, group chat, and human-in-the-loop. Well-suited for multi-step reasoning and coding tasks.

## Installation

```bash
uv pip install pyautogen
```

## Two-Agent Chat

```python
import autogen

config_list = [{"model": "gpt-4", "api_key": "sk-your-key"}]

assistant = autogen.AssistantAgent(
    name="coder",
    llm_config={"config_list": config_list},
)

user = autogen.UserProxyAgent(
    name="user",
    human_input_mode="NEVER",
    code_execution_config={"work_dir": "coding", "use_docker": False},
)

user.initiate_chat(
    assistant,
    message="Write a Python script to download stock prices and plot a moving average crossover.",
)
```

## References
- [AutoGen docs](https://microsoft.github.io/autogen/)
- [AutoGen GitHub](https://github.com/microsoft/autogen)