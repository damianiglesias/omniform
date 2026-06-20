# Omniform
<p align="center">
  <img src="assets/logo.png" alt="Omniform" width="120"/>
</p>

<h1 align="center">Omniform</h1>

> Convert video and audio from any platform to any format.

A cross-platform desktop application for downloading and converting content
from YouTube, TikTok, Instagram and other sources into common video and
audio formats (MP4, WebM, MP3, WAV, FLAC, M4A, OGG).

## Features

- Native desktop interface (Windows, macOS, Linux)
- Download queue with real-time progress
- Output format and quality selection, including audio bitrate
- Automatic download and setup of the required components on first launch —
  no manual configuration needed

## How it works

Omniform is a graphical interface and queue manager built on top of two
well-established command-line tools from the media download and conversion
ecosystem. The app downloads them automatically the first time it runs,
into its own application data folder, without touching any system-wide
installation.

## Development

### Requirements

- [Node.js](https://nodejs.org/) 18 or later
- [Rust](https://www.rust-lang.org/tools/install) (via rustup)
- System dependencies for Tauri depending on your platform — see the
  [official prerequisites guide](https://tauri.app/start/prerequisites/)

### Run in development mode

```bash
npm install
npm run tauri dev
```

### Build for production

```bash
npm run tauri build
```

The resulting installer is generated under
`src-tauri/target/release/bundle/`.

## Usage notice

This tool is intended for downloading content you own or that is licensed
for download. Using it to download third-party content without
authorization may violate the terms of service of the source platforms
and, depending on your jurisdiction, intellectual property law. Use of
this tool is the sole responsibility of the person using it.

## License

GPL-3.0 — see [LICENSE](LICENSE)
