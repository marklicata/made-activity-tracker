# Troubleshooting Guide

## Windows Build Error: `webview2-com-sys` failure

### Error Message
```
error: failed to run custom build command for `webview2-com-sys v0.19.0`
Error: Io(Error { kind: NotFound, message: "program not found" })
```

### Root Cause
Missing Windows build tools required by Tauri on Windows.

### Solution

#### Option 1: Install Visual Studio Build Tools (Recommended)

1. Download **Visual Studio Build Tools 2022**: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

2. Run the installer and select:
   - ✅ **Desktop development with C++**
   - ✅ **Windows 10/11 SDK**
   - ✅ **MSVC v143 build tools**

3. Restart your terminal after installation

4. Try building again:
   ```bash
   npm run tauri dev
   ```

#### Option 2: Install via Chocolatey (Faster)

If you have Chocolatey package manager:

```powershell
# Run as Administrator
choco install visualstudio2022buildtools
choco install visualstudio2022-workload-vctools
```

#### Option 3: Update Tauri (Sometimes fixes it)

```bash
npm update @tauri-apps/cli @tauri-apps/api
cd src-tauri
cargo update
```

---

## WebView2 Runtime Missing

### Error Message
```
WebView2 runtime not found
```

### Solution

Download and install WebView2 Runtime:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section

---

## Rate Limit Errors from GitHub

### Error Message
```
API rate limit exceeded
```

### Solution

1. Make sure you're authenticated (not using anonymous API calls)
2. Reduce the number of repos being synced
3. Increase the sync interval in Settings
4. Check current rate limit:
   ```
   curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/rate_limit
   ```

---

## FastEmbed Model Download Fails

### Error Message
```
Failed to download model
```

### Solution

1. Check internet connection
2. Model downloads from Hugging Face (~80MB)
3. If behind corporate firewall, set proxy:
   ```bash
   set HTTPS_PROXY=http://proxy.company.com:8080
   ```
4. Manual download location: `%APPDATA%\fastembed\models\`

---

## SQLite Database Locked

### Error Message
```
database is locked
```

### Solution

1. Close any other instances of the app
2. Delete lock file:
   ```
   %APPDATA%\made-activity-tracker\db\.sqlite-lock
   ```
3. Restart the app

---

## OAuth Device Flow Timeout

### Error Message
```
Device code expired
```

### Solution

1. Device codes expire after 15 minutes
2. Click "Sign in with GitHub" again to get a new code
3. Complete the flow within 15 minutes

---

## Frontend Won't Load / Blank Screen

### Solution

1. Check browser console (F12 in the app)
2. Clear the Tauri cache:
   ```
   %APPDATA%\made-activity-tracker\cache\
   ```
3. Rebuild:
   ```bash
   npm run build
   npm run tauri dev
   ```

---

## Getting Help

If none of these solutions work:

1. Check the logs:
   - Windows: `%APPDATA%\made-activity-tracker\logs\`
   - Or run with verbose logging: `RUST_LOG=debug npm run tauri dev`

2. Create an issue with:
   - Operating system and version
   - Error message (full output)
   - Steps to reproduce
   - Log files

---

## Quick Checks

Before reporting an issue, verify:

- [ ] Visual Studio Build Tools installed
- [ ] WebView2 Runtime installed
- [ ] GitHub OAuth Client ID configured in `src-tauri/src/github/commands.rs`
- [ ] Node.js version >= 18
- [ ] Rust version >= 1.75
- [ ] Internet connection working
