# Development Workflow Guide

## Quick Reference: Which Command to Use?

### Frontend Development (FAST - No Rust Rebuild)
**Use this 90% of the time for UI/React/TypeScript work**

```bash
npm run dev:frontend
# or just
npm run dev
```

- Hot reload for React/TypeScript changes
- No Rust compilation needed
- Rebuilds in < 1 second
- Perfect for: UI work, styling, React components, TypeScript logic

**Note:** Tauri API calls will fail in this mode. Mock them or wait until backend integration phase.

### Full Stack Development (SLOW - Includes Rust)
**Use when you need to test Rust backend integration**

```bash
npm run dev:tauri
```

- Compiles Rust backend + runs frontend
- First build: 2-5 minutes (with new optimizations)
- Subsequent builds: 30 seconds - 2 minutes (incremental)
- Use for: Testing Tauri commands, database operations, GitHub API calls

### Backend Only (Rust Development)
**Use when working on Rust code without UI**

```bash
npm run dev:backend
# or
cd src-tauri && cargo build
```

- Check Rust compilation without starting the app
- Good for: Verifying Rust changes, running tests, type checking

### Check Rust Without Building
**Fastest way to verify Rust code compiles**

```bash
cd src-tauri
cargo check
```

- 5-10x faster than full build
- Only type-checks, doesn't produce binary
- Use while writing Rust code

## Build Optimizations Applied

### 1. Cargo Profile Optimization (src-tauri/Cargo.toml)
- Your code: Light optimization (opt-level = 1) for fast compilation
- Dependencies: Heavy optimization (opt-level = 3) for fast runtime
- Result: Dependencies build slower once, but your code rebuilds quickly

### 2. Incremental Compilation (src-tauri/.cargo/config.toml)
- Caches intermediate compilation results
- Subsequent builds only recompile changed code
- Reduces rebuild time by 50-80%

### 3. Faster Linking
- Uses optimized linker settings for Windows
- Reduces link time (final step of compilation)

## Recommended Workflow

1. **Start with frontend mode** for UI work:
   ```bash
   npm run dev:frontend
   ```

2. **Switch to Tauri mode** when you need backend integration:
   ```bash
   npm run dev:tauri
   ```

3. **Clean rebuild** if things get weird:
   ```bash
   cd src-tauri
   cargo clean
   cd ..
   npm run dev:tauri
   ```

## Expected Build Times

### First Build (Cold Start)
- **Before optimizations:** 5-10 minutes
- **After optimizations:** 2-5 minutes

### Incremental Build (After Changes)
- **Small Rust change:** 30 seconds - 1 minute
- **TypeScript only:** < 1 second (in frontend mode)
- **Multiple Rust files:** 1-2 minutes

### Type Check Only
- **cargo check:** 10-30 seconds

## Pro Tips

1. Keep `npm run dev:frontend` running while working on UI
2. Only rebuild Rust when absolutely necessary
3. Use `cargo check` in src-tauri/ for quick Rust validation
4. Clean build if incremental compilation acts weird: `cd src-tauri && cargo clean`
5. First build after optimizations will be slow (building optimized dependencies), but subsequent builds will be much faster

## Next Steps for More Speed

If you want even faster builds, consider:
- **Cargo workspace:** Split Rust into separate crates (30 min setup)
- **Feature flags:** Make embeddings optional (saves ~1 min per build)
- **Mold linker:** Even faster linking on Linux/Mac
- **cargo-watch:** Auto-rebuild only changed files
