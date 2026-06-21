<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Omniform" width="120"/>
</p>

<h1 align="center">Omniform</h1>

<p align="center">
  Convert video and audio from any platform to any format.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-d97b3f"/>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-d97b3f"/>
  <img src="https://img.shields.io/badge/license-GPL--3.0-d97b3f"/>
</p>

---

## What is Omniform?

Omniform is a free, open-source desktop application that downloads video
and audio from YouTube, TikTok, Instagram and many other platforms, and
converts it to the format you actually need — MP4, WebM, MP3, WAV, FLAC,
M4A or OGG.

No browser extensions, no sketchy websites, no ads. Just paste a link,
pick a format, and get your file.

Think of it as a graphical alternative to using `yt-dlp` from the command
line — same underlying capability, but with a proper interface, a queue,
and zero setup.

## Features

- **Any platform, one app** — paste a link and Omniform figures out the rest
- **Any format you need** — video (MP4, WebM) or audio (MP3, WAV, FLAC,
  M4A, OGG), with selectable quality
- **Real download queue** — add several links, track each one's progress,
  speed and ETA independently
- **Zero setup** — required components are downloaded automatically the
  first time you open the app
- **No terminal, no command line** — a real desktop app with a real
  interface, nothing runs in a console window
- **Free and open source** — GPL-3.0, no telemetry, no accounts

## Installation

Download the latest installer for your platform from the
[Releases page](../../releases).

- **Windows**: run the `.exe` installer
- **macOS / Linux**: see the Releases page for available builds

No additional setup is required — Omniform downloads everything else it
needs on first launch.

## Usage

1. Open Omniform
2. Choose an output folder
3. Paste a video URL
4. Pick a format and quality
5. Click **Add to queue**

Watch the progress bar fill up, and find your file in the folder you
selected once it's done.

## Building from source

If you'd rather build it yourself instead of using the installer, see
[SETUP.md](SETUP.md) for the full development setup, requirements, and
build instructions.

```bash
npm install
npm run tauri dev      # development mode
npm run tauri build    # production installer
```

## How it works

Omniform is a desktop interface and queue manager built on top of two
well-established, actively maintained open-source tools from the media
download and conversion ecosystem — one used to fetch the content, and a
second one used to handle format conversion. Omniform downloads both
automatically into its own application data folder on first run, without
touching anything on your system.

## A note on responsible use

Omniform is built for downloading content you own or that is licensed for
download — your own uploads, content under a permissive license, or media
you otherwise have the right to save. Using it to download copyrighted
third-party content without authorization may violate the terms of
service of the source platform and, depending on where you live,
copyright law. What you do with this tool is entirely your own
responsibility.

## License

This project is licensed under the **GNU General Public License v3.0** —
see [LICENSE](LICENSE) for details.

## Contributing

Issues and pull requests are welcome. If you run into a bug or have an
idea for a feature, open an issue.
