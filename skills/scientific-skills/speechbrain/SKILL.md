---
name: speechbrain
description: "SpeechBrain — PyTorch speech toolkit. ASR, speaker recognition, speech separation, diarization, enhancement, language identification, and TTS. Recipe-based training with pre-trained model zoo."
tags: [speech-recognition, speaker-diarization, speaker-verification, speech-embeddings, speechbrain]
---
## Overview

SpeechBrain is a PyTorch speech toolkit for ASR, speaker recognition, speech separation, diarization, enhancement, and TTS.

## Installation

```bash
uv pip install speechbrain
```

## Speaker Verification

```python
from speechbrain.inference.speaker import SpeakerRecognition

verification = SpeakerRecognition.from_hparams(
    source="speechbrain/spkrec-ecapa-voxceleb",
    savedir="pretrained_models/spkrec",
)
score, prediction = verification.verify_files("speaker1.wav", "speaker2.wav")
print(f"Same speaker: {prediction}, score: {score:.3f}")
```
