# Activity Tracking System v2 - Simplified Specification

**Building on amplifier-module-tool-github**

**Version**: 2.0  
**Date**: 2025-11-20  
**Status**: Design Phase - Simplified

---

## Executive Summary

An activity tracking system that uses GitHub issue tools for LLM-powered duplicate detection and automatic session tracking. By leveraging the GitHub issue tools module, we get issue retrieval capabilities and can focus purely on intelligent analysis.

**Key Insight**: Don't manage storage/CRUD - use existing GitHub tools and focus on AI-powered analysis.

---

## 1. What We're Building

### 1.1 New Components (What We Actually Need to Build)

1. **hooks-activity-tracker** - Session lifecycle integration
2. **Activity Analyzer** - LLM + embedding analysis
3. **Project Group Manager** - Multi-repo coordination

### 1.2 Existing Components (What We Get for Free)

1. ✅ **amplifier-module-tool-github** - GitHub API integration for issues
   - `github_list_issues` - List and filter issues
   - `github_get_issue` - Get issue details
   - `github_create_issue` - Create new issues
   - `github_update_issue` - Update existing issues
   - `github_comment_issue` - Add comments to issues

---

## 2. Architecture

### 2.1 Component Diagram

```
User Session
    ↓
┌─────────────────────────────────────┐
│  hooks-activity-tracker (NEW)       │
│  • on_session_start()               │
│  • on_session_end()                 │
│  • Uses ActivityAnalyzer            │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  ActivityAnalyzer (NEW)              │
│  • find_related_work()               │
│  • analyze_session_work()            │
│  • Uses embeddings + LLM             │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  ProjectGroupManager (NEW)           │
│  • get_group_for_repo()              │
│  • list_groups()                     │
│  • Config-based storage              │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  amplifier-module-tool-github        │
│  ✅ github_list_issues               │
│  ✅ github_get_issue                 │
│  ✅ github_create_issue              │
│  ✅ github_update_issue              │
│  ✅ github_comment_issue             │
└──────────────┬──────────────────────┘
               ↓
┌─────────────────────────────────────┐
│  GitHub API                          │
│  • Issues & comments                 │
│  • Repository access                 │
└──────────────────────────────────────┘
```

### 2.2 Data Flow

**Session Start**:
```
1. User starts Amplifier with prompt
2. hooks-activity-tracker.on_session_start() fires
3. Capture context (prompt, git status, files)
4. Determine project group (if any)
5. Call github_list_issues(state='open') across group repos
6. ActivityAnalyzer.find_related_work() runs two-phase analysis
7. If high-confidence matches (>0.85), notify user
8. Create session tracking issue via github_create_issue
```

**Session End**:
```
1. User exits or session compacts
2. hooks-activity-tracker.on_session_end() fires
3. ActivityAnalyzer.analyze_session_work() extracts ideas
4. Update session issue status via github_update_issue
5. Create new issues for discovered ideas via github_create_issue
6. Add comments linking related work via github_comment_issue
```

---

## 3. Component Specifications

### 3.1 hooks-activity-tracker (NEW)

**Purpose**: Lifecycle integration with Amplifier sessions

**Hook Points**:
- `session:start`
- `session:end`

**Dependencies**:
- `amplifier-module-tool-github` (via coordinator tools)
- `ActivityAnalyzer`
- `ProjectGroupManager`

**Configuration**:
```yaml
hooks:
  - module: hooks-activity-tracker
    source: git+https://github.com/your-org/amplifier-module-hooks-activity-tracker@main
    config:
      notify_threshold: 0.85  # Confidence to trigger notification
      embedding_model: text-embedding-3-small
      similarity_threshold: 0.7
      auto_track_sessions: true
      auto_file_ideas: true
```

**Core Implementation**:
```python
class ActivityTrackerHook:
    def __init__(self, config):
        self.config = config
        self.analyzer = ActivityAnalyzer(config)
        self.groups = ProjectGroupManager(config)
        self.session_issues = {}  # Track session -> issue_id
    
    async def on_session_start(self, event_data):
        """Check for duplicate/related work on session start"""
        # 1. Get issue-manager from coordinator
        issue_manager = event_data['coordinator'].get('issue-manager')
        if not issue_manager:
            return  # Graceful degradation
        
        # 2. Capture context
        context = {
            'prompt': event_data.get('initial_prompt', ''),
            'working_dir': os.getcwd(),
            'git_status': self._get_git_status(),
            'recent_files': self._get_recent_files()
        }
        
        # 3. Determine project group
        group_name, group_config = self.groups.get_group_for_repo(context['working_dir'])
        
        # 4. Query open work across group
        open_work = []
        if group_config:
            for repo in group_config['repos']:
                # issue-manager is per-repo, need to query each
                repo_issues = self._get_issues_for_repo(repo)
                open_work.extend(repo_issues)
        else:
            # Single repo
            open_work = issue_manager.list_issues(status='open')
        
        # 5. LLM analysis
        related = await self.analyzer.find_related_work(context, open_work)
        
        # 6. Notify if high-confidence
        high_conf = [r for r in related if r['confidence'] > self.config['notify_threshold']]
        if high_conf:
            self._notify_related_work(event_data, high_conf)
        
        # 7. Create session tracking issue
        session_issue = issue_manager.create_issue(
            title=f"Session: {context['prompt'][:50]}",
            description=f"Working directory: {context['working_dir']}\\nPrompt: {context['prompt']}",
            issue_type='task',
            metadata={'session_id': event_data['session_id'], 'auto_tracked': True}
        )
        
        self.session_issues[event_data['session_id']] = session_issue.id
    
    async def on_session_end(self, event_data):
        """Analyze work and file ideas on session end"""
        issue_manager = event_data['coordinator'].get('issue-manager')
        if not issue_manager:
            return
        
        session_id = event_data['session_id']
        session_issue_id = self.session_issues.get(session_id)
        
        if not session_issue_id:
            return
        
        # Analyze transcript
        messages = event_data.get('messages', [])
        analysis = await self.analyzer.analyze_session_work(messages)
        
        # Update session issue
        if analysis['completed']:
            issue_manager.close_issue(session_issue_id, reason=analysis['summary'])
        else:
            issue_manager.update_issue(
                session_issue_id,
                status='open',
                description=analysis['summary']
            )
        
        # File new ideas
        for idea in analysis['new_ideas']:
            new_issue = issue_manager.create_issue(
                title=idea['title'],
                description=idea['description'],
                priority=idea.get('suggested_priority', 2),
                issue_type='task',
                discovered_from=session_issue_id
            )
            
            # Add discovered-from dependency
            issue_manager.add_dependency(
                new_issue.id,
                session_issue_id,
                dep_type='discovered-from'
            )
        
        # Cleanup
        del self.session_issues[session_id]
```

### 3.2 ActivityAnalyzer (NEW)

**Purpose**: LLM-powered work analysis and matching

**Dependencies**:
- LLM client (same as Amplifier session)
- OpenAI embeddings API (configurable)

**Core Methods**:

```python
class ActivityAnalyzer:
    def __init__(self, config):
        self.config = config
        self.llm = self._init_llm_client()
        self.embedding_model = config.get('embedding_model', 'text-embedding-3-small')
        self.cache = EmbeddingCache()
    
    async def find_related_work(self, context, open_issues):
        """
        Two-phase analysis: embeddings pre-filter → LLM reasoning
        
        Args:
            context: Dict with prompt, working_dir, git_status, recent_files
            open_issues: List of Issue objects from issue-manager
        
        Returns:
            List of {issue: Issue, confidence: float, reasoning: str, type: str}
        """
        # Phase 1: Embedding similarity
        context_text = f"{context['prompt']} {context.get('git_status', '')}"
        context_embedding = await self._generate_embedding(context_text)
        
        candidates = []
        for issue in open_issues:
            issue_text = f"{issue.title} {issue.description}"
            issue_embedding = await self._get_cached_embedding(issue.id, issue_text)
            
            similarity = self._cosine_similarity(context_embedding, issue_embedding)
            if similarity > self.config.get('similarity_threshold', 0.7):
                candidates.append({'issue': issue, 'similarity': similarity})
        
        if not candidates:
            return []
        
        # Sort and take top 10
        candidates.sort(key=lambda x: x['similarity'], reverse=True)
        top_candidates = candidates[:10]
        
        # Phase 2: LLM reasoning
        prompt = self._build_analysis_prompt(context, top_candidates)
        result = await self.llm.generate(prompt, format='json')
        
        return self._parse_llm_result(result, top_candidates)
    
    async def analyze_session_work(self, messages):
        """
        Analyze session transcript to extract work done and new ideas
        
        Args:
            messages: List of message dicts from session
        
        Returns:
            {
                completed: bool,
                summary: str,
                new_ideas: [
                    {title: str, description: str, suggested_priority: int}
                ]
            }
        """
        recent = messages[-30:]  # Last 30 messages
        
        prompt = f"""
        Analyze this Amplifier session and extract:
        
        1. Was the main task completed? (yes/no)
        2. Brief summary of work done
        3. New ideas/tasks discovered during session
        
        For each idea, provide:
        - title (short, clear)
        - description (context and details)
        - suggested_priority (0-4, based on urgency)
        
        Session:
        {self._format_messages(recent)}
        
        Return JSON:
        {{
            "completed": true/false,
            "summary": "...",
            "new_ideas": [
                {{"title": "...", "description": "...", "suggested_priority": 2}}
            ]
        }}
        """
        
        result = await self.llm.generate(prompt, format='json')
        return json.loads(result)
    
    def _build_analysis_prompt(self, context, candidates):
        return f"""
        Programmer starting session:
        Prompt: {context['prompt']}
        Working directory: {context['working_dir']}
        
        Potentially related open work (pre-filtered by embeddings):
        {self._format_candidates(candidates)}
        
        Determine which items are ACTUALLY related. Be conservative - only flag:
        1. Duplicates (same work, different words)
        2. Strong blockers
        3. Close collaboration opportunities
        
        For each related item:
        - issue_id
        - confidence (0.0-1.0)
        - reasoning
        - relationship_type (duplicate | blocker | collaboration)
        
        Return JSON:
        {{
            "related": [
                {{
                    "issue_id": "...",
                    "confidence": 0.9,
                    "reasoning": "...",
                    "relationship_type": "duplicate"
                }}
            ]
        }}
        """
```

### 3.3 ProjectGroupManager (NEW)

**Purpose**: Multi-repo project group coordination

**Storage**: YAML config in `.amplifier/settings.yaml`

```python
class ProjectGroupManager:
    def __init__(self, config):
        self.config = config
        self.config_path = self._find_config_path()
        self.groups = self._load_groups()
    
    def get_group_for_repo(self, repo_path):
        """
        Determine which group (if any) this repo belongs to
        
        Returns: (group_name, group_config) or (None, None)
        """
        repo_path = Path(repo_path).resolve()
        
        for group_name, group_data in self.groups.items():
            for group_repo in group_data['repos']:
                if Path(group_repo).resolve() == repo_path:
                    return group_name, group_data
        
        return None, None
    
    def set_group(self, group_name, repos, description=None):
        """Create or update project group"""
        self.groups[group_name] = {
            'repos': [str(Path(r).resolve()) for r in repos],
            'description': description or f"Project group: {group_name}"
        }
        self._save_groups()
    
    def list_groups(self):
        """List all configured groups"""
        return self.groups
```

### 3.4 Enhanced tool-issue (MODIFY EXISTING)

**Add to existing tool-issue**:

```python
# New operations to add to existing tool

async def search_with_llm(self, query: str, group: str = None):
    """
    Search issues with LLM-powered relevance ranking
    
    Args:
        query: Natural language query
        group: Project group name (optional)
    
    Returns:
        List of relevant issues with reasoning
    """
    # Get issues from group or current repo
    if group:
        group_config = self.groups.get_group(group)
        issues = []
        for repo in group_config['repos']:
            repo_issues = self._get_issues_for_repo(repo)
            issues.extend(repo_issues)
    else:
        issues = self.issue_manager.list_issues()
    
    # LLM analysis
    context = {'prompt': query}
    related = await self.analyzer.find_related_work(context, issues)
    
    return related

async def set_project_group(self, group_name: str, repos: list):
    """Configure project group"""
    self.groups.set_group(group_name, repos)
    return {"status": "success", "group": group_name, "repos": repos}

async def list_project_groups(self):
    """List configured project groups"""
    return self.groups.list_groups()
```

---

## 4. Integration with Existing issue-manager

### 4.1 How We Use issue-manager

**Get instance from coordinator**:
```python
issue_manager = coordinator.get('issue-manager')
```

**Use existing API directly**:
```python
# Create issue
issue = issue_manager.create_issue(
    title="Implement feature",
    description="...",
    priority=1,
    issue_type='feature',
    discovered_from=parent_id
)

# List issues
open_issues = issue_manager.list_issues(status='open')

# Add dependency
issue_manager.add_dependency(from_id, to_id, dep_type='blocks')

# Get ready work
ready = issue_manager.get_ready_issues(limit=10)

# Close issue
issue_manager.close_issue(issue_id, reason="Completed")
```

### 4.2 Multi-Repo Strategy

**Challenge**: issue-manager is per-repo (stores in `.amplifier/issues/` in each repo)

**Solution**: Query multiple instances

```python
def _get_issues_for_repo(self, repo_path):
    """
    Get issues from a specific repo's issue-manager
    
    Note: Each repo has its own .amplifier/issues/ directory
    """
    from amplifier_module_issue_manager import IssueManager
    
    data_dir = Path(repo_path) / '.amplifier' / 'issues'
    if not data_dir.exists():
        return []
    
    # Create temporary instance for this repo
    temp_manager = IssueManager(data_dir)
    return temp_manager.list_issues()
```

**Alternative**: Use metadata to track source repo

```python
# When creating cross-repo issue
issue = issue_manager.create_issue(
    title="...",
    metadata={'source_repo': '/path/to/other/repo'}
)
```

---

## 5. Data Models

### 5.1 Existing Models (From issue-manager)

✅ **Issue** - Already defined in issue-manager:
```python
@dataclass
class Issue:
    id: str
    title: str
    description: str
    status: str  # open|in_progress|blocked|closed
    priority: int  # 0-4
    issue_type: str  # bug|feature|task|epic|chore
    assignee: str | None
    created_at: datetime
    updated_at: datetime
    closed_at: datetime | None
    parent_id: str | None
    discovered_from: str | None  # Perfect for our use case!
    blocking_notes: str | None
    metadata: dict[str, Any]  # Can store session_id, auto_tracked, etc.
```

✅ **Dependency** - Already defined:
```python
@dataclass
class Dependency:
    from_id: str
    to_id: str
    dep_type: str  # blocks|related|parent-child|discovered-from
    created_at: datetime
```

✅ **IssueEvent** - Already defined for observability

### 5.2 New Models (What We Add)

**EmbeddingCache** (SQLite):
```sql
CREATE TABLE embeddings (
    issue_id TEXT PRIMARY KEY,
    embedding BLOB,  -- Pickled numpy array
    content_hash TEXT,
    model TEXT,
    created_at TIMESTAMP
);
```

**Project Groups** (YAML config):
```yaml
activity:
  project_groups:
    group-name:
      repos: [list of paths]
      description: string
```

---

## 6. Implementation Phases

### Phase 1: MVP (1-2 weeks, ~25 hours)

**Goal**: Basic automatic tracking with simple LLM analysis

**Tasks**:
1. Create hooks-activity-tracker module skeleton
   - Session start/end hooks
   - Context capture
   - Basic LLM analysis (no embeddings yet)
   - Notification mechanism
   
2. Integrate with existing issue-manager
   - Get instance from coordinator
   - Use CRUD operations
   - Test dependency linking

3. Simple ActivityAnalyzer
   - LLM-only analysis (no embeddings)
   - Basic prompts
   - JSON parsing

4. Documentation
   - Setup guide
   - Configuration reference

**Deliverables**:
- Working hook module
- Can detect obvious duplicates
- Can file ideas on session end
- Integrates with existing issue-manager

**Success Criteria**:
- Detects duplicates >80% of time (obvious cases)
- Files ideas automatically
- Zero crashes

### Phase 2: Enhanced Analysis (1 week, ~20 hours)

**Goal**: Add embeddings for speed and accuracy

**Tasks**:
1. Embedding generation
   - OpenAI API integration
   - Content hashing
   
2. Embedding cache
   - SQLite storage
   - Cache hit/miss logic
   
3. Two-phase matching
   - Embeddings pre-filter
   - LLM reasoning on candidates
   
4. Performance testing
   - Benchmark with 100+ issues
   - Optimize if needed

**Deliverables**:
- Fast analysis (<5s)
- Better accuracy
- Embedding cache working

**Success Criteria**:
- Analysis <5s for 100 issues
- False positive rate <10%
- Cache hit rate >70%

### Phase 3: Multi-Repo (1 week, ~15 hours)

**Goal**: Project groups and cross-repo coordination

**Tasks**:
1. ProjectGroupManager
   - Config storage/loading
   - Group resolution
   
2. Multi-repo querying
   - Query multiple issue-manager instances
   - Aggregate results
   
3. Enhanced tool-issue operations
   - search_with_llm
   - set_project_group
   - list_project_groups
   
4. Documentation
   - Multi-repo setup guide

**Deliverables**:
- Project groups working
- Cross-repo duplicate detection
- Enhanced tool operations

**Success Criteria**:
- Works with 3-5 repo groups
- Detects cross-repo duplicates
- Configuration persists

### Phase 4: Polish (3-5 days, ~10 hours)

**Goal**: Production-ready

**Tasks**:
1. Error handling
2. Testing
3. Documentation
4. Performance optimization

**Total Estimated Effort**: 50-70 hours (vs 130-180 with Beads)

---

## 7. Configuration Examples

### Profile Configuration

```yaml
# ~/.amplifier/profiles/team-dev.md
---
profile:
  name: team-dev
  extends: dev

hooks:
  - module: hooks-activity-tracker
    source: git+https://github.com/your-org/amplifier-module-hooks-activity-tracker@main
    config:
      notify_threshold: 0.85
      embedding_model: text-embedding-3-small
      similarity_threshold: 0.7
      auto_track_sessions: true
      auto_file_ideas: true

tools:
  - module: tool-issue
    source: git+https://github.com/payneio/payne-amplifier@main#subdirectory=max_payne_collection/modules/tool-issue
    config:
      data_dir: .amplifier/issues
      auto_create_dir: true
---
```

### Project Group Configuration

```yaml
# .amplifier/settings.yaml (team project)
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

---

## 8. Key Advantages of This Approach

### Simplicity

✅ **60% reduction in code to write** - Leverage existing issue-manager  
✅ **No external dependencies** - Pure Amplifier modules  
✅ **Python only** - No Go toolchain needed  
✅ **Teammate collaboration** - Can work with Paul on enhancements  

### Maintainability

✅ **Single storage system** - issue-manager's JSONL  
✅ **Single data model** - Issue/Dependency/Event  
✅ **Clear separation** - issue-manager = storage, our code = intelligence  
✅ **Regeneratable** - All modules can be rebuilt from specs  

### Functionality

✅ **All core features present** - CRUD, dependencies, ready work, events  
✅ **Battle-tested** - Paul's using it, has tests, defensive I/O  
✅ **Git-friendly** - JSONL format, works with version control  
✅ **Event tracking** - Built-in observability  

---

## 9. Migration Path from Beads (If Needed)

If teams are already using Beads, we can support both:

```python
# Detect storage backend
if (Path('.beads') / 'issues.jsonl').exists():
    # Use Beads adapter
    storage = BeadsAdapter()
elif (Path('.amplifier') / 'issues').exists():
    # Use issue-manager
    storage = coordinator.get('issue-manager')
else:
    # Initialize issue-manager
    storage = setup_issue_manager()
```

**Recommendation**: Start fresh with issue-manager, provide Beads import tool if needed.

---

## 10. Next Steps

1. **Review this simplified spec** - Does this approach make sense?
2. **Collaborate with Paul** - Discuss any enhancements needed to issue-manager
3. **Start Phase 1** - Build hooks-activity-tracker MVP
4. **Test with small team** - Validate approach before scaling

---

**This specification is dramatically simpler than v1 because we're building on solid existing work rather than reinventing the wheel.**
