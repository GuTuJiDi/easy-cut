<p align="center">
  <h1 align="center">✂️ EasyCut</h1>
  <p align="center">
    <strong>Lossless video processing desktop app with intelligent marker annotation</strong>
  </p>
  <p align="center">
    Built with Tauri 2 · Vue 3 · Rust · FFmpeg
  </p>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/Tauri-v2-FFC131?logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/Vue-3.5-42B883?logo=vue.js" alt="Vue 3.5">
  <img src="https://img.shields.io/badge/Rust-2021-DEA584?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/FFmpeg-stream_copy-007808?logo=ffmpeg" alt="FFmpeg Stream Copy">
</p>

---

## ✨ Features

### Free

| Feature | Description |
|---------|-------------|
| **Lossless Duration Splitting** | Split video by fixed duration using FFmpeg stream copy — zero re-encoding, near-instant results |
| **Manual Marker Annotation** | Create in/out point markers on video timelines with full undo/redo history |
| **Marker File Management** | CRUD for marker projects: search, filter, sort, soft-delete with undo, trash restore |
| **Floating Sync Widget** | Transparent always-on-top stopwatch overlay for "blind marking" with external players |
| **Global Settings** | Workspace path, auto-save, performance concurrency, machine code display |

### PRO

| Feature | Description |
|---------|-------------|
| **Fixed-Count Splitting** | Split video into N equal segments |
| **Audio Extraction** | FFprobe-based smart codec detection, lossless audio track extraction (.m4a/.ac3/.flac etc.) |
| **Video Format Conversion** | Lossless container conversion (MP4/MKV/MOV/AVI) via stream copy |
| **Media Track Inspector** | Probe all video/audio/subtitle tracks with codec, resolution, FPS, language metadata |
| **Browser Extension Pairing** | Secure 8-char pairing code to connect Chrome extension without exposing PRO token |
| **AI Marker Recognition** | AI-powered automatic marker generation (planned: Whisper integration) |

### Cross-Cutting

- **6-Type Semantic Export** — Each marker supports: `master`, `no_subs`, `pure_video`, `iso_track`, `audio_only`, `subs_only`, with automatic subdirectory routing
- **Trash/Undo System** — Soft-delete with configurable retention, 10-second undo toast
- **Chrome Extension** — Manifest V3 extension for immersive in-browser marking on Bilibili, YouTube, Douyin, and more

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Frontend (Vue 3)                  │
│  Pinia Stores · Vue Router · 9 Composables          │
│  Module Registry Pattern · Lazy-loaded Views         │
├─────────────────────────────────────────────────────┤
│                  Tauri IPC Bridge                    │
├──────────────────┬──────────────────────────────────┤
│   media/         │  security/       │  storage/     │
│  · FFmpeg engine │  · RSA license   │  · Workspace  │
│  · Task planner  │  · Machine ID    │  · Markers    │
│  · Async runtime │  · Session token │  · Settings   │
│  · Cancellation  │  · Rate limiter  │  · Trash      │
├──────────────────┴──────────────────────────────────┤
│              Local Axum HTTP Hub (:random)           │
│           Browser Extension Communication            │
├─────────────────────────────────────────────────────┤
│             Chrome Extension (Manifest V3)           │
│  Content Script · Background Worker · Popup          │
└─────────────────────────────────────────────────────┘
```

**Key patterns:**
- **Module Registry** — Single source of truth in `config/modules.ts` generates routes and sidebar automatically
- **Zero-Trust Security** — Every PRO command validates both session token AND Rust-side `is_pro` boolean
- **Cooperative Cancellation** — `tokio::select!` + `broadcast::channel` for clean FFmpeg task abortion
- **Plugin Token Isolation** — Browser extensions get minimal-scope `plugin_*` tokens, never the PRO token

---

## 📁 Project Structure

```
easy-cut/
├── src/                          # Vue 3 frontend
│   ├── views/                    # 7 page views (Split, Marker, Audio, etc.)
│   ├── components/               # UI components + file-manager/ + marker/ subdirs
│   ├── composables/              # 9 composables (auto-save, drag, export, history...)
│   ├── stores/                   # Pinia stores (auth, settings, workspace)
│   ├── config/modules.ts         # Module registry (routes + sidebar generation)
│   ├── types/                    # TypeScript type definitions
│   └── utils/                    # Formatters, logger, security guard
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── main.rs               # App entry, Axum hub, command registration
│       ├── media/                # FFmpeg engine, task planning, workflow orchestration
│       ├── security/             # DRM, license, hardware fingerprint, widget manager
│       └── storage/              # Workspace, markers, settings, trash management
├── chrome-extension/             # Manifest V3 browser extension
│   ├── content.js                # Video detection + in-page marker panel
│   ├── background.js             # Hub communication + pairing flow
│   └── popup.js                  # Extension popup UI
└── docs/                         # Design docs and roadmap
```

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) — `npm install -g pnpm`
- [Rust](https://www.rust-lang.org/tools/install) (edition 2021)
- [FFmpeg](https://ffmpeg.org/) + FFprobe — place binaries in `src-tauri/bin/` (production) or add to system PATH (development)

### Install & Run

```bash
# Clone the repository
git clone https://github.com/JerryZ00/easy-cut.git
cd easy-cut

# Install frontend dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Browser Extension

1. Open `chrome://extensions/` in Chrome/Edge
2. Enable **Developer mode**
3. Click **Load unpacked** and select the `chrome-extension/` directory
4. Pair with EasyCut via the 8-character code in Settings → Plugin Pairing

---

## 🔒 Security

EasyCut implements a defense-in-depth security model:

| Layer | Mechanism |
|-------|-----------|
| **License Verification** | RSA-2048 signature with compile-time obfuscated public key (`obfstr!`) |
| **Machine Fingerprint** | 3-component hardware ID (CPU, hostname, UUID) with 2-of-3 fuzzy matching |
| **Credential Storage** | OS keyring (Windows Credential Manager / macOS Keychain) via `keyring` crate |
| **Binary Integrity** | SHA-256 hash verification of FFmpeg/FFprobe on every invocation |
| **Frontend Obfuscation** | Control flow flattening, dead code injection, RC4 string encoding |
| **Session Isolation** | UUID v4 session tokens; plugin tokens with minimal permissions |
| **Brute-Force Protection** | 5 failed attempts → 60-second lockout |

---

## 🗺️ Roadmap

- [x] Phase 1: DRM licensing, async concurrent engine, frontend obfuscation
- [x] Phase 2: Audio extraction, format conversion, PRO feature gating
- [ ] Phase 3: macOS native build, iPad exploration via Tauri v2 mobile
- [ ] Phase 4: AI-powered markers (Whisper integration), cross-source marking network

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full roadmap (Chinese).

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|-----------|
| **Desktop Runtime** | [Tauri v2](https://v2.tauri.app/) |
| **Frontend** | [Vue 3.5](https://vuejs.org/) · [TypeScript](https://www.typescriptlang.org/) · [Vite 6](https://vitejs.dev/) · [Pinia 3](https://pinia.vuejs.org/) |
| **Backend** | [Rust](https://www.rust-lang.org/) (edition 2021) · [Tokio](https://tokio.rs/) · [Axum 0.7](https://github.com/tokio-rs/axum) |
| **Video Processing** | [FFmpeg](https://ffmpeg.org/) stream copy (`-c copy`) — no re-encoding |
| **Security** | RSA-2048 · SHA-256 · OS Keyring · Compile-time string obfuscation |
| **Browser Extension** | Chrome Manifest V3 |

---

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure your code follows the existing style and includes appropriate tests.

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) — for the lightweight, secure desktop framework
- [FFmpeg](https://ffmpeg.org/) — for the powerful multimedia processing engine
- [Vue.js](https://vuejs.org/) — for the progressive JavaScript framework
- [Rust](https://www.rust-lang.org/) — for memory safety and performance
