# Project Structure - Activity Tracker

**Project**: Activity Tracking System for Amplifier  
**Last Updated**: 2025-11-20  
**Status**: Planning Complete, Ready for Implementation

---

## Repository Structure

### Main Repository: amplifier-module-hooks-activity-tracker

```
amplifier-module-hooks-activity-tracker/
├── amplifier_module_hooks_activity_tracker/
│   ├── __init__.py                 # Module entry point with mount()
│   ├── hooks.py                    # ActivityTrackerHook class
│   ├── analyzer.py                 # ActivityAnalyzer class
│   ├── embedding_cache.py          # EmbeddingCache class
│   ├── embedding_generator.py      # EmbeddingGenerator class
│   ├── project_group_manager.py    # ProjectGroupManager class
│   └── utils.py                    # Helper functions
│
├── tests/
│   ├── unit/
│   │   ├── test_hooks.py
│   │   ├── test_analyzer.py
│   │   ├── test_embedding_cache.py
│   │   ├── test_embedding_generator.py
│   │   ├── test_project_groups.py
│   │   └── test_utils.py
│   ├── integration/
│   │   ├── test_hook_analyzer.py
│   │   ├── test_hook_issue_manager.py
│   │   ├── test_multi_module.py
│   │   └── test_multi_repo.py
│   ├── performance/
│   │   ├── test_analysis_performance.py
│   │   └── test_cache_performance.py
│   ├── e2e/
│   │   ├── test_new_user_workflow.py
│   │   ├── test_duplicate_detection.py
│   │   ├── test_idea_filing.py
│   │   └── test_multi_repo_workflow.py
│   ├── fixtures/
│   │   ├── sample_issues.json
│   │   ├── sample_context.json
│   │   ├── sample_llm_responses.json
│   │   └── sample_embeddings.pkl
│   ├── conftest.py                 # Shared fixtures
│   └── generators.py               # Test data generators
│
├── docs/
│   ├── SETUP.md                    # Installation & setup guide
│   ├── USER_GUIDE.md               # User documentation
│   ├── API_REFERENCE.md            # API documentation
│   ├── CONFIGURATION.md            # Configuration reference
│   ├── TROUBLESHOOTING.md          # Common issues & solutions
│   ├── ARCHITECTURE.md             # Technical architecture
│   └── EXAMPLES.md                 # Usage examples
│
├── scripts/
│   ├── benchmark.py                # Performance benchmarking
│   ├── test_setup.py               # Test environment setup
│   └── migrate_from_beads.py       # Migration tool (optional)
│
├── .github/
│   └── workflows/
│       ├── test.yml                # CI test pipeline
│       ├── release.yml             # Release automation
│       └── coverage.yml            # Coverage reporting
│
├── pyproject.toml                  # Project configuration
├── README.md                       # Project overview
├── LICENSE                         # MIT License
├── .gitignore                      # Git ignore rules
├── .pre-commit-config.yaml         # Pre-commit hooks
└── CHANGELOG.md                    # Version history
```

---

## Module Breakdown

### 1. hooks.py - ActivityTrackerHook

**Purpose**: Amplifier session lifecycle integration

**Key Classes**:
```python
class ActivityTrackerHook:
    """Main hook class that integrates with Amplifier"""
    
    def __init__(self, config: dict)
    async def on_session_start(self, event_data: dict)
    async def on_session_end(self, event_data: dict)
    
    # Private helpers
    def _capture_context(self, event_data: dict) -> dict
    def _get_git_status(self) -> str | None
    def _get_recent_files(self, hours: int = 24) -> list[str]
    def _notify_related_work(self, event_data: dict, related: list)
    async def _get_issues_for_repo(self, repo_path: str) -> list[Issue]
```

**Dependencies**:
- `ActivityAnalyzer` - For LLM analysis
- `ProjectGroupManager` - For multi-repo coordination
- `issue-manager` (via coordinator) - For storage

**Configuration**:
```python
{
    'notify_threshold': 0.85,
    'embedding_model': 'text-embedding-3-small',
    'similarity_threshold': 0.7,
    'auto_track_sessions': True,
    'auto_file_ideas': True,
    'silent_mode': False
}
```

---

### 2. analyzer.py - ActivityAnalyzer

**Purpose**: LLM-powered work analysis and duplicate detection

**Key Classes**:
```python
class ActivityAnalyzer:
    """Analyzes session context and finds related work"""
    
    def __init__(self, config: dict)
    async def find_related_work(
        self, 
        context: dict, 
        open_issues: list[Issue]
    ) -> list[dict]
    async def analyze_session_work(
        self, 
        messages: list[dict]
    ) -> dict
    
    # Private helpers
    async def _generate_embedding(self, text: str) -> list[float]
    async def _get_cached_embedding(
        self, 
        issue_id: str, 
        content: str
    ) -> list[float]
    def _cosine_similarity(self, vec1: list, vec2: list) -> float
    def _build_analysis_prompt(
        self, 
        context: dict, 
        candidates: list
    ) -> str
    def _parse_llm_result(self, result: str) -> list[dict]
    def _format_messages(self, messages: list) -> str
    def _format_candidates(self, candidates: list) -> str
```

**Dependencies**:
- LLM client (from Amplifier session)
- `EmbeddingGenerator` - For embeddings
- `EmbeddingCache` - For caching

**Algorithms**:
- **Two-phase matching**: Embeddings pre-filter → LLM reasoning
- **Cosine similarity**: Vector comparison for pre-filtering
- **Confidence scoring**: LLM provides 0.0-1.0 confidence

---

### 3. embedding_cache.py - EmbeddingCache

**Purpose**: SQLite-based caching for embeddings

**Key Classes**:
```python
class EmbeddingCache:
    """Caches embeddings to avoid regeneration"""
    
    def __init__(self, cache_path: Path | None = None)
    async def get(
        self, 
        issue_id: str, 
        content_hash: str
    ) -> list[float] | None
    async def set(
        self, 
        issue_id: str, 
        embedding: list[float],
        model: str,
        content_hash: str
    )
    def invalidate(self, issue_id: str)
    def clear()
    def get_stats() -> dict
```

**Schema**:
```sql
CREATE TABLE embeddings (
    issue_id TEXT PRIMARY KEY,
    embedding BLOB,              -- Pickled numpy array
    content_hash TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_content_hash ON embeddings(content_hash);
CREATE INDEX idx_model ON embeddings(model);
```

**Storage Location**: `.amplifier/embeddings_cache.db` (gitignored)

---

### 4. embedding_generator.py - EmbeddingGenerator

**Purpose**: Generate embeddings via OpenAI API

**Key Classes**:
```python
class EmbeddingGenerator:
    """Generates embeddings using configured model"""
    
    def __init__(self, config: dict)
    async def generate(self, text: str) -> list[float]
    async def generate_batch(self, texts: list[str]) -> list[list[float]]
    
    # Private helpers
    async def _call_api(self, text: str) -> list[float]
    def _handle_rate_limit(self, retry_after: int)
    def _validate_embedding(self, embedding: list[float]) -> bool
```

**Configuration**:
```python
{
    'model': 'text-embedding-3-small',
    'api_key': os.getenv('OPENAI_API_KEY'),
    'max_retries': 3,
    'timeout': 30
}
```

**Error Handling**:
- Rate limiting with exponential backoff
- API failures with retry logic
- Fallback to None (triggers LLM-only mode)

---

### 5. project_group_manager.py - ProjectGroupManager

**Purpose**: Multi-repo project group coordination

**Key Classes**:
```python
class ProjectGroupManager:
    """Manages project groups for multi-repo tracking"""
    
    def __init__(self, config: dict)
    def get_group_for_repo(
        self, 
        repo_path: str
    ) -> tuple[str | None, dict | None]
    def get_group(self, group_name: str) -> dict | None
    def set_group(
        self, 
        group_name: str, 
        repos: list[str],
        description: str | None = None
    )
    def list_groups(self) -> dict
    def delete_group(self, group_name: str)
    
    # Private helpers
    def _find_config_path(self) -> Path
    def _load_groups(self) -> dict
    def _save_groups(self)
    def _validate_repos(self, repos: list[str]) -> bool
```

**Configuration Format** (YAML):
```yaml
activity:
  project_groups:
    auth-system:
      repos:
        - /Users/dev/code/auth-service
        - /Users/dev/code/user-service
        - /Users/dev/code/auth-frontend
      description: "Authentication system spanning multiple services"
    
    backend-platform:
      repos:
        - /Users/dev/code/auth-service
        - /Users/dev/code/api-gateway
      description: "Core backend platform"
  
  current_group: auth-system
```

**Storage Priority**:
1. `.amplifier/settings.yaml` (project-level)
2. `~/.amplifier/settings.yaml` (user-level)

---

### 6. utils.py - Helper Functions

**Purpose**: Shared utilities

**Functions**:
```python
def compute_content_hash(text: str) -> str
    """Generate SHA-256 hash of text"""

def format_notification(related_items: list[dict]) -> str
    """Format notification message for display"""

def parse_git_status(git_output: str) -> dict
    """Parse git status output into structured data"""

def find_recently_modified_files(
    directory: Path, 
    hours: int = 24
) -> list[str]
    """Find files modified in last N hours"""

def sanitize_llm_response(response: str) -> str
    """Clean up LLM response for parsing"""

def validate_config(config: dict, schema: dict) -> bool
    """Validate configuration against schema"""
```

---

## Configuration Files

### pyproject.toml

```toml
[project]
name = "amplifier-module-hooks-activity-tracker"
version = "0.1.0"
description = "Activity tracking hook for Amplifier with LLM-powered duplicate detection"
readme = "README.md"
requires-python = ">=3.11"
license = { text = "MIT" }
authors = [
    { name = "Your Name", email = "your.email@example.com" }
]

dependencies = [
    "amplifier-core",
    "openai>=1.0.0",
    "pyyaml>=6.0",
    "numpy>=1.24.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "pytest-asyncio>=0.21.0",
    "pytest-cov>=4.0.0",
    "pytest-mock>=3.10.0",
    "black>=23.0.0",
    "ruff>=0.1.0",
    "mypy>=1.0.0",
]

[project.entry-points."amplifier.modules"]
hooks-activity-tracker = "amplifier_module_hooks_activity_tracker"

[tool.pytest.ini_options]
testpaths = ["tests"]
asyncio_mode = "auto"
markers = [
    "unit: Unit tests",
    "integration: Integration tests",
    "e2e: End-to-end tests",
    "performance: Performance tests",
]

[tool.coverage.run]
source = ["amplifier_module_hooks_activity_tracker"]
omit = ["*/tests/*"]

[tool.coverage.report]
exclude_lines = [
    "pragma: no cover",
    "def __repr__",
    "raise NotImplementedError",
    "if __name__ == .__main__.:",
]

[tool.black]
line-length = 100
target-version = ["py311"]

[tool.ruff]
line-length = 100
target-version = "py311"

[tool.mypy]
python_version = "3.11"
strict = true
warn_return_any = true
warn_unused_configs = true
```

---

## Data Flow Diagrams

### Session Start Flow

```
User starts session
        ↓
coordinator.emit('session:start')
        ↓
ActivityTrackerHook.on_session_start()
        ↓
1. Capture context (prompt, git, files)
        ↓
2. Get project group (if any)
        ↓
3. Query open issues across group
   ├─ Current repo (issue-manager)
   └─ Other repos in group (temp IssueManager instances)
        ↓
4. ActivityAnalyzer.find_related_work()
   ├─ Phase 1: Generate embeddings
   │   ├─ Context embedding
   │   └─ Issue embeddings (cached)
   ├─ Phase 2: Cosine similarity pre-filter
   │   └─ Top 10 candidates
   └─ Phase 3: LLM reasoning
       └─ Confidence scores + reasoning
        ↓
5. If high-confidence matches (>0.85)
   └─ Display notification
        ↓
6. Create session tracking issue
        ↓
7. Store session_id → issue_id mapping
        ↓
Done (user continues session)
```

### Session End Flow

```
User ends session
        ↓
coordinator.emit('session:end')
        ↓
ActivityTrackerHook.on_session_end()
        ↓
1. Get session issue_id from mapping
        ↓
2. ActivityAnalyzer.analyze_session_work()
   ├─ Extract last 30 messages
   ├─ Build analysis prompt
   ├─ Call LLM
   └─ Parse: {completed, summary, new_ideas}
        ↓
3. Update session issue
   ├─ If completed → close issue
   └─ Else → update with summary
        ↓
4. For each new idea:
   ├─ Determine target repo (if multi-repo)
   ├─ Create new issue via issue-manager
   └─ Add discovered-from dependency
        ↓
5. Clean up session mapping
        ↓
Done (session complete)
```

---

## Development Workflow

### Initial Setup

```bash
# Clone repository
git clone https://github.com/your-org/amplifier-module-hooks-activity-tracker.git
cd amplifier-module-hooks-activity-tracker

# Create virtual environment
python -m venv venv
source venv/bin/activate  # or `venv\Scripts\activate` on Windows

# Install in development mode
pip install -e ".[dev]"

# Set up pre-commit hooks
pre-commit install

# Run tests to verify setup
pytest
```

### Development Cycle

```bash
# 1. Create feature branch
git checkout -b feature/your-feature

# 2. Make changes
# ... edit code ...

# 3. Run tests
pytest tests/unit  # Fast unit tests
pytest              # Full suite

# 4. Check coverage
pytest --cov --cov-report=html
open htmlcov/index.html

# 5. Format code
black amplifier_module_hooks_activity_tracker tests
ruff check amplifier_module_hooks_activity_tracker tests

# 6. Type checking
mypy amplifier_module_hooks_activity_tracker

# 7. Commit (pre-commit hooks run automatically)
git add .
git commit -m "feat: your feature description"

# 8. Push and create PR
git push origin feature/your-feature
```

### Testing During Development

```bash
# Run specific test
pytest tests/unit/test_analyzer.py::TestAnalyzer::test_find_related_work -v

# Run with debugging
pytest tests/unit/test_analyzer.py --pdb

# Run with print output
pytest tests/unit/test_analyzer.py -s

# Watch mode (reruns on changes)
ptw

# Performance profiling
pytest tests/performance --profile

# Generate coverage report
pytest --cov --cov-report=term-missing
```

---

## Deployment

### Installation Methods

**Method 1: Direct from GitHub (Users)**
```yaml
# In Amplifier profile
hooks:
  - module: hooks-activity-tracker
    source: git+https://github.com/your-org/amplifier-module-hooks-activity-tracker@main
```

**Method 2: Local Development (Developers)**
```yaml
# In Amplifier profile
hooks:
  - module: hooks-activity-tracker
    source: file:///path/to/amplifier-module-hooks-activity-tracker
```

**Method 3: Via Collection (Teams)**
```bash
# Install collection that includes this module
amplifier collection add git+https://github.com/your-org/your-collection@main
```

### Version Management

**Semantic Versioning**: `MAJOR.MINOR.PATCH`
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

**Release Process**:
```bash
# 1. Update version in pyproject.toml
# 2. Update CHANGELOG.md
# 3. Commit changes
git add pyproject.toml CHANGELOG.md
git commit -m "chore: bump version to 1.0.0"

# 4. Tag release
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0

# 5. GitHub Actions builds and publishes
```

---

## Monitoring & Observability

### Logs

**Location**: `~/.amplifier/logs/activity-tracker.log`

**Format**: JSON Lines (structured logging)
```json
{"timestamp": "2025-11-20T13:00:00Z", "level": "INFO", "module": "hooks", "message": "Session started", "session_id": "abc123"}
{"timestamp": "2025-11-20T13:00:01Z", "level": "INFO", "module": "analyzer", "message": "Found 3 related items", "confidence": [0.92, 0.85, 0.78]}
```

### Metrics

**Tracked**:
- Analysis latency (p50, p95, p99)
- Cache hit rate
- LLM API call count
- Error rate
- Session tracking success rate

**Reporting**:
- Optional: Send metrics to coordinator
- Optional: Export to monitoring system

---

## Security Considerations

### API Keys

**Storage**: Environment variables or Amplifier config
```bash
# Environment variable (recommended)
export OPENAI_API_KEY="sk-..."

# Or in ~/.amplifier/settings.yaml
activity:
  embedding:
    api_key: "${OPENAI_API_KEY}"
```

**Never commit**: API keys, tokens, credentials

### Data Privacy

**What's sent to LLM**:
- Session prompts
- Issue titles and descriptions
- Code snippets (in context)

**What's NOT sent**:
- File contents (unless in git status)
- Full repository code
- Credentials or secrets

**User control**: Can disable auto-tracking or use local-only mode

---

## Performance Characteristics

### Target Performance

| Operation | Target | Phase 1 | Phase 2 |
|-----------|--------|---------|---------|
| Session start overhead | <1s | <2s | <1s |
| Context capture | <500ms | <500ms | <500ms |
| Analysis (100 issues) | <5s | <15s | <5s |
| Embedding generation | <2s | N/A | <2s |
| Cache operations | <10ms | N/A | <10ms |
| Session end processing | <3s | <5s | <3s |

### Scalability

**Tested configurations**:
- Single repo: 1-500 open issues
- Multi-repo: 3-5 repos, 50-200 issues each
- Team size: 1-20 developers

**Known limits**:
- LLM context window (typically 100-200k tokens)
- Embedding API rate limits (3000 requests/min)
- SQLite cache size (unlimited, but monitor disk)

---

## Maintenance

### Regular Tasks

**Weekly**:
- Review error logs
- Check cache hit rate
- Monitor performance metrics

**Monthly**:
- Update dependencies
- Review and close stale issues
- Update documentation

**Quarterly**:
- Security audit
- Performance review
- User feedback collection

### Upgrades

**Dependency Updates**:
```bash
# Check for updates
pip list --outdated

# Update specific package
pip install --upgrade openai

# Test after update
pytest
```

**Breaking Changes**:
- Document in CHANGELOG.md
- Provide migration guide
- Bump major version

---

## Troubleshooting

### Common Issues

**Issue**: Module not loading
- Check profile configuration
- Verify module in PATH
- Check logs for errors

**Issue**: LLM analysis failing
- Check API key configured
- Verify network connectivity
- Check rate limits

**Issue**: Cache not working
- Check .amplifier/embeddings_cache.db exists
- Verify write permissions
- Check disk space

**Issue**: Multi-repo not finding issues
- Verify project group configuration
- Check each repo has .amplifier/issues/
- Verify repos are initialized

See `docs/TROUBLESHOOTING.md` for detailed solutions.

---

**Last Updated**: 2025-11-20  
**Next Review**: After Phase 1 completion
