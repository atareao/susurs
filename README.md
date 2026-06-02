# susurs

![CI](https://img.shields.io/github/actions/workflow/status/atareao/susurs/ci.yml?style=flat-square)
![Crates.io](https://img.shields.io/crates/v/susurs?style=flat-square)
![License](https://img.shields.io/crates/l/susurs?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-2024-edition?style=flat-square)

A fully offline Text-to-Speech CLI tool that converts text to MP3 or WAV audio using local AI TTS models. Supports 9 Spanish voices with configurable speech parameters.

## Features

- **Offline operation** - All processing happens locally after initial model download
- **9 Spanish TTS models** - Including Argentine, Spanish (Spain), and Mexican variants
- **Multiple output formats** - MP3 (via pure-Rust `shine-rs`) and WAV (via `hound`)
- **Configurable speech** - Rate, pitch, volume, noise scale, length scale, silence scale, speaker ID
- **Presets** - `human`, `fast`, `slow`, `quiet`, `loud` with per-parameter override
- **Model management** - List, delete, update, and clean cached models
- **Audio quality controls** - Bitrate selection, sample rate override, normalization, silence padding
- **Scripting support** - `--dry-run`, `--estimate`, `--output-format json`

## Installation

### From crates.io

```bash
cargo install susurs
```

### From source

```bash
git clone https://github.com/atareao/susurs.git
cd susurs
cargo build --release
# binary at target/release/susurs
```

### Requirements

- Rust edition 2024
- ~50MB disk space per downloaded TTS model

## Usage

### Basic

```bash
# Speak text directly
susurs --text "Hola, esto es una prueba de voz" -o output.mp3

# Read from file
susurs --input input.txt -o output.mp3

# Read from stdin
echo "Hola mundo" | susurs -o output.mp3

# Auto-detect format from extension
susurs --text "Hola" -o speech.wav    # outputs WAV
susurs --text "Hola" -o speech.mp3    # outputs MP3
```

### Presets

```bash
susurs --text "Hola" --preset fast          # fast speech
susurs --text "Hola" --preset slow          # slow speech
susurs --text "Hola" --preset human         # natural voice
susurs --text "Hola" --preset quiet --volume 2.0  # preset + override
```

### Advanced parameters

```bash
# Custom rate and pitch
susurs --text "Hola" --rate 1.5 --pitch 1.0 -o out.mp3

# MP3 quality
susurs --text "Hola" --bitrate 320 -o out.mp3

# Normalize and add silence
susurs --text "Hola" --normalize --silence 0.5 -o out.mp3

# Multi-speaker model
susurs --model supertonic-3-es --sid 1 --text "Hola" -o out.mp3
```

### Model management

```bash
# List available models
susurs --list-models

# Show model info
susurs --model-info vits-piper-es_AR-daniela-high

# Delete cached model
susurs --delete-model vits-piper-es_AR-daniela-high

# Update all cached models
susurs --update-models

# Clean entire model cache
susurs --clean-cache
```

### Scripting

```bash
# Dry run (no audio generated)
susurs --text "Hola" --dry-run

# Estimate duration
susurs --text "Hola" --estimate
susurs --text "Hola" --estimate --output-format json
```

## CLI Reference

| Flag | Description | Default |
|---|---|---|
| `-t, --text` | Text to synthesize | |
| `-i, --input` | Read text from file | |
| `-o, --output` | Output file path | `output.mp3` |
| `--stdout` | Write audio to stdout | |
| `--model` | TTS model name | `vits-piper-es_AR-daniela-high` |
| `--rate` | Speaking rate (0.5-2.0) | `1.0` |
| `--pitch` | Voice pitch (0.2-1.5) | `0.667` |
| `--volume` | Volume scaling (0.5-2.0) | `1.0` |
| `--noise-scale-w` | Prosody variation | `0.8` |
| `--length-scale` | Tempo control | `1.0` |
| `--silence-scale` | Pause duration | `0.2` |
| `--sid` | Speaker ID (multi-speaker models) | `0` |
| `--preset` | Preset: human, fast, slow, quiet, loud | |
| `--format` | Output format: mp3, wav, auto | `auto` |
| `--bitrate` | MP3 bitrate (128/192/320) | `128` |
| `--sample-rate` | Override output sample rate | model default |
| `--normalize` | Normalize audio volume | off |
| `--silence` | Add silence at start/end (seconds) | `0` |
| `--dry-run` | Show what would be done | |
| `--estimate` | Estimate audio duration | |
| `--output-format` | Output format: text, json | `text` |

## Available Models

| Model | Language | Gender | Quality |
|---|---|---|---|
| `vits-piper-es_AR-daniela-high` | Argentine Spanish | Female | High (default) |
| `vits-piper-es_ES-carlfm-x_low` | Spanish (Spain) | Male | Low |
| `vits-piper-es_ES-davefx-medium` | Spanish (Spain) | Male | Medium |
| `vits-piper-es_ES-glados-medium` | Spanish (Spain) | Female | Medium |
| `vits-piper-es_ES-miro-high` | Spanish (Spain) | Male | High |
| `vits-piper-es_ES-sharvard-medium` | Spanish (Spain) | Male | Medium |
| `vits-piper-es_MX-ald-medium` | Mexican Spanish | Male | Medium |
| `vits-piper-es_MX-claude-high` | Mexican Spanish | Male | High |
| `supertonic-3-es` | Spanish | Multi-speaker | High |

Models are automatically downloaded from [k2-fsa/sherpa-onnx releases](https://github.com/k2-fsa/sherpa-onnx/releases/tag/tts-models) on first use and cached in `~/.local/share/tts-models/`.

## Examples

```bash
# Generate MP3 with fast preset, high quality
susurs --text "Bienvenido al sistema" --preset fast --bitrate 320 -o welcome.mp3

# Generate WAV for further processing
susurs --input script.txt --format wav --normalize -o script.wav

# Batch convert (using a loop)
for f in *.txt; do susurs --input "$f" -o "${f%.txt}.mp3"; done

# Estimate before generating
susurs --input long_text.txt --estimate --output-format json
```

## References

- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) - ONNX speech toolkit
- [Piper TTS](https://github.com/rhasspy/piper) - Neural TTS models
- [shine-rs](https://github.com/wshon/shine-rs) - Pure-Rust MP3 encoder
