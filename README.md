<h1 align="center">
    <span>FBN Lofi Engine</span>
    <img height="30" src="assets/music-icon.svg" alt="Fantasy Broadcast Network logo"/>
</h1>

<p align="center">
    Generate fantasy-themed lofi music using natural language prompts and the MusicGen model. Extended with infinite music generation support.
</p>

# Overview

FBN Lofi Engine is a fork of [MusicGPT](https://github.com/gabotechs/MusicGPT) adapted specifically for fantasy/ambient lofi generation. It supports:

- Text-conditioned music generation with fantasy narrative prompts
- Seamless chunk-based generation to produce tracks beyond 30 seconds
- Export to `.wav` without playback
- Integration-ready for automation pipelines (e.g. `n8n`, `cron`, etc)

# Features
- [x] Fantasy-themed prompt input
- [x] Extended duration generation via chunk stitching
- [x] Command-line interface for automation
- [x] Offline-compatible (no network requirement during use)

# Build & Install

You must have the [Rust toolchain](https://www.rust-lang.org/tools/install) installed.

```bash
cargo build --release
```

Binaries will be available under `target/release/fbnlofi`

# Usage

You can generate music using the command line:

```bash
fbnlofi "elven forest ambient" --secs 180 --model large --output forest_song.wav
```

## Extended Generation (Infinite Mode)

Use `--infinite` to generate longer tracks by stitching together overlapping chunks.

```bash
fbnlofi "mystical tavern tune" --secs 240 --infinite --chunksize 30 --overlap 8 --model large --output tavern.wav
```

### Parameters
- `--secs`: total desired track length (in seconds)
- `--infinite`: enable infinite mode (chunk stitching)
- `--chunksize`: length of each chunk (default: 30s, max: 30)
- `--overlap`: overlap duration in seconds between chunks (to smooth transitions)
- `--model`: one of `small`, `medium`, or `large`
- `--output`: filename to save audio
- `--no-interactive`: disables UI mode
- `--no-playback`: disables audio playback (useful for headless environments)

## Example for Automation

```bash
fbnlofi "whispers of an ancient ruin" \
  --secs 200 \
  --infinite \
  --chunksize 25 \
  --overlap 6 \
  --model medium \
  --output ancient_ruin.wav \
  --no-interactive \
  --no-playback
```

# Storage

Model files are cached locally under:
- Linux: `~/.cache/fbnlofi`
- macOS: `~/Library/Caches/fbnlofi`
- Windows: `%LOCALAPPDATA%\fbnlofi`

# License

FBN Lofi Engine is based on MusicGPT and shares the same licenses:

- Code: [MIT License](./LICENSE)
- Model Weights: [CC-BY-NC-4.0 License](https://spdx.org/licenses/CC-BY-NC-4.0)

Models from:
- https://huggingface.co/facebook/musicgen-small
- https://huggingface.co/facebook/musicgen-medium
- https://huggingface.co/facebook/musicgen-large
