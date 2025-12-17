# GitHub CLI Fallback Implementation Status

## Overview
Implementing automatic fallback to GitHub CLI (`gh`) when GraphQL and REST API calls fail, particularly for SAML-protected repositories.

**Status:** 🟡 In Progress (70% complete)

---

## ✅ Completed Components

### 1. CLI Module (`src-tauri/src/github/cli.rs`)
**Status:** ✅ Complete and functional

**What it does:**
- Detects if `gh` CLI is installed and authenticated
- Provides async methods to fetch GitHub data using CLI commands
- Transforms CLI JSON responses into our existing data models

**Key Functions:**
```rust
pub struct GitHubCli {
    command_path: String,
    is_available: bool,
    is_authenticated: bool,
}

impl GitHubCli {
    pub async fn new() -> Result<Self>
    pub fn check_auth(&self) -> Result<bool>
    pub async fn fetch_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>>
    pub async fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>>
    pub async fn fetch_milestones(&self, owner: &str, repo: &str) -> Result<Vec<Milestone>>
    pub async fn fetch_pr_reviews(&self, owner: &str, repo: &str, pr_number: i32) -> Result<Vec<PrReview>>
}
```

**CLI Commands Used:**
- Issues: `gh issue list --repo owner/repo --state all --json [fields] --limit 1000`
- PRs: `gh pr list --repo owner/repo --state all --json [fields] --limit 1000`
- Milestones: `gh api repos/owner/repo/milestones`
- Reviews: `gh api repos/owner/repo/pulls/{number}/reviews`

**Error Handling:**
- Returns clear error if CLI not installed: "Install from: https://cli.github.com"
- Returns clear error if not authenticated: "Run: gh auth login"
- Provides detailed parsing errors if CLI output format changes

---

### 2. Module Export (`src-tauri/src/github/mod.rs`)
**Status:** ✅ Complete

**Change:**
```rust
pub mod auth;
pub mod cli;        // ← Added
pub mod commands;
pub mod graphql;
pub mod rest_api;
pub mod sync;
```

---

### 3. Sync Module Updates - Partial (`src-tauri/src/github/sync.rs`)
**Status:** 🟡 Partially complete

**What's Done:**
1. ✅ Added import: `use crate::github::cli::GitHubCli;`
2. ✅ Updated `sync_issues_rest_fallback()` to call CLI fallback on error (line ~605)

**What's Pending:**
1. ❌ Add three CLI fallback functions at end of file
2. ❌ Update `sync_pull_requests_rest_fallback()` to call CLI fallback
3. ❌ Update `sync_milestones_rest_fallback()` to call CLI fallback

---

## 🔄 Remaining Work

### Step 1: Add CLI Fallback Functions to sync.rs

Add these three functions at the END of `src-tauri/src/github/sync.rs` (before the final closing brace):

#### Function 1: `sync_issues_cli_fallback`

```rust
/// GitHub CLI fallback for syncing issues when both GraphQL and REST API fail
async fn sync_issues_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
    excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for issues in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "issues")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_issues(owner, name).await {
        Ok(mut issues) => {
            let mut total_synced = 0;

            for issue in &mut issues {
                issue.repo_id = repo_id;

                // Upsert issue (author and assignee IDs will be resolved later)
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_issue(
                    &conn,
                    issue.github_id,
                    repo_id,
                    issue.number,
                    &issue.title,
                    issue.body.as_deref(),
                    &issue.state,
                    issue.author_id,
                    issue.assignee_id,
                    issue.milestone_id,
                    &issue.created_at,
                    &issue.updated_at,
                    issue.closed_at.as_deref(),
                    &issue.labels,
                )?;

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} issues for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}
```

#### Function 2: `sync_pull_requests_cli_fallback`

```rust
/// GitHub CLI fallback for syncing pull requests when both GraphQL and REST API fail
async fn sync_pull_requests_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
    _excluded_bots: &[String],
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for PRs in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "pull_requests")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_pull_requests(owner, name).await {
        Ok(mut prs) => {
            let mut total_synced = 0;

            for pr in &mut prs {
                pr.repo_id = repo_id;

                // Upsert PR (author ID will be resolved later)
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_pull_request(
                    &conn,
                    pr.github_id,
                    repo_id,
                    pr.number,
                    &pr.title,
                    pr.body.as_deref(),
                    &pr.state,
                    pr.author_id,
                    &pr.created_at,
                    &pr.updated_at,
                    pr.merged_at.as_deref(),
                    pr.closed_at.as_deref(),
                    pr.additions,
                    pr.deletions,
                    pr.changed_files,
                    pr.review_comments,
                    &pr.labels,
                )?;

                total_synced += 1;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} PRs for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}
```

#### Function 3: `sync_milestones_cli_fallback`

```rust
/// GitHub CLI fallback for syncing milestones when both GraphQL and REST API fail
async fn sync_milestones_cli_fallback(
    state: &AppState,
    repo_id: i64,
    owner: &str,
    name: &str,
) -> Result<()> {
    tracing::info!("🔧 Using GitHub CLI fallback for milestones in {}/{}", owner, name);

    let log_id = {
        let conn = state.sqlite.lock().unwrap();
        queries::record_sync_start(&conn, repo_id, "milestones")?
    };

    // Initialize CLI client
    let cli = match GitHubCli::new().await {
        Ok(cli) => cli,
        Err(e) => {
            tracing::error!("❌ GitHub CLI not available: {}", e);
            tracing::warn!("   Install GitHub CLI from: https://cli.github.com");
            return Ok(()); // Don't fail the entire sync
        }
    };

    // Check if CLI is authenticated
    if let Err(e) = cli.check_auth() {
        tracing::error!("❌ GitHub CLI not authenticated: {}", e);
        tracing::warn!("   Run: gh auth login");
        return Ok(()); // Don't fail the entire sync
    }

    match cli.fetch_milestones(owner, name).await {
        Ok(milestones) => {
            let total_synced = milestones.len() as i32;

            for milestone in &milestones {
                let conn = state.sqlite.lock().unwrap();
                queries::upsert_milestone(
                    &conn,
                    milestone.github_id,
                    repo_id,
                    &milestone.title,
                    milestone.description.as_deref(),
                    &milestone.state,
                    milestone.due_on.as_deref(),
                    milestone.open_issues,
                    milestone.closed_issues,
                )?;
            }

            let conn = state.sqlite.lock().unwrap();
            queries::record_sync_complete(&conn, log_id, total_synced)?;

            tracing::info!("✅ GitHub CLI fallback succeeded: Synced {} milestones for {}/{}", total_synced, owner, name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("❌ GitHub CLI fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   All sync methods failed. Please ensure:");
            tracing::warn!("   1. You have access to this repository");
            tracing::warn!("   2. GitHub CLI is installed and authenticated: gh auth login");
            tracing::warn!("   3. For SAML-protected repos: gh auth status");
            Ok(()) // Don't fail the entire sync
        }
    }
}
```

---

### Step 2: Update REST API Fallback Error Handlers

#### Update PRs (Around line 650 in sync.rs)

Find the `sync_pull_requests_rest_fallback` function and change the error handler:

**Find:**
```rust
        Err(e) => {
            tracing::error!("❌ REST API fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   Please authorize this app at: https://github.com/orgs/{}/sso", owner);
            Ok(()) // Don't fail the entire sync
        }
```

**Replace with:**
```rust
        Err(rest_error) => {
            tracing::warn!("❌ REST API fallback failed for {}/{}: {}", owner, name, rest_error);
            tracing::info!("⚙️  Trying GitHub CLI fallback...");

            // Try CLI as final fallback
            return sync_pull_requests_cli_fallback(state, repo_id, owner, name, excluded_bots).await;
        }
```

#### Update Milestones (Around line 730 in sync.rs)

Find the `sync_milestones_rest_fallback` function and change the error handler:

**Find:**
```rust
        Err(e) => {
            tracing::error!("❌ REST API fallback failed for {}/{}: {}", owner, name, e);
            tracing::warn!("   Please authorize this app at: https://github.com/orgs/{}/sso", owner);
            Ok(()) // Don't fail the entire sync
        }
```

**Replace with:**
```rust
        Err(rest_error) => {
            tracing::warn!("❌ REST API fallback failed for {}/{}: {}", owner, name, rest_error);
            tracing::info!("⚙️  Trying GitHub CLI fallback...");

            // Try CLI as final fallback
            return sync_milestones_cli_fallback(state, repo_id, owner, name).await;
        }
```

---

## 🎯 How It Works

### Fallback Flow

```
┌─────────────────┐
│  Sync Request   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Try GraphQL    │
└────────┬────────┘
         │
    ┌────▼────┐
    │ Success?│
    └────┬────┘
         │
    No   │   Yes
    ┌────▼────────────────┐
    │ SAML Error?         │
    └────┬────────────────┘
         │
    Yes  │   No (return error)
    ┌────▼────────────────┐
    │ Try REST API        │
    └────┬────────────────┘
         │
    ┌────▼────┐
    │ Success?│
    └────┬────┘
         │
    No   │   Yes (done!)
    ┌────▼────────────────┐
    │ Try GitHub CLI      │
    └────┬────────────────┘
         │
    ┌────▼────┐
    │ Success?│
    └────┬────┘
         │
    Yes  │   No
    Done │   Log all errors
         │   Continue with
         │   other repos
         ▼
```

### User Experience

**With CLI installed and authenticated:**
```
[INFO] Syncing milestones for microsoft/amplifier...
⚠️  SAML SSO required for microsoft/amplifier, trying REST API fallback...
❌ REST API fallback failed: 403 Forbidden
⚙️  Trying GitHub CLI fallback...
✅ GitHub CLI fallback succeeded: Synced 12 milestones for microsoft/amplifier
```

**Without CLI installed:**
```
[INFO] Syncing milestones for microsoft/amplifier...
⚠️  SAML SSO required for microsoft/amplifier, trying REST API fallback...
❌ REST API fallback failed: 403 Forbidden
⚙️  Trying GitHub CLI fallback...
❌ GitHub CLI not available: GitHub CLI (gh) is not installed. Install from: https://cli.github.com
   All sync methods failed. Please ensure:
   1. You have access to this repository
   2. GitHub CLI is installed and authenticated: gh auth login
   3. For SAML-protected repos: gh auth status
```

---

## 📝 Testing Plan

### 1. Unit Tests (Optional)
Test CLI command execution and response parsing:
```bash
cd src-tauri
cargo test cli::
```

### 2. Manual Integration Test

**Prerequisites:**
1. Install GitHub CLI: `winget install GitHub.cli`
2. Authenticate: `gh auth login`
3. For SAML repos: `gh auth status` (ensure SSO is authorized)

**Test Steps:**
1. Add a SAML-protected repository to config
2. Run sync: `npm run dev:tauri`
3. Check logs for CLI fallback messages
4. Verify data appears in database/UI

**Expected Result:**
- GraphQL fails with SAML error
- REST API fallback fails
- CLI fallback succeeds
- Data synced successfully

### 3. Test Without CLI

1. Temporarily rename `gh.exe` to test error handling
2. Run sync
3. Should see clear error messages about missing CLI
4. Other repos should continue syncing

---

## 🐛 Common Issues & Solutions

### Issue: "gh: command not found"
**Solution:** Install GitHub CLI from https://cli.github.com

### Issue: "gh: not logged in"
**Solution:** Run `gh auth login`

### Issue: "gh: SAML protected"
**Solution:** Run `gh auth refresh -h github.com -s admin:org`

### Issue: CLI returns empty arrays
**Solution:** Check repository access with `gh repo view owner/repo`

### Issue: Compilation errors about `tokio::process`
**Solution:** Already included in tokio "full" features in Cargo.toml

---

## 📊 Success Metrics

After implementation:
- [ ] Can sync SAML-protected repositories without manual intervention
- [ ] Clear error messages guide users to fix authentication issues
- [ ] Fallback adds < 2 seconds overhead per operation
- [ ] Zero data inconsistencies between GraphQL, REST, and CLI sources
- [ ] All 41 core Microsoft org repos can be tracked

---

## 🎉 Next Steps After Implementation

1. **Test with your SAML repos** - Verify it works for microsoft/* repos
2. **Update documentation** - Add CLI setup to QUICK_START.md
3. **Consider Phase 2 enhancements:**
   - CLI-only mode (skip GraphQL entirely)
   - Parallel verification (compare GraphQL vs CLI results)
   - Auto-detect and suggest CLI installation
   - Cache CLI availability checks

4. **Move to next feature:**
   - Project Deep Dive page
   - User-Centric View
   - Generate those awesome productivity reports!

---

## 📂 Files Modified

```
src-tauri/src/github/
├── cli.rs                 ✅ New file (complete)
├── mod.rs                 ✅ Updated (complete)
└── sync.rs                🟡 Updated (needs completion)
```

**Total lines added:** ~450 lines of Rust code
**Estimated completion time:** 15 minutes remaining
