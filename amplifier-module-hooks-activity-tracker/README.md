# Activity Tracker for Amplifier

**Version**: 1.0.0  
**Status**: Production Ready

Intelligent activity tracking system for Amplifier with LLM-powered duplicate detection and automatic idea filing.

---

## Features

✅ **Automatic Duplicate Detection** - Uses LLM + embeddings to find related work  
✅ **Auto-File Ideas** - Captures ideas discovered during sessions  
✅ **Multi-Repo Support** - Coordinate work across project groups  
✅ **Session Tracking** - Automatic issue creation for each session  
✅ **High Performance** - <5s analysis for 100 issues with embedding cache  
✅ **Minimal Overhead** - <2s impact on session start  

---

## Quick Start

### Installation

Add to your Amplifier profile:

```yaml
# ~/.amplifier/profiles/my-profile.md
---
profile:
  name: my-profile
  extends: dev

hooks:
  - module: hooks-activity-tracker
    source: file:///path/to/amplifier-module-hooks-activity-tracker
    config:
      notify_threshold: 0.85
      embedding_model: text-embedding-3-small
      similarity_threshold: 0.7
      auto_track_sessions: true
      auto_file_ideas: true
---
```

### Prerequisites

1. **OpenAI API Key** (for embeddings and LLM):
   ```bash
   export OPENAI_API_KEY="sk-..."
   ```

2. **issue-manager** module installed and configured:
   ```yaml
   # Also in your profile
   tools:
     - module: tool-issue
       source: git+https://github.com/payneio/payne-amplifier@main#subdirectory=max_payne_collection/modules/tool-issue
   ```

### First Use

Start an Amplifier session:

```bash
amplifier --profile my-profile
> I want to implement user authentication

[Activity Tracker] Checking for related work...
[Activity Tracker] No duplicates found. Creating tracking issue...
[Activity Tracker] Created issue-123

> <work on authentication>
> exit

[Activity Tracker] Analyzing session work...
[Activity Tracker] Filed 2 new ideas:
  • Add rate limiting
  • Implement password recovery
```

---

## Configuration

### Hook Configuration

```yaml
hooks:
  - module: hooks-activity-tracker
    config:
      # Notification settings
      notify_threshold: 0.85        # Confidence threshold (0.0-1.0)
      silent_mode: false            # Suppress all notifications
      
      # Analysis settings
      embedding_model: text-embedding-3-small  # OpenAI model
      similarity_threshold: 0.7     # Embedding pre-filter (0.0-1.0)
      
      # Behavior settings
      auto_track_sessions: true     # Create tracking issue per session
      auto_file_ideas: true          # File discovered ideas automatically
```

### Environment Variables

- `OPENAI_API_KEY` - Required for LLM and embeddings

---

## Multi-Repo Project Groups

Track work across multiple repositories:

```yaml
# .amplifier/settings.yaml
activity:
  project_groups:
    auth-system:
      repos:
        - ~/code/auth-service
        - ~/code/user-service
        - ~/code/auth-frontend
      description: "Authentication system"
    
  current_group: auth-system
```

When you start a session in any repo in the group, the tracker searches all repos for related work.

---

## How It Works

### Session Start

1. Captures context (prompt, git status, recent files)
2. Queries open issues from issue-manager
3. Two-phase analysis:
   - **Phase 1**: Embedding similarity (fast pre-filter)
   - **Phase 2**: LLM reasoning (accurate classification)
4. Notifies if high-confidence duplicates found (>0.85)
5. Creates session tracking issue

### Session End

1. Analyzes session transcript
2. Extracts: completed work, summary, new ideas
3. Updates/closes session tracking issue
4. Files new ideas with `discovered-from` links

---

## Examples

### Example 1: Duplicate Detection

```
$ amplifier
> I want to add OAuth authentication

[Activity Tracker] Found related work:
  • issue-456: "Implement OAuth 2.0" (confidence: 92%, duplicate)
    Reason: Both tasks implement OAuth authentication
    
[Link to issue-456] [Start separately] [Ignore]
```

### Example 2: Idea Filing

```
$ amplifier
> Let's implement user login

... work happens ...

> Oh we should also add rate limiting for security
> exit

[Activity Tracker] Analyzing session...
[Activity Tracker] Filed issue-789: "Add rate limiting"
[Activity Tracker] Linked as discovered-from issue-123
```

### Example 3: Multi-Repo

```
$ cd ~/code/auth-service
$ amplifier
> Need to add JWT tokens

[Activity Tracker] Checking auth-system group (3 repos)...
[Activity Tracker] Found in user-service:
  • issue-321: "JWT token generation" (in_progress, @bob)
  
[Collaborate with @bob] [Start separately]
```

---

## Performance

**Targets** (met in testing):
- Session start overhead: <1s
- Analysis (100 issues): <5s
- Embedding cache hit rate: >70%
- Memory usage: <100MB

**Optimizations**:
- Embedding cache (SQLite)
- Two-phase matching (fast pre-filter)
- Async operations
- Batch API calls

---

## Troubleshooting

### "No related work found" (always)

**Problem**: Embeddings not being generated  
**Solution**: Check `OPENAI_API_KEY` is set

### Session issues not being created

**Problem**: issue-manager not available  
**Solution**: Ensure issue-manager is mounted and `.amplifier/issues/` exists

### Slow analysis (>10s)

**Problem**: Cache not being used  
**Solution**: Check `.amplifier/embeddings_cache.db` is writable

### Multi-repo not working

**Problem**: Project group not configured  
**Solution**: Check `.amplifier/settings.yaml` has correct repo paths

---

## Architecture

```
Session Start/End
       ↓
ActivityTrackerHook
       ↓
ActivityAnalyzer (LLM + Embeddings)
       ↓
issue-manager (Storage)
```

**Components**:
- `hooks.py` - Session lifecycle integration
- `analyzer.py` - LLM and embedding analysis
- `embedding_generator.py` - OpenAI embeddings
- `embedding_cache.py` - SQLite cache
- `project_group_manager.py` - Multi-repo coordination
- `utils.py` - Helper functions

---

## Testing

Run tests:

```bash
cd amplifier-module-hooks-activity-tracker

# All tests
pytest

# Unit tests only
pytest tests/unit

# Integration tests
pytest tests/integration

# With coverage
pytest --cov --cov-report=html
```

**Test Coverage**: >80%

---

## Development

### Setup

```bash
git clone <repo-url>
cd amplifier-module-hooks-activity-tracker

python -m venv venv
source venv/bin/activate  # or venv\Scripts\activate on Windows

pip install -e ".[dev]"
pytest
```

### Code Quality

```bash
# Format
black amplifier_module_hooks_activity_tracker tests

# Lint
ruff check amplifier_module_hooks_activity_tracker

# Type check
mypy amplifier_module_hooks_activity_tracker
```

---

## FAQ

**Q: Do I need Beads?**  
A: No, this uses Paul Payne's issue-manager module instead. Much simpler!

**Q: Does this work with other LLM providers?**  
A: Currently OpenAI only. Azure OpenAI support planned.

**Q: Can I disable auto-tracking?**  
A: Yes, set `auto_track_sessions: false` in config.

**Q: How much does embedding generation cost?**  
A: Very little. ~$0.0001 per issue with text-embedding-3-small.

**Q: Can I use local embeddings?**  
A: Not yet, but planned for future release.

---

## Contributing

This module follows Amplifier's implementation philosophy:
- Ruthless simplicity
- Test behavior, not implementation
- Fail fast and visibly
- Clear error messages

See `SPECIFICATION_V2.md` for architecture details.

---

## License

MIT License

---

## Credits

Built on:
- **Paul Payne's issue-manager** - Storage and CRUD
- **Amplifier** - Modular AI platform
- **OpenAI** - LLM and embeddings

---

## Support

**Issues**: Create issue in repository  
**Documentation**: See `docs/` directory  
**Tests**: See `tests/` directory

---

**Ready to track your team's activity with AI-powered intelligence!**
