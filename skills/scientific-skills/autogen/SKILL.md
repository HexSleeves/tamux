---
name: autogen
description: "AutoGen (Microsoft) — multi-agent conversation framework. Agent-to-agent chat, code generation & execution, tool use, group chat, and human-in-the-loop. Build collaborative AI systems with specialized agents."
tags: [autogen, multi-agent, conversation, microsoft, llm, agent-framework, zorai]
---
## Overview

AutoGen (Microsoft) provides multi-agent conversations with code generation, tool use, group chat, and human-in-the-loop.

## Installation

```bash
uv pip install pyautogen
```

## Two-Agent Chat

```python
import autogen

config_list = [{"model": "gpt-4", "api_key": "YOUR_KEY"}]

assistant = autogen.AssistantAgent(
    name="assistant",
    llm_config={"config_list": config_list},
)

user_proxy = autogen.UserProxyAgent(
    name="user_proxy",
    human_input_mode="NEVER",
    code_execution_config={"work_dir": "coding", "use_docker": False},
)

user_proxy.initiate_chat(
    assistant,
    message="Write a Python script to download stock data and plot it.",
)
```
