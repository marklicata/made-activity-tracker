# Rust GitHub Sync: Data Completeness & Separation Spec

## Goals
- Ensure repositories and users are independent, fully populated entities (no hidden coupling).
- Guarantee incremental, gap-free sync for repos, PRs, issues, reviews, and milestones.
- Make tracked users meaningful (either drive syncing or scope user-facing views).
- Prevent stale overwrites; reliably attach milestones and user references.

## Current Problems (observed)
- Repo-driven only: `tracked_users` isn’t used to scope sync or UI; adding a tracked user just triggers a full repo sync.
- Upserts overwrite blindly and don’t backfill: issues/PRs/reviews ignore newer-vs-older ordering and don’t refresh author/assignee/reviewer on conflict; users don’t refresh is_bot/name/avatar.
- No per-entity watermarks: incremental sync relies on `history_days`, so long gaps or late reviews can be missed.
- Milestone lookup mismatch: schema stores milestone `github_id` (databaseId) but sync looks up by number, so milestones can stay unset.
- Completeness risk: user-centric views depend on author/reviewer links that may stay NULL because upserts don’t fix them.

## Target Design (minimal, high-impact)
- Add per-entity watermarks (`sync_updated_at`) to issues, pull_requests, pr_reviews; drive incremental sync off the max watermark per repo/type.
- Guard upserts with updatedAt/submittedAt ordering and always backfill foreign keys (author/assignee/reviewer/milestone) when present.
- Fix milestone identity: match by milestone `github_id` (databaseId) with UNIQUE(repo_id, github_id); only fall back to title if id missing.
- Refresh user metadata on conflict (login/name/avatar/is_bot).
- Consolidate tracked state into `users` via `tracked BOOLEAN DEFAULT false` and `tracked_at TEXT NULL` for UI filtering/alerting only; sync remains repo-driven (no separate `tracked_users` table needed post-migration).

## Schema Changes
1) Add `sync_updated_at TEXT NULL` to:
   - issues (backfill with updated_at)
   - pull_requests (backfill with updated_at)
   - pr_reviews (backfill with submitted_at)
2) Milestones: ensure `github_id` stores databaseId; add UNIQUE(repo_id, github_id) index. Title uniqueness can remain but isn’t the primary match.
3) Consolidate tracking into users:
   - Add `tracked BOOLEAN NOT NULL DEFAULT false` and `tracked_at TEXT NULL` to users.
   - Migrate existing `tracked_users` rows into users (set tracked=true, tracked_at=added_at) and then deprecate/remove `tracked_users` table usage.

## Sync Pipeline Changes
- For each repo/type (issues, PRs, reviews):
  - Compute `since = max(sync_updated_at)` for that repo/type; fallback to `now - history_days` on first run.
  - Pass `since` into GraphQL/REST for issues, PRs, and reviews.
  - Set `sync_updated_at = updatedAt/submittedAt` from GitHub payload.
- Milestones: resolve by `github_id` (databaseId); only use title fallback if id missing.

## Upsert Rules (db/queries.rs)
- issues/pull_requests/pr_reviews:
  - On conflict, update only if `incoming.sync_updated_at >= existing.sync_updated_at OR existing.sync_updated_at IS NULL`.
  - Always set author_id/assignee_id/reviewer_id/milestone_id when provided (backfill missing FKs).
- users:
  - On conflict, refresh login, name, avatar_url, is_bot when provided.
  - Allow setting tracked/tracked_at and keep them on conflict (UI-only semantics; does not change sync scope).

## Tracked Users (decision)
- Consolidate into `users.tracked`/`tracked_at`; use for UI filtering/alerting only. Sync remains repo-driven.

## Implementation Plan (step-by-step)
1) Schema migration
   - Add columns: sync_updated_at to issues/pull_requests/pr_reviews; tracked, tracked_at to users.
   - Add UNIQUE(repo_id, github_id) on milestones (confirm github_id is databaseId).
   - Data backfill: set sync_updated_at = updated_at (issues/PRs), sync_updated_at = submitted_at (reviews); set users.tracked/tracked_at from tracked_users table if present.
   - Leave tracked_users table intact temporarily for backward compatibility but mark deprecated.

2) Config and Tauri commands
   - Update AppConfig (Rust) to carry users with tracked/tracked_at or map tracked_users -> users.tracked internally.
   - Update load_config/save_config commands to translate trackedUsers from UI into users.tracked and back (or emit tracked_users for backward compatibility while reading into users.tracked).
   - Update team/commands.rs to set users.tracked/tracked_at instead of writing tracked_users table; remove automatic full repo sync on toggle unless explicitly requested.

3) Sync pipeline
   - Compute since = max(sync_updated_at) per repo/type (issues, PRs, reviews); fallback to now - history_days on first run.
   - Pass since to GraphQL/REST; set sync_updated_at from updatedAt/submittedAt.
   - Upsert guards: only update when incoming.sync_updated_at >= existing.sync_updated_at OR existing is NULL; always backfill author/assignee/reviewer/milestone when provided.
   - Milestones: resolve by github_id (databaseId); title fallback only if id missing.

4) Frontend config store + Settings page
   - Update configStore to read/write tracked users in the new shape expected by backend (tracked flag on users).
   - Keep UX identical (checkbox list), but ensure serialization matches the new contract.
   - If we add a “Tracked only” filter later, wire it to backend flag; for now ensure Settings persists tracked correctly.

5) Cleanup
   - Remove/deprecate tracked_users table usage from code paths once migration is safe; retain table for one release if needed.
   - Update specs/AI_CHAT_PANEL.md and any other docs if they reference tracked_users table semantics.

## Testing Plan (incremental, avoid breakage)
1) Migration tests
   - Run migration on a copy of the DB; verify schema (new columns + milestone index) and backfilled values.
   - Ensure existing tracked_users entries became users.tracked=true.

2) Config/commands
   - Load existing config with tracked_users; verify it surfaces in UI as tracked users.
   - Save config; verify persisted shape matches chosen contract and reloads correctly.

3) Sync pipeline
   - Seed DB with older data; run sync with updated code and confirm:
     - since uses watermarks (inspect logs), and sync_updated_at updates.
     - Late review/issue update after initial sync is picked up on next sync.
     - Milestones attach by github_id.
     - Authors/assignees/reviewers are backfilled on conflicting rows.

4) Frontend
   - Manual UI check: add/remove/toggle tracked user in Settings; reload app; ensure state persists and reflects in backend config.

5) Regression
   - Project timelines and user views still render (basic smoke in app).
   - No crashes on sync when tracked list is empty or populated.

## Validation Checklist
- Late-arriving review appears after incremental sync (no gaps).
- Milestones show on issues/PRs consistently.
- Authors/assignees/reviewers populated for previously partial records.
- Users’ avatar/name/is_bot reflect latest GitHub state.
- Tracked users visible via users.tracked UI filter (no separate table).

## Decisions
- No per-repo/type cursors in sync_log beyond watermarks.
- No extra bot heuristics beyond GitHub’s is_bot flag.

## Implementation Plan (step-by-step, with checkpoints)
1) Schema migration
   - Add sync_updated_at columns (issues, pull_requests, pr_reviews); backfill from updated_at/submitted_at.
   - Add users.tracked (BOOL DEFAULT false) and users.tracked_at (TEXT NULL); migrate tracked_users rows into users, then stop using tracked_users table in code (table can remain temporarily for rollback safety).
   - Add UNIQUE(repo_id, github_id) on milestones to align lookup; verify github_id stores databaseId.
   - Checkpoint: run migration, inspect schema; spot-check a few rows for backfilled sync_updated_at and migrated tracked flags.

2) Backend config contract (Rust)
   - Update AppConfig to represent tracked users via users.tracked (or map tracked_users field into tracked flags on load). Preserve backward compatibility by accepting tracked_users in config input and emitting tracked users on save if desired.
   - Update load_config/save_config commands to translate config <-> users.tracked.
   - Checkpoint: load/save round-trip in dev; verify config persists tracked flags and repositories unchanged.

3) Backend commands/services
   - team/commands.rs: replace tracked_users table usage with users.tracked/tracked_at upsert; remove forced full repo sync on toggle (unless explicitly desired later).
   - Any “list tracked users” command should read users.tracked=true.
   - Checkpoint: invoke tracked user add/remove/toggle via Tauri command (manual or test) and confirm users table updates tracked fields.

4) Sync pipeline (github/sync.rs)
   - For each repo/type (issues, PRs, reviews): compute since = max(sync_updated_at) for that repo/type; fallback to now - history_days on first run. Pass since to GraphQL/REST. Set sync_updated_at from updatedAt/submittedAt.
   - Upsert guards: only update when incoming >= existing sync_updated_at (or existing NULL); always backfill author/assignee/reviewer/milestone IDs when present.
   - Milestones: resolve by github_id (databaseId); title fallback only if id missing.
   - Users: on conflict, refresh login/name/avatar/is_bot and tracked/tracked_at if provided.
   - Checkpoint: run a single-repo sync in dev; verify logs show since watermark; spot-check DB rows for updated sync_updated_at and foreign keys filled.

5) Frontend config store & Settings UI
   - Update configStore to map trackedUsers UI to backend contract (users.tracked). Keep UX unchanged.
   - Settings page continues to add/remove/toggle tracked users but routes through updated store shape.
   - Checkpoint: add/toggle/remove tracked users in UI; reload app; verify persistence and backend sees tracked flags.

6) Optional UI filtering
   - Add a “Tracked only” toggle for user-centric views that applies users.tracked filter via backend queries (if/when needed). Not required to ship core fix but plan for parity.
   - Checkpoint: if implemented, verify toggling narrows results as expected.

7) Cleanup and backward compatibility
   - Leave tracked_users table physically until confident; code should stop writing/reading it after migration.
   - Ensure config load/save handles legacy tracked_users gracefully.

## Test Strategy (do-as-you-go)
- Migration validation: after running migrations, query schema and sample rows to ensure sync_updated_at is populated and tracked flags migrated.
- Config round-trip: load/save config locally; confirm tracked users persist and repositories unaffected.
- Command tests: exercise tracked user add/remove/toggle via Tauri invoke (or integration test) and confirm users.tracked/tracked_at update.
- Sync sanity: run sync on one repo with known recent activity; verify latest issues/PRs/reviews reflect changes and milestones attach; confirm sync_updated_at advances.
- Regression checks: UI Settings add/remove/toggle tracked users, reload page, confirm persistence. Spot-check project/user views still load data.
- Safety: avoid enabling destructive schema drops; keep tracked_users table during rollout; prefer feature-flagging UI filter changes if added.

- tracked_users table deprecated; tracking lives on users.tracked/tracked_at; sync remains repo-driven.
