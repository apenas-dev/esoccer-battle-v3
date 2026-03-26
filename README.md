# E-Soccer Battle V3

Tauri v2 desktop app — mic capture, Whisper transcription, audio playback.

## Stack

- **Backend**: Rust + Tauri v2
- **Transcription**: whisper-rs (local, no API)
- **Audio**: cpal (capture), rodio (playback)
- **Frontend**: React

## 🚀 Quick Start (Windows)

### 1. Clone & Setup (one command)

```powershell
git clone <repo-url>
cd esoccer-battle-v3
```

Right-click `setup.ps1` → **Run with PowerShell** (Admin):

```powershell
npm run setup
```

This installs automatically:
- ✅ Rust (stable)
- ✅ Node.js 22
- ✅ Visual Studio Build Tools (C++ workload) — needed for whisper-rs
- ✅ CMake
- ✅ npm dependencies

### 2. Run

```powershell
npm run tauri dev
```

> ⏳ **First build takes 5-10 minutes** — it compiles the Whisper C++ engine from source. Subsequent builds are fast.

## Requirements (manual, if you prefer)

If you already have these, skip `setup.ps1`:

| Tool | Min Version | Install |
|------|------------|---------|
| Rust | stable | <https://rustup.rs> |
| Node.js | 22+ | <https://nodejs.org> |
| VS Build Tools | C++ workload | <https://visualstudio.microsoft.com/visual-cpp-build-tools/> |
| CMake | 3.20+ | <https://cmake.org/download/> |

## Dev

```bash
# Frontend only
npm run dev

# Full Tauri app (Rust + frontend)
npm run tauri dev

# Rust check only
cd src-tauri && cargo check
```

## Download Whisper Models

The app downloads models at runtime via Settings. Available models:

| Model | Size | RAM | Speed |
|-------|------|-----|-------|
| Tiny | 77 MB | ~390 MB | ⚡ Fastest |
| Tiny (Quantized) | 33 MB | ~390 MB | ⚡⚡ Fastest |
| Base | 148 MB | ~500 MB | 🔥 Best balance |
| Base (Quantized) | 60 MB | ~500 MB | 🔥 Good balance |

## Troubleshooting

### 72 C++ compilation errors (whisper-rs)

You're missing Visual Studio Build Tools with the **C++ workload**:

1. Download: <https://visualstudio.microsoft.com/visual-cpp-build-tools/>
2. Install with **"Desktop development with C++"** workload
3. Reopen terminal and run `cargo clean && npm run tauri dev`

### CMake not found

```powershell
winget install Kitware.CMake
```

### Rust not found

```powershell
winget install Rustlang.Rustup
```

## License

MIT
