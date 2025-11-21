# Testing Strategy - Activity Tracker

**Project**: Activity Tracking System for Amplifier  
**Last Updated**: 2025-11-20  
**Status**: Planning Phase

---

## Testing Philosophy

Follow Amplifier's implementation philosophy:
- **Ruthless simplicity** in test design
- **Test behavior, not implementation**
- **Focus on integration over unit tests** (but include both)
- **Make tests fast and reliable**
- **Tests as documentation** of expected behavior

### Testing Pyramid

```
        /\
       /  \
      /E2E \      10% - Full system tests
     /------\
    /        \
   /Integration\  30% - Module integration tests
  /------------\
 /              \
/   Unit Tests   \ 60% - Function/class level tests
------------------
```

---

## Test Coverage Goals

### Overall Coverage
- **Target**: >80% code coverage
- **Minimum**: >70% for MVP
- **Focus areas**: Critical paths, error handling, data integrity

### Per-Module Coverage
- **hooks-activity-tracker**: >85% (critical path)
- **ActivityAnalyzer**: >90% (complex logic)
- **ProjectGroupManager**: >80% (configuration heavy)
- **EmbeddingCache**: >95% (data integrity critical)

---

## Testing Levels

## 1. Unit Tests

**Purpose**: Test individual functions and classes in isolation

### What to Test

#### hooks-activity-tracker
- [ ] Context capture (`_capture_context()`)
  - Captures prompt correctly
  - Handles missing git repo
  - Handles git command timeout
  - Returns all required fields
- [ ] Git status parsing
  - Parses output correctly
  - Handles empty status
  - Handles errors
- [ ] File discovery
  - Finds recently modified files
  - Respects time window
  - Handles missing files
- [ ] Notification formatting
  - Formats messages correctly
  - Includes all required fields
  - Respects configuration

#### ActivityAnalyzer
- [ ] Embedding generation
  - Generates embeddings correctly
  - Handles API failures
  - Respects rate limits
- [ ] Cosine similarity calculation
  - Computes correctly
  - Handles edge cases (zero vectors)
- [ ] Content hashing
  - Generates consistent hashes
  - Detects changes
- [ ] Prompt building
  - Includes all context
  - Formats correctly
  - Truncates appropriately
- [ ] Response parsing
  - Parses JSON reliably
  - Handles malformed responses
  - Validates structure

#### ProjectGroupManager
- [ ] Configuration loading
  - Loads YAML correctly
  - Handles missing files
  - Validates structure
- [ ] Group resolution
  - Identifies correct group
  - Handles multiple groups
  - Handles no group
- [ ] Configuration saving
  - Writes YAML correctly
  - Creates directories
  - Preserves other config

#### EmbeddingCache
- [ ] Cache operations
  - get() returns cached values
  - get() returns None on miss
  - set() stores correctly
  - Content hash validation works
- [ ] Stale detection
  - Detects changed content
  - Regenerates embeddings
- [ ] Database operations
  - Creates tables correctly
  - Handles concurrent access
  - Cleans up properly

### Unit Test Framework

**Tools**:
- `pytest` - Test framework
- `pytest-asyncio` - Async test support
- `pytest-mock` - Mocking
- `pytest-cov` - Coverage reporting

**Mock Strategy**:
- Mock external dependencies (LLM, embeddings API, issue-manager)
- Use fixtures for common test data
- Create test doubles for complex objects

**Example Unit Test**:
```python
# test_context_capture.py
import pytest
from unittest.mock import Mock, patch
from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook

class TestContextCapture:
    @pytest.fixture
    def hook(self):
        config = {'notify_threshold': 0.85}
        return ActivityTrackerHook(config)
    
    @pytest.fixture
    def mock_event_data(self):
        return {
            'session_id': 'test-session-123',
            'initial_prompt': 'Implement authentication',
            'coordinator': Mock()
        }
    
    def test_capture_context_basic(self, hook):
        """Test basic context capture"""
        with patch('os.getcwd', return_value='/test/path'):
            context = hook._capture_context({
                'initial_prompt': 'Test prompt'
            })
        
        assert context['prompt'] == 'Test prompt'
        assert context['working_dir'] == '/test/path'
        assert 'git_status' in context
        assert 'recent_files' in context
    
    def test_capture_context_no_git(self, hook):
        """Test context capture in non-git directory"""
        with patch('subprocess.run', side_effect=FileNotFoundError):
            context = hook._capture_context({
                'initial_prompt': 'Test'
            })
        
        assert context['git_status'] is None  # Should not crash
    
    def test_capture_context_git_timeout(self, hook):
        """Test git command timeout"""
        with patch('subprocess.run', side_effect=subprocess.TimeoutExpired('git', 5)):
            context = hook._capture_context({
                'initial_prompt': 'Test'
            })
        
        assert context['git_status'] is None  # Should handle gracefully
```

---

## 2. Integration Tests

**Purpose**: Test module interactions and coordinator integration

### What to Test

#### Hook → Analyzer Integration
- [ ] Session start triggers analysis
- [ ] Analysis results passed to notification
- [ ] Errors handled gracefully

#### Hook → issue-manager Integration
- [ ] Can get issue-manager from coordinator
- [ ] Creates issues correctly
- [ ] Updates issues correctly
- [ ] Handles missing issue-manager

#### Analyzer → LLM Integration
- [ ] LLM called with correct prompts
- [ ] Responses parsed correctly
- [ ] API errors handled

#### Analyzer → Embeddings Integration
- [ ] Embeddings generated correctly
- [ ] Cache used appropriately
- [ ] Fallback to LLM-only mode works

#### Multi-Module Integration
- [ ] Hook + Analyzer + issue-manager full flow
- [ ] Hook + ProjectGroupManager + issue-manager
- [ ] All components work together

### Integration Test Framework

**Tools**:
- `pytest` - Test framework
- `pytest-asyncio` - Async support
- Real issue-manager instance (test database)
- Mock LLM and embeddings (controlled responses)

**Test Environment**:
- Temporary directory for test data
- Clean database per test
- Isolated coordinator instance

**Example Integration Test**:
```python
# test_integration_session_flow.py
import pytest
from pathlib import Path
from amplifier_core import Coordinator
from amplifier_module_issue_manager import IssueManager
from amplifier_module_hooks_activity_tracker import mount as mount_hooks

@pytest.fixture
async def test_environment(tmp_path):
    """Set up test environment with coordinator and issue-manager"""
    # Create coordinator
    coordinator = Coordinator()
    
    # Mount issue-manager
    issue_data_dir = tmp_path / '.amplifier' / 'issues'
    issue_data_dir.mkdir(parents=True)
    issue_manager = IssueManager(issue_data_dir)
    await coordinator.mount('issue-manager', issue_manager)
    
    # Mount hooks
    config = {
        'notify_threshold': 0.85,
        'auto_track_sessions': True
    }
    await mount_hooks(coordinator, config)
    
    return coordinator, issue_manager

@pytest.mark.asyncio
async def test_full_session_lifecycle(test_environment):
    """Test complete session start → end flow"""
    coordinator, issue_manager = test_environment
    
    # Simulate session start
    session_id = 'test-session-123'
    await coordinator.emit('session:start', {
        'session_id': session_id,
        'initial_prompt': 'Implement authentication',
        'coordinator': coordinator
    })
    
    # Verify tracking issue created
    issues = issue_manager.list_issues(status='open')
    assert len(issues) == 1
    assert 'Session:' in issues[0].title
    assert issues[0].metadata['session_id'] == session_id
    
    # Simulate session end
    await coordinator.emit('session:end', {
        'session_id': session_id,
        'messages': [
            {'role': 'user', 'content': 'Implement auth'},
            {'role': 'assistant', 'content': 'I implemented OAuth 2.0'}
        ],
        'coordinator': coordinator
    })
    
    # Verify issue updated
    issues = issue_manager.list_issues()
    assert issues[0].status == 'closed'  # Or 'open' depending on completion
```

---

## 3. Performance Tests

**Purpose**: Ensure system meets performance targets

### Performance Targets

| Metric | Target | Acceptable | Action if Miss |
|--------|--------|------------|----------------|
| Analysis time (100 issues) | <5s | <10s | Optimize pre-filter |
| Context capture | <500ms | <1s | Add caching |
| Embedding generation | <2s | <5s | Batch requests |
| Cache hit rate (warmed) | >70% | >50% | Review invalidation |
| Memory usage | <100MB | <200MB | Profile and optimize |
| Session start overhead | <1s | <2s | Async operations |

### Performance Test Suite

```python
# test_performance.py
import pytest
import time
from amplifier_module_hooks_activity_tracker import ActivityAnalyzer

class TestPerformance:
    @pytest.fixture
    def large_issue_set(self):
        """Generate 100 test issues"""
        return [
            create_test_issue(f"Issue {i}", f"Description {i}")
            for i in range(100)
        ]
    
    @pytest.mark.performance
    @pytest.mark.asyncio
    async def test_analysis_speed_100_issues(self, large_issue_set):
        """Test analysis completes in <5s for 100 issues"""
        analyzer = ActivityAnalyzer(config={'embedding_model': 'test'})
        context = {'prompt': 'Test query'}
        
        start = time.time()
        results = await analyzer.find_related_work(context, large_issue_set)
        elapsed = time.time() - start
        
        assert elapsed < 5.0, f"Analysis took {elapsed:.2f}s (target: <5s)"
    
    @pytest.mark.performance
    def test_cache_hit_rate(self):
        """Test cache achieves >70% hit rate"""
        cache = EmbeddingCache()
        
        # Warm cache
        for i in range(100):
            cache.set(f"issue-{i}", mock_embedding(), "model", "hash")
        
        # Test hits
        hits = 0
        for i in range(100):
            if cache.get(f"issue-{i}", "hash"):
                hits += 1
        
        hit_rate = hits / 100
        assert hit_rate > 0.7, f"Hit rate {hit_rate:.0%} (target: >70%)"
```

### Benchmarking Script

```python
# scripts/benchmark.py
"""
Benchmark script for performance testing

Usage: python scripts/benchmark.py --issues 100 --runs 5
"""
import asyncio
import time
import statistics
from amplifier_module_hooks_activity_tracker import ActivityAnalyzer

async def benchmark_analysis(num_issues, num_runs):
    analyzer = ActivityAnalyzer(config={})
    issues = generate_test_issues(num_issues)
    context = {'prompt': 'Test query'}
    
    times = []
    for run in range(num_runs):
        start = time.time()
        await analyzer.find_related_work(context, issues)
        elapsed = time.time() - start
        times.append(elapsed)
    
    print(f"\nBenchmark Results ({num_issues} issues, {num_runs} runs):")
    print(f"  Mean: {statistics.mean(times):.2f}s")
    print(f"  Median: {statistics.median(times):.2f}s")
    print(f"  Stdev: {statistics.stdev(times):.2f}s")
    print(f"  Min: {min(times):.2f}s")
    print(f"  Max: {max(times):.2f}s")

if __name__ == '__main__':
    asyncio.run(benchmark_analysis(100, 5))
```

---

## 4. End-to-End Tests

**Purpose**: Test complete user workflows in realistic scenarios

### E2E Test Scenarios

#### Scenario 1: New User Setup
```
Given: Fresh installation
When: User configures profile and starts first session
Then: System initializes correctly, creates tracking issue
```

#### Scenario 2: Duplicate Detection
```
Given: Existing open issue "Implement OAuth"
When: User starts session with "Add OAuth authentication"
Then: System notifies user of related work with high confidence
```

#### Scenario 3: Idea Filing
```
Given: User working on authentication
When: User mentions "we should add rate limiting" during session
And: Session ends
Then: System files new idea issue with discovered-from link
```

#### Scenario 4: Multi-Repo Coordination
```
Given: Project group with 3 repos
When: User starts session in one repo
Then: System searches all repos in group
And: Finds related work across repos
```

#### Scenario 5: Error Recovery
```
Given: LLM API is down
When: User starts session
Then: System falls back gracefully, still creates tracking issue
And: Logs error appropriately
```

### E2E Test Framework

**Tools**:
- Full Amplifier installation
- Real configuration files
- Actual git repositories
- Mock LLM responses (controlled but realistic)

**Example E2E Test**:
```python
# test_e2e_workflows.py
import pytest
from pathlib import Path
import subprocess

@pytest.mark.e2e
def test_new_user_workflow(tmp_path):
    """Test complete new user setup workflow"""
    # Set up test environment
    test_repo = tmp_path / 'test-project'
    test_repo.mkdir()
    
    # Initialize git
    subprocess.run(['git', 'init'], cwd=test_repo)
    
    # Install issue-manager
    issues_dir = test_repo / '.amplifier' / 'issues'
    issues_dir.mkdir(parents=True)
    
    # Create profile with activity tracking
    profile = test_repo / '.amplifier' / 'profiles' / 'test.md'
    profile.parent.mkdir(parents=True)
    profile.write_text("""
---
profile:
  name: test
  extends: dev
hooks:
  - module: hooks-activity-tracker
    config:
      auto_track_sessions: true
---
""")
    
    # Start Amplifier session (simulated)
    # This would use Amplifier CLI in real E2E test
    result = start_amplifier_session(
        profile='test',
        prompt='Implement user authentication',
        working_dir=test_repo
    )
    
    # Verify tracking issue created
    from amplifier_module_issue_manager import IssueManager
    manager = IssueManager(issues_dir)
    issues = manager.list_issues()
    
    assert len(issues) == 1
    assert 'Session:' in issues[0].title
    assert 'authentication' in issues[0].description.lower()
```

---

## 5. Manual Testing Checklist

**Purpose**: Human verification of UX and edge cases

### Pre-Release Manual Tests

#### Installation & Setup (30 min)
- [ ] Fresh installation on clean machine
- [ ] Profile configuration works
- [ ] issue-manager integration works
- [ ] Error messages are helpful
- [ ] Documentation is accurate

#### Core Workflows (45 min)
- [ ] Session start captures context
- [ ] Duplicate detection notifications appear
- [ ] Notifications are clear and actionable
- [ ] Session end files ideas correctly
- [ ] Ideas have correct metadata
- [ ] Dependencies are linked correctly

#### Multi-Repo (30 min)
- [ ] Project group configuration works
- [ ] Cross-repo search finds issues
- [ ] Issues created in correct repos
- [ ] Group detection is accurate

#### Error Scenarios (30 min)
- [ ] LLM API failure → graceful degradation
- [ ] Embeddings API failure → fallback works
- [ ] issue-manager missing → system continues
- [ ] Git errors → handled gracefully
- [ ] Network errors → retry logic works

#### Performance (15 min)
- [ ] Session start is fast (<2s)
- [ ] Analysis completes quickly (<5s)
- [ ] No noticeable lag
- [ ] Memory usage acceptable

#### Edge Cases (30 min)
- [ ] Empty prompts
- [ ] Very long prompts
- [ ] No open issues
- [ ] 100+ open issues
- [ ] Non-git directory
- [ ] Network disconnected

**Total Manual Testing Time**: ~3 hours per release

---

## Test Data Management

### Test Fixtures

**Location**: `tests/fixtures/`

**Contents**:
- `sample_issues.json` - Realistic test issues
- `sample_context.json` - Session contexts
- `sample_llm_responses.json` - Mock LLM outputs
- `sample_embeddings.pkl` - Pre-computed embeddings

### Test Data Generators

```python
# tests/generators.py
def create_test_issue(title, description, priority=2, status='open'):
    """Generate test issue with realistic data"""
    return Issue(
        id=str(uuid.uuid4()),
        title=title,
        description=description,
        status=status,
        priority=priority,
        issue_type='task',
        assignee=None,
        created_at=datetime.now(),
        updated_at=datetime.now()
    )

def create_test_context(prompt, with_git=True):
    """Generate test session context"""
    context = {
        'prompt': prompt,
        'working_dir': '/test/path',
        'recent_files': ['src/auth.py', 'tests/test_auth.py']
    }
    if with_git:
        context['git_status'] = 'M src/auth.py\n?? new_file.py'
    return context
```

---

## Continuous Integration

### CI Pipeline

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-python@v2
        with:
          python-version: '3.11'
      
      - name: Install dependencies
        run: |
          pip install -e ".[dev]"
      
      - name: Run unit tests
        run: pytest tests/unit -v --cov --cov-report=xml
      
      - name: Run integration tests
        run: pytest tests/integration -v
      
      - name: Check coverage
        run: |
          coverage report --fail-under=80
      
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

### Pre-Commit Hooks

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: pytest-unit
        name: Run unit tests
        entry: pytest tests/unit -x
        language: system
        pass_filenames: false
        always_run: true
      
      - id: pytest-coverage
        name: Check coverage
        entry: pytest tests/unit --cov --cov-fail-under=80
        language: system
        pass_filenames: false
        always_run: true
```

---

## Test Execution

### Running Tests

```bash
# All tests
pytest

# Unit tests only
pytest tests/unit

# Integration tests only
pytest tests/integration

# E2E tests only (slower)
pytest tests/e2e -v

# Performance tests
pytest tests/performance -m performance

# With coverage
pytest --cov=amplifier_module_hooks_activity_tracker --cov-report=html

# Specific test file
pytest tests/unit/test_context_capture.py

# Specific test
pytest tests/unit/test_context_capture.py::TestContextCapture::test_basic

# Watch mode (reruns on changes)
pytest-watch

# Parallel execution (faster)
pytest -n auto
```

### Coverage Reports

```bash
# Generate HTML coverage report
pytest --cov --cov-report=html
open htmlcov/index.html

# Generate terminal report
pytest --cov --cov-report=term

# Check coverage threshold
pytest --cov --cov-fail-under=80
```

---

## Test Organization

### Directory Structure

```
amplifier-module-hooks-activity-tracker/
├── amplifier_module_hooks_activity_tracker/
│   ├── __init__.py
│   ├── hooks.py
│   ├── analyzer.py
│   └── ...
├── tests/
│   ├── unit/
│   │   ├── test_context_capture.py
│   │   ├── test_analyzer.py
│   │   ├── test_embedding_cache.py
│   │   └── test_project_groups.py
│   ├── integration/
│   │   ├── test_hook_analyzer_integration.py
│   │   ├── test_hook_issue_manager_integration.py
│   │   └── test_multi_module_integration.py
│   ├── performance/
│   │   ├── test_analysis_performance.py
│   │   └── test_cache_performance.py
│   ├── e2e/
│   │   ├── test_new_user_workflow.py
│   │   ├── test_duplicate_detection.py
│   │   └── test_multi_repo_workflow.py
│   ├── fixtures/
│   │   ├── sample_issues.json
│   │   ├── sample_context.json
│   │   └── sample_llm_responses.json
│   ├── conftest.py  # Shared fixtures
│   └── generators.py  # Test data generators
└── pyproject.toml
```

---

## Debugging Failed Tests

### Common Issues

**Issue**: Test fails intermittently
- **Cause**: Race condition, timing issue
- **Fix**: Add explicit waits, use `pytest-timeout`

**Issue**: Mock not working
- **Cause**: Incorrect patch path
- **Fix**: Patch where used, not where defined

**Issue**: Coverage too low
- **Cause**: Missing edge case tests
- **Fix**: Review coverage report, add tests

**Issue**: Integration test hangs
- **Cause**: Async not awaited properly
- **Fix**: Ensure all async operations awaited

### Debug Commands

```bash
# Run with verbose output
pytest -vv

# Run with print statements
pytest -s

# Run with debugger on failure
pytest --pdb

# Run last failed tests only
pytest --lf

# Run with detailed traceback
pytest --tb=long
```

---

## Quality Gates

### Before Merge
- [ ] All tests pass
- [ ] Coverage >80%
- [ ] No linting errors
- [ ] Type checking passes
- [ ] Performance tests pass

### Before Release
- [ ] All tests pass (including E2E)
- [ ] Coverage >80%
- [ ] Manual testing complete
- [ ] Performance benchmarks meet targets
- [ ] Documentation updated
- [ ] No known critical bugs

---

## Test Metrics Tracking

### Track Over Time
- Test execution time (should stay fast)
- Coverage percentage (should stay high)
- Number of tests (should grow)
- Flaky test count (should be zero)
- Performance benchmark results

### Dashboard (Optional)
- Use pytest-html for reports
- Track metrics in CI/CD
- Set up alerts for regressions

---

## Testing Best Practices

### Do's
✅ Write tests first (TDD when appropriate)
✅ Test behavior, not implementation
✅ Use descriptive test names
✅ Keep tests independent
✅ Use fixtures for common setup
✅ Mock external dependencies
✅ Test edge cases
✅ Keep tests fast

### Don'ts
❌ Test implementation details
❌ Create interdependent tests
❌ Skip error case testing
❌ Ignore flaky tests
❌ Over-mock (test becomes meaningless)
❌ Write slow tests without marking
❌ Commit failing tests

---

## Phase-Specific Testing

### Phase 1 MVP
- Focus: Unit tests + basic integration
- Coverage target: >70%
- Performance: Not critical yet

### Phase 2 Enhanced
- Focus: Performance tests
- Coverage target: >80%
- Performance: Must meet targets

### Phase 3 Multi-Repo
- Focus: Integration tests across repos
- Coverage target: >80%
- Edge cases: Empty repos, missing config

### Phase 4 Polish
- Focus: E2E tests + manual testing
- Coverage target: >85%
- Quality: Production-ready

---

**Last Updated**: 2025-11-20  
**Review Schedule**: After each phase completion
