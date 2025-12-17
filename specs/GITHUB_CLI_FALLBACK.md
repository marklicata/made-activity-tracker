# Feature Spec: GitHub CLI Fallback

## Overview
Implement automatic fallback to GitHub CLI (`gh`) when GraphQL API calls fail, particularly for SAML-protected repositories. This provides a seamless experience when the GraphQL API returns errors while the CLI still has valid authentication.

## Priority
**HIGH** - Implement first

## Problem Statement
Currently, when syncing SAML-protected repositories, the GraphQL API fails with SAML authorization errors even when the user has already authenticated via SSO in their browser. However, the `gh` CLI tool works fine because it uses a different authentication mechanism that respects browser-based SSO sessions.

Example failure:
- GraphQL call to fetch issues/PRs fails with SAML error
- User runs `gh issue list --repo owner/repo` and it works perfectly

## User Story
As a developer working with SAML-protected repositories, I want the application to automatically use the GitHub CLI when API calls fail, so that I can sync data without manual intervention or workarounds.

## Requirements

### Functional Requirements

#### 1. Automatic Fallback Detection
- **FR-1.1**: When any GitHub GraphQL API call fails, detect the failure type
- **FR-1.2**: For SAML errors, automatically attempt to use `gh` CLI as fallback
- **FR-1.3**: For other API errors, also attempt CLI fallback to maximize success rate
- **FR-1.4**: If CLI also fails, surface both error messages to the user

#### 2. CLI Integration
- **FR-2.1**: Check if `gh` CLI is installed on the system
- **FR-2.2**: Verify that `gh` CLI is authenticated (`gh auth status`)
- **FR-2.3**: Execute appropriate `gh` commands based on the data being fetched
- **FR-2.4**: Parse CLI JSON output into the same data structures as GraphQL responses

#### 3. Supported Operations
All GitHub API operations should support CLI fallback:

- **FR-3.1**: **Issues**: `gh issue list --repo owner/repo --json ...`
- **FR-3.2**: **Pull Requests**: `gh pr list --repo owner/repo --json ...`
- **FR-3.3**: **Milestones**: `gh api repos/owner/repo/milestones`
- **FR-3.4**: **Repository Info**: `gh repo view owner/repo --json ...`
- **FR-3.5**: **User Info**: `gh api users/{username}`
- **FR-3.6**: **PR Reviews**: `gh api repos/owner/repo/pulls/{number}/reviews`

#### 4. Data Consistency
- **FR-4.1**: CLI responses must be transformed into the same `models.rs` structures
- **FR-4.2**: Database schema remains unchanged
- **FR-4.3**: Field mapping documented for each CLI command to model field

#### 5. User Feedback
- **FR-5.1**: Log when fallback is attempted (INFO level)
- **FR-5.2**: Log successful CLI fallback (INFO level)
- **FR-5.3**: Show user-facing message in UI when CLI fallback is used
- **FR-5.4**: If both methods fail, provide actionable error message

### Non-Functional Requirements

#### Performance
- **NFR-1**: CLI fallback should add < 2 seconds overhead per operation
- **NFR-2**: Cache CLI availability check (don't check on every call)
- **NFR-3**: Execute CLI commands with appropriate timeouts (30s default)

#### Reliability
- **NFR-4**: Gracefully handle CLI not installed
- **NFR-5**: Gracefully handle CLI not authenticated
- **NFR-6**: Support both `gh` and `github` CLI command names (legacy)

#### Maintainability
- **NFR-7**: Abstract CLI calls into a separate module (`github/cli.rs`)
- **NFR-8**: Use strongly-typed command builders (not raw string commands)
- **NFR-9**: Comprehensive error types for CLI failures

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────┐
│  Sync Operation (github/sync.rs)           │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│  GitHub API Client (github/mod.rs)         │
│  - Try GraphQL first                       │
│  - Catch errors                            │
│  - Determine if fallback appropriate       │
└─────────────────┬───────────────────────────┘
                  │
         ┌────────┴────────┐
         │                 │
         ▼                 ▼
┌────────────────┐  ┌─────────────────┐
│ GraphQL Path   │  │  CLI Path       │
│ (current)      │  │  (new)          │
│ graphql.rs     │  │  cli.rs         │
└────────┬───────┘  └────────┬────────┘
         │                   │
         └────────┬──────────┘
                  │
                  ▼
         ┌────────────────┐
         │ Unified Models │
         │ (models.rs)    │
         └────────────────┘
```

### New Modules

#### `src-tauri/src/github/cli.rs`
New module for CLI operations:

```rust
pub struct GitHubCli {
    command_path: String,
    is_available: bool,
    is_authenticated: bool,
}

impl GitHubCli {
    pub async fn new() -> Result<Self>;
    pub async fn check_auth(&self) -> Result<bool>;

    // Data fetching methods
    pub async fn fetch_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>>;
    pub async fn fetch_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>>;
    pub async fn fetch_milestones(&self, owner: &str, repo: &str) -> Result<Vec<Milestone>>;
    pub async fn fetch_repository(&self, owner: &str, repo: &str) -> Result<RepositoryInfo>;
    pub async fn fetch_user(&self, username: &str) -> Result<User>;
    pub async fn fetch_pr_reviews(&self, owner: &str, repo: &str, pr_number: i32) -> Result<Vec<PrReview>>;
}
```

#### Updated `src-tauri/src/github/mod.rs`
Add fallback logic:

```rust
pub enum FetchStrategy {
    GraphQL,
    CLI,
    AutoFallback,  // Default: try GraphQL, fallback to CLI
}

pub struct GitHubClient {
    graphql: GraphQLClient,
    cli: Option<GitHubCli>,
    strategy: FetchStrategy,
}

impl GitHubClient {
    pub async fn fetch_issues_with_fallback(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        match self.strategy {
            FetchStrategy::GraphQL => self.graphql.fetch_issues(owner, repo).await,
            FetchStrategy::CLI => self.cli.as_ref()?.fetch_issues(owner, repo).await,
            FetchStrategy::AutoFallback => {
                match self.graphql.fetch_issues(owner, repo).await {
                    Ok(issues) => Ok(issues),
                    Err(e) => {
                        tracing::warn!("GraphQL failed, trying CLI fallback: {}", e);
                        self.cli.as_ref()?.fetch_issues(owner, repo).await
                    }
                }
            }
        }
    }
}
```

### CLI Command Mapping

| Operation | GraphQL Query | CLI Command | JSON Fields |
|-----------|---------------|-------------|-------------|
| Issues | `query { repository { issues }}` | `gh issue list --repo owner/repo --state all --json number,title,body,state,author,assignees,milestone,createdAt,updatedAt,closedAt,labels --limit 1000` | All issue fields |
| PRs | `query { repository { pullRequests }}` | `gh pr list --repo owner/repo --state all --json number,title,body,state,author,createdAt,updatedAt,mergedAt,closedAt,additions,deletions,changedFiles,labels --limit 1000` | All PR fields |
| PR Reviews | `query { pullRequest { reviews }}` | `gh api repos/owner/repo/pulls/{number}/reviews` | Reviewer, state, submittedAt |
| Milestones | `query { repository { milestones }}` | `gh api repos/owner/repo/milestones` | All milestone fields |
| Repo Info | `query { repository { ... }}` | `gh repo view owner/repo --json name,owner,id,updatedAt` | Basic repo fields |
| User Info | `query { user { ... }}` | `gh api users/{username}` | User fields |

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum FallbackError {
    #[error("CLI not installed. Install with: https://cli.github.com")]
    CliNotInstalled,

    #[error("CLI not authenticated. Run: gh auth login")]
    CliNotAuthenticated,

    #[error("Both GraphQL and CLI failed. GraphQL: {graphql_error}, CLI: {cli_error}")]
    BothFailed {
        graphql_error: String,
        cli_error: String,
    },

    #[error("CLI command failed: {0}")]
    CliCommandFailed(String),

    #[error("Failed to parse CLI output: {0}")]
    CliParseError(String),
}
```

## Implementation Plan

### Phase 1: CLI Infrastructure (Priority 1)
1. Create `src-tauri/src/github/cli.rs` module
2. Implement `GitHubCli` struct with availability checks
3. Add CLI command execution utilities
4. Add tests for CLI detection and command building

### Phase 2: Data Fetching (Priority 1)
1. Implement `fetch_issues` with CLI
2. Implement `fetch_pull_requests` with CLI
3. Implement `fetch_milestones` with CLI
4. Implement `fetch_pr_reviews` with CLI
5. Add response parsing and model transformation
6. Add tests for each data fetching method

### Phase 3: Fallback Integration (Priority 1)
1. Update `github/mod.rs` with fallback logic
2. Modify `sync.rs` to use new client with fallback
3. Add logging for fallback events
4. Update error handling to surface both errors when both fail

### Phase 4: User Feedback (Priority 2)
1. Add UI notification when CLI fallback is used
2. Add settings page toggle for fetch strategy preference
3. Add diagnostics page showing GraphQL vs CLI status

### Phase 5: Documentation (Priority 2)
1. Update README with CLI requirements
2. Document troubleshooting steps for CLI auth
3. Add architecture diagram to docs

## Testing Strategy

### Unit Tests
- CLI detection on systems with/without `gh` installed
- CLI authentication status checking
- Command building for each operation type
- JSON parsing from CLI responses
- Model transformation from CLI data

### Integration Tests
- End-to-end sync using CLI only
- Fallback from GraphQL to CLI
- Error handling when both methods fail
- Performance comparison GraphQL vs CLI

### Manual Testing Checklist
- [ ] Sync SAML-protected repo with GraphQL (should fail)
- [ ] Sync SAML-protected repo with CLI fallback (should succeed)
- [ ] Sync normal repo (should use GraphQL)
- [ ] System without `gh` CLI (should show helpful error)
- [ ] System with unauthenticated `gh` (should show auth prompt)
- [ ] Compare data consistency between GraphQL and CLI fetches

## Success Metrics
- SAML-protected repositories can be synced successfully
- Fallback adds < 2 seconds overhead
- 0 data consistency issues between GraphQL and CLI sources
- User-facing error rate decreases by > 80% for SAML repos

## Future Enhancements
1. Parallel fetch (GraphQL + CLI) with verification mode
2. User preference to always use CLI (skip GraphQL entirely)
3. Automatic CLI installation prompt
4. CLI authentication within the app (launch browser flow)
5. Diff tool to compare GraphQL vs CLI results for debugging

## Dependencies
- `tokio::process::Command` for async CLI execution
- Existing `serde_json` for parsing CLI output
- Current database models (no schema changes needed)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CLI not installed | HIGH | Clear error message with installation link |
| CLI version incompatibility | MEDIUM | Check minimum version (gh 2.0.0+) |
| CLI output format changes | MEDIUM | Pin to stable JSON fields, add version check |
| Performance degradation | LOW | Benchmark and optimize, use caching |
| Different data between sources | MEDIUM | Extensive testing, field mapping validation |

## Open Questions
1. Should we cache CLI availability checks? For how long? - no, no need to cache these.
2. Should we support `hub` CLI as additional fallback? - I am not sure what that is. Let's not use it fo rnow.
3. Should there be a setting to prefer CLI over GraphQL? - No, always use GraphQL and CLI if that fails.
4. How do we handle pagination differences between GraphQL and CLI? - We care about the total list of items returned, not the pagination. So long as the full lists are returned, the pages don't matter.

## Related Issues
- Current branch: `fix-saml-error-handling`
- Related to Phase 1 SAML error detection already implemented
