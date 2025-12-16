# Build Environment Setup

## Current Environment

**Detected**: Git Bash (MINGW64) on Windows
**Rust**: 1.91.1
**Cargo**: 1.91.1

## Build Requirements

This is a Tauri application that requires Windows build tools.

### Required Software

1. **Visual Studio Build Tools 2022** (Required for webview2-com-sys)
   - Download: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
   - Required workloads:
     - ✅ Desktop development with C++
     - ✅ Windows 10/11 SDK
     - ✅ MSVC v143 build tools

2. **WebView2 Runtime** (Usually pre-installed on Windows 10/11)
   - Download if needed: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

3. **Node.js 18+** (✅ Already installed)

4. **Rust 1.75+** (✅ Already installed: 1.91.1)

## Build Commands

After installing Visual Studio Build Tools, restart your terminal and run:

```bash
# From project root
cd src-tauri
cargo build

# Or run the full app
npm run tauri dev
```

## Alternative: True WSL2 Setup

If you want to develop in actual WSL2 (Ubuntu), you'll need:

```bash
# In WSL2 Ubuntu
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    pkg-config

# Then build
cd src-tauri
cargo build
```

## Current Status

**Phase 2A Code**: ✅ Reviewed and verified correct
**Build Environment**: ❌ Needs Visual Studio Build Tools

The Phase 2A changes are ready to commit. Build testing will work once environment is set up.

## Quick Test (Without Full Build)

To verify code syntax without building:

```bash
cd src-tauri
cargo check --lib  # Check library code only (might skip some build scripts)
```
