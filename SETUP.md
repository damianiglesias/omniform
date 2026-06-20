# Getting Omniform running

## 1. Install requirements

- **Node.js 18+**: https://nodejs.org
- **Rust** (via rustup): https://rustup.rs
- **Tauri system prerequisites** for your OS — follow the official guide:
  https://tauri.app/start/prerequisites/

## 2. Install project dependencies

From the project root (where `package.json` lives):

```bash
npm install
```

## 3. Run in development mode

```bash
npm run tauri dev
```

This opens the app window with hot-reload. On first run, the app will try
to automatically download `yt-dlp` and `ffmpeg` into its data folder — it
needs an internet connection the first time.

## 4. Generate the real app icon (before building for production)

The icons included in `src-tauri/icons/` are a minimal placeholder (a flat
color square) only meant to let the project compile without errors. Before
shipping a real build:

1. Prepare a square image, at least 1024x1024px (PNG, transparent
   background recommended) with the final logo.
2. Run:
   ```bash
   npm run tauri icon path/to/your/logo.png
   ```
   This automatically generates every required format (`.ico` for Windows,
   `.icns` for macOS, and PNGs at several resolutions) inside
   `src-tauri/icons/`.
3. Edit `src-tauri/tauri.conf.json` and add back to the `icon` list:
   ```json
   "icons/icon.ico",
   "icons/icon.icns"
   ```
   (step 2 already generates them in that folder).

## 5. Build for production

```bash
npm run tauri build
```

The final installer lands in `src-tauri/target/release/bundle/`
(`.msi`/`.exe` on Windows, `.dmg`/`.app` on macOS, `.deb`/`.AppImage` on
Linux, depending on where you build).

## Notes on the automatic download URLs

`src-tauri/src/dependencies.rs` holds the URLs the app downloads `yt-dlp`
and `ffmpeg` from on first run. `yt-dlp` always points to the latest
standalone binary from its official GitHub releases. For `ffmpeg`, since
there is no single official standalone binary per platform, the app relies
on widely used static builds from the community:

- Windows: builds from gyan.dev
- macOS: builds from evermeet.cx
- Linux: static builds from johnvansickle.com

If you ever want to change these sources (for example, vendoring the
binaries directly into the repo instead of downloading them), that's the
only file you need to touch.
