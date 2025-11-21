# Activity Tracker Implementation Tasks

**Project**: Activity Tracking System for Amplifier  
**Based on**: issue-manager substrate  
**Total Estimated Effort**: 50-70 hours  
**Status**: Planning Phase

---

## Phase 1: MVP - Basic Automatic Tracking (Week 1-2, ~25 hours)

**Goal**: Prove the concept with core functionality - automatic duplicate detection and idea filing

### Module 1: hooks-activity-tracker

#### Task 1.1: Module Scaffold (2 hours)
- [ ] Create repository structure
  - [ ] `amplifier-module-hooks-activity-tracker/`
  - [ ] `amplifier_module_hooks_activity_tracker/__init__.py`
  - [ ] `pyproject.toml` with dependencies
  - [ ] `README.md` with module overview
  - [ ] `.gitignore`
- [ ] Define module entry point (`mount()` function)
- [ ] Set up coordinator integration
- [ ] Add basic logging

**Acceptance Criteria**:
- Module loads without errors
- Can be mounted via profile configuration
- Logs initialization messages

**Dependencies**: None  
**Priority**: P0 (blocking)

---

#### Task 1.2: Context Capture (3 hours)
- [ ] Implement `_capture_context()` method
  - [ ] Get initial prompt from event_data
  - [ ] Get current working directory
  - [ ] Get git status (if git repo)
  - [ ] Get recently modified files (last 24h)
  - [ ] Get session_id
- [ ] Handle non-git directories gracefully
- [ ] Add timeout protection for git commands
- [ ] Write unit tests for context capture

**Acceptance Criteria**:
- Captures all context fields reliably
- Doesn't crash on non-git directories
- Git commands timeout after 5 seconds
- Test coverage >80%

**Dependencies**: Task 1.1  
**Priority**: P0 (blocking)

---

#### Task 1.3: Session Start Hook - Basic (4 hours)
- [ ] Implement `on_session_start()` hook
  - [ ] Register with coordinator for `session:start` event
  - [ ] Capture context
  - [ ] Get issue-manager from coordinator
  - [ ] Query open issues from issue-manager
  - [ ] Create session tracking issue
  - [ ] Store session_id -> issue_id mapping
- [ ] Add graceful degradation if issue-manager not available
- [ ] Log all operations for debugging
- [ ] Write integration tests

**Acceptance Criteria**:
- Hook fires on session start
- Creates tracking issue with correct metadata
- Handles missing issue-manager gracefully
- Logs capture context and issue creation

**Dependencies**: Task 1.2  
**Priority**: P0 (blocking)

---

#### Task 1.4: Session End Hook - Basic (4 hours)
- [ ] Implement `on_session_end()` hook
  - [ ] Register with coordinator for `session:end` event
  - [ ] Retrieve session issue_id from mapping
  - [ ] Update issue with basic summary
  - [ ] Clean up session tracking
- [ ] Handle missing session_id gracefully
- [ ] Add error recovery for failed updates
- [ ] Write integration tests

**Acceptance Criteria**:
- Hook fires on session end
- Updates session issue status
- Cleans up internal state
- No memory leaks

**Dependencies**: Task 1.3  
**Priority**: P0 (blocking)

---

#### Task 1.5: Basic Notification System (2 hours)
- [ ] Implement `_notify_related_work()` method
  - [ ] Format notification message
  - [ ] Display via coordinator (if available)
  - [ ] Or print to console as fallback
- [ ] Add configuration for notification style
- [ ] Support silent mode
- [ ] Write display tests

**Acceptance Criteria**:
- Notifications display clearly
- Shows issue ID, title, confidence
- Respects silent mode setting
- Falls back gracefully

**Dependencies**: Task 1.3  
**Priority**: P1 (important)

---

### Module 2: ActivityAnalyzer - Simple LLM (Phase 1)

#### Task 1.6: Analyzer Scaffold (2 hours)
- [ ] Create `activity_analyzer.py`
- [ ] Implement `ActivityAnalyzer` class
  - [ ] `__init__()` with config
  - [ ] LLM client initialization
  - [ ] Configuration validation
- [ ] Add error handling for LLM failures
- [ ] Write basic unit tests

**Acceptance Criteria**:
- Class initializes correctly
- LLM client configured from Amplifier session
- Validates required config fields
- Tests pass

**Dependencies**: None  
**Priority**: P0 (blocking)

---

#### Task 1.7: Simple LLM Duplicate Detection (4 hours)
- [ ] Implement `find_related_work()` - LLM only (no embeddings yet)
  - [ ] Build analysis prompt
  - [ ] Include context (prompt, git status)
  - [ ] Include candidate issues (limit to 20 for now)
  - [ ] Call LLM with JSON format
  - [ ] Parse LLM response
  - [ ] Return structured results
- [ ] Add retry logic for LLM failures
- [ ] Add timeout (30 seconds)
- [ ] Write tests with mock LLM

**Acceptance Criteria**:
- Returns list of related issues with confidence scores
- Handles LLM timeouts gracefully
- Parses JSON reliably
- Test coverage >80%

**Dependencies**: Task 1.6  
**Priority**: P0 (blocking)

---

#### Task 1.8: Session Work Analysis (3 hours)
- [ ] Implement `analyze_session_work()`
  - [ ] Build session analysis prompt
  - [ ] Include last 30 messages
  - [ ] Extract: completed, summary, new_ideas
  - [ ] Parse structured response
- [ ] Handle long transcripts (truncation)
- [ ] Add retry logic
- [ ] Write tests with mock data

**Acceptance Criteria**:
- Extracts work summary reliably
- Identifies new ideas with 70%+ accuracy
- Handles edge cases (empty sessions, very short sessions)
- Tests pass

**Dependencies**: Task 1.6  
**Priority**: P0 (blocking)

---

### Integration & Testing

#### Task 1.9: End-to-End Integration (3 hours)
- [ ] Connect hooks to analyzer
- [ ] Test full session lifecycle:
  - [ ] Start session
  - [ ] Capture context
  - [ ] Query issues
  - [ ] (Simple LLM analysis - later)
  - [ ] Create tracking issue
  - [ ] End session
  - [ ] Analyze work
  - [ ] Update issue
- [ ] Verify issue-manager state
- [ ] Test error recovery paths

**Acceptance Criteria**:
- Full cycle works end-to-end
- Issues created in correct format
- No crashes or hangs
- Clean shutdown

**Dependencies**: Tasks 1.3, 1.4, 1.8  
**Priority**: P0 (blocking)

---

#### Task 1.10: Documentation - Phase 1 (2 hours)
- [ ] Write setup guide
  - [ ] Installation instructions
  - [ ] Profile configuration example
  - [ ] issue-manager setup
- [ ] Write user guide
  - [ ] How it works
  - [ ] What to expect
  - [ ] Troubleshooting
- [ ] Write configuration reference
- [ ] Add code comments

**Acceptance Criteria**:
- New user can set up in <15 minutes
- All config options documented
- Examples work as-is

**Dependencies**: Task 1.9  
**Priority**: P1 (important)

---

## Phase 2: Enhanced Analysis - Embeddings (Week 3, ~20 hours)

**Goal**: Add embeddings for speed and accuracy

### Embeddings Infrastructure

#### Task 2.1: Embedding Generator (3 hours)
- [ ] Implement `EmbeddingGenerator` class
  - [ ] OpenAI API integration
  - [ ] Error handling and retries
  - [ ] Rate limiting
  - [ ] Batch generation support
- [ ] Add configuration for model selection
- [ ] Add fallback to LLM-only mode if API fails
- [ ] Write tests with mock API

**Acceptance Criteria**:
- Generates embeddings reliably
- Handles API errors gracefully
- Respects rate limits
- Tests pass

**Dependencies**: Phase 1 complete  
**Priority**: P0 (blocking)

---

#### Task 2.2: Embedding Cache - SQLite (4 hours)
- [ ] Implement `EmbeddingCache` class
  - [ ] SQLite database creation
  - [ ] Table schema (issue_id, embedding, content_hash, model, created_at)
  - [ ] `get()` method with content hash check
  - [ ] `set()` method
  - [ ] `invalidate()` method
  - [ ] `clear()` method for testing
- [ ] Add automatic cache location (`.amplifier/embeddings_cache.db`)
- [ ] Add cache hit/miss metrics
- [ ] Write comprehensive tests

**Acceptance Criteria**:
- Cache persists across sessions
- Correctly detects stale embeddings via content hash
- Cache operations are fast (<10ms)
- Test coverage >90%

**Dependencies**: None  
**Priority**: P0 (blocking)

---

#### Task 2.3: Two-Phase Matching Algorithm (4 hours)
- [ ] Refactor `find_related_work()` to use two phases:
  - [ ] Phase 1: Embedding similarity pre-filter
    - [ ] Generate context embedding
    - [ ] Load/generate issue embeddings
    - [ ] Compute cosine similarity
    - [ ] Filter by threshold (>0.7)
    - [ ] Sort and take top 10
  - [ ] Phase 2: LLM reasoning (existing logic)
    - [ ] Only analyze candidates from phase 1
- [ ] Add `_cosine_similarity()` helper
- [ ] Add `_get_cached_embedding()` helper
- [ ] Benchmark performance
- [ ] Write tests

**Acceptance Criteria**:
- Analysis completes in <5s for 100 issues
- Maintains or improves accuracy vs LLM-only
- Cache hit rate >70% after warmup
- Tests pass

**Dependencies**: Tasks 2.1, 2.2  
**Priority**: P0 (blocking)

---

#### Task 2.4: Content Hashing (2 hours)
- [ ] Implement content hash generation
  - [ ] Hash title + description
  - [ ] Use SHA-256
- [ ] Add hash to Issue metadata or separate tracking
- [ ] Invalidate cache on issue updates
- [ ] Write tests

**Acceptance Criteria**:
- Detects changed issues reliably
- Cache invalidation works correctly
- Minimal performance overhead

**Dependencies**: Task 2.2  
**Priority**: P1 (important)

---

### Performance & Optimization

#### Task 2.5: Performance Benchmarking (3 hours)
- [ ] Create benchmark script
  - [ ] Generate test issues (10, 50, 100, 200)
  - [ ] Measure analysis time
  - [ ] Measure cache hit rate
  - [ ] Measure memory usage
- [ ] Document performance characteristics
- [ ] Identify bottlenecks
- [ ] Add performance tests

**Acceptance Criteria**:
- Benchmark results documented
- Meets performance targets (<5s for 100 issues)
- Memory usage acceptable (<100MB)

**Dependencies**: Task 2.3  
**Priority**: P1 (important)

---

#### Task 2.6: Async/Concurrent Processing (2 hours)
- [ ] Add concurrent embedding generation
  - [ ] Batch API calls where possible
  - [ ] Use asyncio.gather for parallel requests
- [ ] Add concurrent cache reads
- [ ] Test with large issue sets
- [ ] Ensure thread safety

**Acceptance Criteria**:
- 2x+ speedup on large issue sets
- No race conditions
- Error handling maintained

**Dependencies**: Task 2.3  
**Priority**: P2 (nice to have)

---

#### Task 2.7: Documentation - Phase 2 (2 hours)
- [ ] Document embedding configuration
- [ ] Document cache management
- [ ] Add performance tuning guide
- [ ] Update troubleshooting guide

**Acceptance Criteria**:
- Users understand embedding configuration
- Cache management is clear
- Performance expectations documented

**Dependencies**: Tasks 2.3, 2.5  
**Priority**: P1 (important)

---

## Phase 3: Multi-Repo Support (Week 4, ~15 hours)

**Goal**: Project groups and cross-repo coordination

### Project Group Infrastructure

#### Task 3.1: ProjectGroupManager - Core (3 hours)
- [ ] Create `project_group_manager.py`
- [ ] Implement `ProjectGroupManager` class
  - [ ] `__init__()` with config path resolution
  - [ ] `_load_groups()` from YAML
  - [ ] `_save_groups()` to YAML
  - [ ] `get_group_for_repo()` - detect which group current repo belongs to
  - [ ] `get_group()` - get group config by name
  - [ ] `set_group()` - create/update group
  - [ ] `list_groups()` - list all groups
- [ ] Write unit tests

**Acceptance Criteria**:
- Loads/saves groups from YAML correctly
- Detects repo membership accurately
- Handles missing config gracefully
- Test coverage >80%

**Dependencies**: Phase 2 complete  
**Priority**: P0 (blocking)

---

#### Task 3.2: Multi-Repo Issue Querying (4 hours)
- [ ] Implement `_get_issues_for_repo()` helper
  - [ ] Create temporary IssueManager instance for target repo
  - [ ] Handle missing `.amplifier/issues/` directory
  - [ ] Return empty list if repo not initialized
- [ ] Update `on_session_start()` to query across group
  - [ ] Detect current project group
  - [ ] Query all repos in group
  - [ ] Aggregate results
  - [ ] Mark issues with source repo in metadata
- [ ] Add caching for cross-repo queries
- [ ] Write integration tests

**Acceptance Criteria**:
- Queries multiple repos reliably
- Handles missing/uninitialized repos
- Performance acceptable (<2s for 3 repos)
- Tests pass

**Dependencies**: Task 3.1  
**Priority**: P0 (blocking)

---

#### Task 3.3: Smart Repo Selection for New Issues (3 hours)
- [ ] Implement `_determine_target_repo()` method
  - [ ] Option 1: Default to current repo
  - [ ] Option 2: LLM suggests repo based on context
  - [ ] Option 3: Ask user (interactive)
- [ ] Add configuration for selection strategy
- [ ] Test with multi-repo scenarios
- [ ] Write tests

**Acceptance Criteria**:
- Issues created in appropriate repos
- User can override selection
- Default strategy works well (>80% correct)

**Dependencies**: Task 3.2  
**Priority**: P1 (important)

---

### Tool Enhancements

#### Task 3.4: Enhanced tool-issue Operations (3 hours)
- [ ] Fork/extend existing tool-issue module
- [ ] Add `search_with_llm` operation
  - [ ] Natural language query
  - [ ] Optional group parameter
  - [ ] Uses ActivityAnalyzer
  - [ ] Returns ranked results
- [ ] Add `set_project_group` operation
- [ ] Add `list_project_groups` operation
- [ ] Update tool schema
- [ ] Write tests

**Acceptance Criteria**:
- New operations work correctly
- Backward compatible with existing operations
- Tool schema updated
- Tests pass

**Dependencies**: Tasks 3.1, 3.2  
**Priority**: P0 (blocking)

---

#### Task 3.5: Documentation - Phase 3 (2 hours)
- [ ] Write multi-repo setup guide
  - [ ] How to configure project groups
  - [ ] Best practices for repo organization
  - [ ] Examples for common scenarios
- [ ] Document new tool operations
- [ ] Add troubleshooting for multi-repo issues
- [ ] Create migration guide (single → multi-repo)

**Acceptance Criteria**:
- Teams can set up multi-repo tracking in <30 minutes
- All new features documented
- Migration path is clear

**Dependencies**: Tasks 3.1-3.4  
**Priority**: P1 (important)

---

## Phase 4: Polish & Production Ready (Week 5, ~10 hours)

**Goal**: Robust, well-tested, production-ready

### Error Handling & Robustness

#### Task 4.1: Comprehensive Error Handling (2 hours)
- [ ] Audit all error paths
- [ ] Add try/except with specific exceptions
- [ ] Add informative error messages
- [ ] Add error recovery where possible
- [ ] Log errors appropriately
- [ ] Add error metrics
- [ ] Test error scenarios

**Acceptance Criteria**:
- No unhandled exceptions
- Error messages guide users to solutions
- System degrades gracefully
- Errors logged with context

**Dependencies**: Phases 1-3 complete  
**Priority**: P0 (blocking)

---

#### Task 4.2: Logging & Diagnostics (2 hours)
- [ ] Add structured logging throughout
  - [ ] Use Python logging module
  - [ ] Add log levels appropriately
  - [ ] Include context in logs
- [ ] Add debug mode
- [ ] Add diagnostic commands/tools
- [ ] Document log locations
- [ ] Write logging tests

**Acceptance Criteria**:
- Logs are helpful for debugging
- Log levels used appropriately
- Debug mode provides detailed output
- No sensitive data in logs

**Dependencies**: Task 4.1  
**Priority**: P1 (important)

---

### Testing & Quality

#### Task 4.3: Integration Test Suite (2 hours)
- [ ] Create comprehensive integration tests
  - [ ] Full session lifecycle
  - [ ] Multi-repo scenarios
  - [ ] Error recovery
  - [ ] Performance tests
- [ ] Add test fixtures
- [ ] Add test data generators
- [ ] Document test execution

**Acceptance Criteria**:
- Integration tests cover major workflows
- Tests are repeatable
- Test data is realistic
- Tests run in <60 seconds

**Dependencies**: Phases 1-3 complete  
**Priority**: P0 (blocking)

---

#### Task 4.4: Performance Testing (1 hour)
- [ ] Run benchmarks from Task 2.5
- [ ] Verify performance targets met
- [ ] Profile code for bottlenecks
- [ ] Optimize if needed
- [ ] Document final performance characteristics

**Acceptance Criteria**:
- Meets all performance targets
- No memory leaks
- Acceptable CPU usage
- Performance documented

**Dependencies**: Task 4.3  
**Priority**: P1 (important)

---

### Documentation & Release

#### Task 4.5: Final Documentation (2 hours)
- [ ] Complete API reference
- [ ] Finalize user guide
- [ ] Complete troubleshooting guide
- [ ] Add FAQ
- [ ] Create video walkthrough (optional)
- [ ] Review all docs for accuracy
- [ ] Add examples

**Acceptance Criteria**:
- Documentation is complete
- Examples work as-is
- No broken links
- Clear and professional

**Dependencies**: Tasks 4.1-4.4  
**Priority**: P1 (important)

---

#### Task 4.6: Release Preparation (1 hour)
- [ ] Tag version 1.0.0
- [ ] Create release notes
- [ ] Publish modules
- [ ] Announce to team
- [ ] Set up issue tracking
- [ ] Plan post-release support

**Acceptance Criteria**:
- Release is tagged and published
- Release notes are clear
- Team is informed
- Support plan in place

**Dependencies**: Task 4.5  
**Priority**: P1 (important)

---

## Ongoing Tasks (Throughout Project)

### Task O.1: Code Review & Refactoring (Ongoing)
- [ ] Review code after each task
- [ ] Refactor for clarity
- [ ] Remove duplication
- [ ] Improve naming
- [ ] Add comments where needed

**Cadence**: After each major task  
**Priority**: P1 (important)

---

### Task O.2: Test Coverage Maintenance (Ongoing)
- [ ] Maintain >80% test coverage
- [ ] Add tests for bug fixes
- [ ] Review test quality
- [ ] Update tests when code changes

**Cadence**: Continuous  
**Priority**: P0 (blocking)

---

### Task O.3: Documentation Updates (Ongoing)
- [ ] Update docs as features change
- [ ] Add examples for new features
- [ ] Keep troubleshooting guide current
- [ ] Review docs for clarity

**Cadence**: After each phase  
**Priority**: P1 (important)

---

## Summary by Phase

### Phase 1 MVP: 10 tasks, ~25 hours
Focus: Basic automatic tracking, simple LLM analysis

### Phase 2 Enhanced: 7 tasks, ~20 hours
Focus: Embeddings, performance, accuracy

### Phase 3 Multi-Repo: 5 tasks, ~15 hours
Focus: Project groups, cross-repo coordination

### Phase 4 Polish: 6 tasks, ~10 hours
Focus: Error handling, testing, documentation, release

### Total: 28 discrete tasks, 50-70 hours estimated

---

## Task Tracking

**Current Status**: Planning Phase

Use this document to track progress:
- [ ] = Not started
- [x] = Complete
- [~] = In progress
- [!] = Blocked

Update this file as tasks complete and new issues arise.

---

## Priority Legend

- **P0 (Blocking)**: Must complete before moving forward
- **P1 (Important)**: Should complete in this phase
- **P2 (Nice to have)**: Can defer if time constrained
- **P3 (Future)**: Post-1.0 features

---

**Last Updated**: 2025-11-20
