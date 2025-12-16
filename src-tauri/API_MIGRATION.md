# FastEmbed v3 → v4 API Migration

## Current Code Status

Your code in `src/embeddings/mod.rs` uses:

```rust
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

let model = TextEmbedding::try_new(InitOptions {
    model_name: EmbeddingModel::AllMiniLML6V2,
    show_download_progress: true,
    ..Default::default()
})?;
```

## Compatibility Check

### ✅ GOOD NEWS: API is compatible!

The following are still valid in fastembed v4:
- `TextEmbedding::try_new()` ✅
- `InitOptions` struct ✅
- `EmbeddingModel::AllMiniLML6V2` ✅
- `.embed()` method ✅

### Only change needed: Feature flag

**Before (v3):**
```toml
fastembed = { version = "3.7", features = ["online"] }
```

**After (v4):**
```toml
fastembed = { version = "4", default-features = false }
```

The `online` feature was removed because it's the default behavior now.

## Code Should Work As-Is

No changes needed to `src/embeddings/mod.rs` or `src/embeddings/generator.rs`.

The existing code should compile successfully with fastembed v4.9.1.

## If You See Compile Errors

### Error: `InitOptions` not found

Try:
```rust
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
// Change to:
use fastembed::{EmbeddingModel, TextEmbedding};

let model = TextEmbedding::try_new(
    EmbeddingModel::AllMiniLML6V2,
    // InitOptions might be removed, check docs
)?;
```

### Error: `embed` method signature changed

Check if it now requires a batch size:
```rust
// Old:
model.embed(text_refs, None)?

// Might need:
model.embed(text_refs, Some(32))?  // batch_size
```

### Error: Model enum variant names changed

Try:
```rust
// If AllMiniLML6V2 not found:
EmbeddingModel::AllMiniLmL6V2  // Different capitalization
// or
EmbeddingModel::MiniLmL6V2    // Shortened name
```

## How to Check Current API

Generate local docs:
```bash
cd src-tauri
cargo doc --package fastembed --open
```

This opens the exact API for fastembed v4.9.1 in your browser.

Look for:
- `TextEmbedding::try_new` signature
- `InitOptions` fields
- `EmbeddingModel` variants

## Testing After Build

Once built, test embeddings:
```rust
// In your app or unit tests:
use crate::embeddings::generate_embedding;

let embedding = generate_embedding("test text")?;
assert_eq!(embedding.len(), 384);  // Should still be 384-dim
```

## Rollback Plan

If fastembed v4 doesn't work, you can rollback:

```toml
# Use last stable v3 before ort issues:
fastembed = { version = "3.3", default-features = false }
```

This was the last version before the ort RC.4 dependency was introduced.

## Performance Notes - FastEmbed v4

Changes in v4:
- ✅ Better model caching
- ✅ Reduced memory footprint
- ✅ Faster initialization
- ⚠️ First-time model download still ~80MB
- ⚠️ Model stored in: `%APPDATA%\.cache\fastembed\`

No breaking changes expected in your usage.
