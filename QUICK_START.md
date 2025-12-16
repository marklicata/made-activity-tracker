# Quick Start - Building and Running the App

## TL;DR - The Fix

**Problem:** Build fails with `ort` (ONNX Runtime) compilation errors  
**Solution:** Updated `fastembed` dependency in `Cargo.toml` to use compatible version  
**Status:** Ready to build (will take 15-20 min on first run)

---

## Understanding the Build Process

### Two separate build steps:

1. **Frontend Build** (`npm run build` in root or `src-tauri/`)
   - Compiles TypeScript → JavaScript
   - Bundles with Vite
   - Fast (~30 seconds)
   - ✅ This was working fine

2. **Rust Backend Build** (happens during `npm run tauri dev`)
   - Compiles 600+ Rust crates (dependencies)
   - Includes ONNX Runtime (machine learning)
   - Slow on first build (10-20 minutes)
   - ❌ This was failing

---

## What Was Wrong

The `fastembed` crate (used for semantic search embeddings) was pulling in:
```
ort v2.0.0-rc.4  ← Release candidate with breaking changes
```

This RC version has type system incompatibilities causing errors like:
- `E0599: no method named 'expect' found`
- `E0282: type annotations needed`
- `E0308: mismatched types`

---

## What I Fixed

Updated `src-tauri/Cargo.toml`:

```toml
# Before (broken):
fastembed = { version = "3.7", default-features = false, features = ["online"] }

# After (fixed):
fastembed = { version = "4", default-features = false }
```

This now pulls in `ort v2.0.0-rc.9` which is more stable.

---

## How to Build Now

### Option 1: Quick Test (recommended first)

Run the test script:
```cmd
test_build.cmd
```

This will:
- Check your Rust version
- Verify dependencies resolve correctly
- Attempt a full build
- Show clear success/failure message

### Option 2: Manual Build

```bash
cd src-tauri
cargo clean          # Clear old artifacts
cargo build          # Build in debug mode (faster)
```

**Expected behavior:**
- You'll see "Compiling ..." for 600+ crates
- This is NORMAL and takes 10-20 minutes
- Watch for actual ERROR messages (red text)
- "warning:" messages are usually okay

### Option 3: Skip to Dev Mode

```bash
npm run tauri dev
```

This builds AND runs the app. Same long compile time on first run.

---

## Progress Indicators

During build, you'll see:
```
Compiling proc-macro2 v1.0.76
Compiling unicode-ident v1.0.12
Compiling libc v0.2.153
...
(598 more crates)
...
Compiling made-activity-tracker v0.1.0
```

**This is normal!** Each crate only compiles once, then gets cached.

Subsequent builds are MUCH faster (30-60 seconds).

---

## If Build Still Fails

### Check 1: Verify the fix was applied
```bash
cd src-tauri
findstr /C:"fastembed" Cargo.toml
```

Should show:
```toml
fastembed = { version = "4", default-features = false }
```

### Check 2: Verify dependency versions
```bash
cargo tree -i ort
```

Should show `ort v2.0.0-rc.9` or higher (not rc.4).

### Check 3: Platform-specific dependencies

**Windows:**
- Need Visual Studio Build Tools with C++ workload
- See: `TROUBLESHOOTING.md` section on `webview2-com-sys`

**Linux:**
```bash
sudo apt install libssl-dev pkg-config
```

**macOS:**
```bash
xcode-select --install
```

### Check 4: Rust version
```bash
rustc --version  # Should be 1.75+
rustup update stable
```

---

## Alternative: Disable Embeddings Temporarily

If you want to develop WITHOUT the semantic search feature while debugging:

1. Comment out fastembed in `src-tauri/Cargo.toml`:
   ```toml
   # fastembed = { version = "4", default-features = false }
   ```

2. Stub out the embeddings module in `src-tauri/src/embeddings/mod.rs`:
   ```rust
   pub mod generator;
   use anyhow::Result;
   
   pub fn generate_embeddings(_texts: &[String]) -> Result<Vec<Vec<f32>>> {
       Ok(vec![])  // Disabled
   }
   
   pub fn generate_embedding(_text: &str) -> Result<Vec<f32>> {
       Ok(vec![])  // Disabled
   }
   ```

3. Build should complete in ~5 minutes instead of 15-20 minutes

You can add embeddings back later when the ecosystem stabilizes.

---

## Success Indicators

Build succeeded if you see:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXm
```

Then the app window should launch automatically with `npm run tauri dev`.

---

## Performance Tips

### Speed up builds:

1. **Use debug builds during development:**
   ```bash
   cargo build          # Fast, large binary
   ```

2. **Only use release for production:**
   ```bash
   cargo build --release   # Slow, optimized binary
   ```

3. **Incremental compilation (default in dev):**
   - First build: 15-20 min
   - Subsequent: 30-60 sec (only changed code recompiles)

4. **Parallel jobs (if you have 8+ CPU cores):**
   ```bash
   cargo build -j 8
   ```

---

## Next Steps After Successful Build

1. **Run the app:**
   ```bash
   npm run tauri dev
   ```

2. **Check functionality:**
   - OAuth login should work
   - Database should initialize
   - Frontend should load
   - ⚠️ Embeddings will try to download model (~80MB) on first use

3. **Development workflow:**
   - Edit TypeScript files → hot reload in ~1s
   - Edit Rust files → incremental rebuild in ~30s

---

## Getting Help

Still stuck? Check:
- `BUILD_FIX.md` - Detailed technical explanation
- `TROUBLESHOOTING.md` - Platform-specific issues
- Build logs: `src-tauri/target/debug/build.log`

Or run with verbose logging:
```bash
RUST_LOG=debug npm run tauri dev
```
