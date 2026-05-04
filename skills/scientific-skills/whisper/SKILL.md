---
name: whisper
description: "OpenAI Whisper — general-purpose speech recognition. Multilingual transcription, translation to English, and speaker-agnostic ASR. Models from tiny to large. Robust to noise, accents, and technical vocabulary."
tags: [speech-to-text, multilingual-asr, audio-transcription, translation-asr, whisper]
---
## Overview

OpenAI Whisper provides multilingual speech recognition with models from tiny to large, robust to noise and accents.

## Installation

```bash
uv pip install openai-whisper
```

## Transcription

```python
import whisper

model = whisper.load_model("base")  # tiny, base, small, medium, large
result = model.transcribe("audio.mp3")
print(result["text"])

# Translate to English
result = model.transcribe("french_audio.mp3", task="translate")
```
