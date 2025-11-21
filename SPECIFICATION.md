# Activity Tracking System for Amplifier - Technical Specification

**Version**: 1.0  
**Date**: 2025-11-20  
**Status**: Design Phase

---

## Executive Summary

An activity tracking system that integrates with Amplifier to help engineering teams coordinate work, prevent duplicate effort, and maintain awareness of ongoing tasks across projects. The system uses LLM-powered analysis to automatically detect related work and file new ideas, with minimal programmer overhead.

**Core Architecture**: Hook + Tool modules built on Beads substrate, with embedding-enhanced LLM analysis for intelligent work matching.

---

## 1. System Overview

### 1.1 Goals

**Primary Objectives**:
- Automatically detect when programmers start work that duplicates or relates to existing work
- Capture new ideas discovered during development without interrupting flow
- Enable search/query of the team's collective idea pool
- Support multi-repo project groups for coordinated work across services
- Minimize manual overhead while maximizing coordination value

**Non-Goals** (Phase 1):
- Real-time team chat/messaging
- Project management workflows (sprints, planning, estimation)
- Time tracking or metrics
- Code review integration

### 1.2 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Amplifier Session                         │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │        amplifier-module-hooks-activity (Hook Module)       │ │
│  │                                                             │ │
│  │  session:start  → Check for related work                   │ │
│  │                → Notify if high-confidence duplicates       │ │
│  │                → Track session in Beads                     │ │
│  │                                                             │ │
│  │  session:end    → Analyze work accomplished                │ │
│  │                → Update issue status                        │ │
│  │                → File discovered ideas                      │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              ↓                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │        amplifier-module-tool-activity (Tool Module)        │ │
│  │                                                             │ │
│  │  activity_search         → Search across project group     │ │
│  │  activity_register       → Manually file ideas             │ │
│  │  activity_update         → Update work status              │ │
│  │  activity_query_related  → Find related work               │ │
│  │  activity_set_group      → Configure project groups        │ │
│  └────────────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            ↓
              ┌─────────────────────────────┐
              │   Activity Analysis Layer    │
              │                              │
              │  • LLM client (same as       │
              │    Amplifier session)        │
              │  • Embedding generator       │
              │    (OpenAI by default)       │
              │  • Embedding cache (SQLite)  │
              │  • Two-phase matching        │
              │    (embeddings → LLM)        │
              └──────────────┬───────────────┘
                             ↓
           ┌─────────────────────────────────────┐
           │           Beads Substrate            │
           │                                      │
           │  • SQLite database (per repo)       │
           │  • JSONL audit log (git-committed)  │
           │  • Auto-sync (5s debounce)          │
           │  • Git hooks (immediate sync)       │
           │  • Multi-repo support (native)      │
           │  • CLI interface (bd command)       │
           └─────────────────────────────────────┘
```

### 1.3 Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Storage substrate | Beads | Proven distributed sync, agent-friendly, maintained |
| Module types | Hook + Tool | Automatic detection + explicit control |
| LLM provider | Same as Amplifier session | Simpler config, consistent behavior |
| Embedding model | OpenAI text-embedding-3-small (configurable) | Good quality/cost balance |
| Sync mechanism | Git push/pull | Simple, no additional infrastructure |
| Notification style | Silent unless high-confidence | Less intrusive |
| Work taxonomy | Beads types (task/bug/feature/epic/chore/idea) | Standard, proven |
| Privacy | No restrictions (user confirmed) | Full transcript analysis enabled |
| Multi-repo | Project groups | Coordinate across related services |

---

## 2. Component Specifications

### 2.1 Hook Module: `amplifier-module-hooks-activity`

**Repository**: `amplifier-module-hooks-activity`  
**Type**: Hook module  
**Dependencies**: `amplifier-core`, `beads` (CLI)

#### 2.1.1 Responsibilities

- Intercept Amplifier session lifecycle events
- Capture session context (prompt, files, git status)
- Trigger LLM analysis for duplicate detection
- Notify programmer of high-confidence related work
- Track sessions in Beads
- Extract and file new ideas on session end

#### 2.1.2 Hook Registration

```python
# amplifier_module_hooks_activity/__init__.py

async def mount(coordinator, config):
    """Mount hook module"""
    from .hooks import ActivityHook
    
    hook = ActivityHook(config)
    
    # Register lifecycle hooks
    coordinator.on('session:start', hook.on_session_start)
    coordinator.on('session:end', hook.on_session_end)
    
    # Optional: track tool usage patterns
    if config.get('track_tools', False):
        coordinator.on('tool:before', hook.on_tool_before)
    
    return None
```

#### 2.1.3 Session Start Handler

```python
async def on_session_start(self, event_data):
    """
    Triggered when Amplifier session starts
    
    Flow:
    1. Capture session context
    2. Determine project group (if any)
    3. Query open work across group repos
    4. Run two-phase analysis (embeddings + LLM)
    5. If high-confidence duplicates, notify
    6. Create session tracking issue
    """
    context = self._capture_context(event_data)
    group_config = self._get_project_group(context['working_dir'])
    
    # Query work across project group
    open_work = await self._query_group_work(group_config)
    
    # Two-phase analysis
    related = await self.analyzer.find_related_work(context, open_work)
    
    # Notify only if high-confidence (score > 0.85)
    high_confidence = [r for r in related if r['confidence'] > 0.85]
    if high_confidence:
        self._notify_related_work(high_confidence)
    
    # Track session start
    session_issue = await self._create_session_issue(context, group_config)
    self.session_tracker[event_data['session_id']] = session_issue['id']
```

**Context Capture**:
```python
def _capture_context(self, event_data):
    return {
        'session_id': event_data['session_id'],
        'prompt': event_data.get('initial_prompt', ''),
        'working_dir': os.getcwd(),
        'git_status': self._get_git_status(),
        'recent_files': self._get_recent_files(hours=24),
        'timestamp': datetime.utcnow().isoformat()
    }

def _get_git_status(self):
    """Get git status if in git repo"""
    try:
        result = subprocess.run(
            ['git', 'status', '--short'],
            capture_output=True, text=True, timeout=5
        )
        return result.stdout if result.returncode == 0 else None
    except:
        return None
```

#### 2.1.4 Session End Handler

```python
async def on_session_end(self, event_data):
    """
    Triggered when session ends or context is compacted
    
    Flow:
    1. Analyze session transcript
    2. Extract: completed work, new ideas, issues
    3. Update session issue status
    4. File new ideas with discovered-from links
    5. Update related issues if needed
    """
    session_id = event_data['session_id']
    session_issue_id = self.session_tracker.get(session_id)
    
    if not session_issue_id:
        return  # No tracking issue created
    
    # Analyze transcript
    messages = event_data.get('messages', [])
    analysis = await self.analyzer.analyze_session_work(messages)
    
    # Update session issue
    if analysis['completed']:
        await self._close_issue(session_issue_id, analysis['summary'])
    else:
        await self._update_issue(session_issue_id, 
                                  status='open',
                                  notes=analysis['summary'])
    
    # File new ideas
    group_config = self._get_project_group_for_session(session_id)
    for idea in analysis['new_ideas']:
        target_repo = await self._determine_target_repo(idea, group_config)
        await self._file_idea(idea, target_repo, 
                              discovered_from=session_issue_id)
    
    # Cleanup
    del self.session_tracker[session_id]
```

#### 2.1.5 Configuration Schema

```python
# In profile YAML
hooks:
  - module: hooks-activity
    source: git+https://github.com/your-org/amplifier-module-hooks-activity@main
    config:
      # Beads CLI path
      beads_cli: "bd"  # Default: 'bd' (assumes in PATH)
      
      # Notification settings
      notify_threshold: 0.85  # Confidence score to trigger notification
      silent_mode: false      # True = never notify, just log
      
      # Analysis settings
      embedding_model: "text-embedding-3-small"  # OpenAI model
      similarity_threshold: 0.7  # Pre-filter threshold
      
      # Session tracking
      auto_track_sessions: true  # Create issue per session
      auto_file_ideas: true      # File ideas on session end
      
      # Optional features
      track_tools: false  # Track tool usage patterns
```

### 2.2 Tool Module: `amplifier-module-tool-activity`

**Repository**: `amplifier-module-tool-activity`  
**Type**: Tool module  
**Dependencies**: `amplifier-core`, `beads` (CLI)

#### 2.2.1 Tool Schema

```python
TOOLS = [
    {
        "name": "activity_search",
        "description": "Search for work items, ideas, or tasks across project group",
        "input_schema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search terms (title/description)"
                },
                "status": {
                    "type": "string",
                    "enum": ["open", "in_progress", "blocked", "closed", "idea"],
                    "description": "Filter by status"
                },
                "repo": {
                    "type": "string",
                    "description": "Specific repo path (optional, searches group if omitted)"
                },
                "include_closed": {
                    "type": "boolean",
                    "description": "Include closed issues (default: false)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results (default: 10)"
                }
            },
            "required": ["query"]
        }
    },
    {
        "name": "activity_register",
        "description": "Register a new idea, task, or bug",
        "input_schema": {
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Short title"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description"
                },
                "type": {
                    "type": "string",
                    "enum": ["task", "bug", "feature", "epic", "chore", "idea"],
                    "description": "Type of work item (default: idea)"
                },
                "priority": {
                    "type": "integer",
                    "description": "Priority 0-4, 0=highest (default: 2)"
                },
                "repo": {
                    "type": "string",
                    "description": "Target repo (default: current)"
                },
                "discovered_from": {
                    "type": "string",
                    "description": "Parent issue ID if discovered during other work"
                }
            },
            "required": ["title"]
        }
    },
    {
        "name": "activity_update",
        "description": "Update work item status or details",
        "input_schema": {
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Issue ID (e.g., bd-a1b2)"
                },
                "status": {
                    "type": "string",
                    "enum": ["open", "in_progress", "blocked", "closed"],
                    "description": "New status"
                },
                "notes": {
                    "type": "string",
                    "description": "Additional notes"
                },
                "assignee": {
                    "type": "string",
                    "description": "Assign to user"
                }
            },
            "required": ["id"]
        }
    },
    {
        "name": "activity_query_related",
        "description": "Find work related to current context or specific work item",
        "input_schema": {
            "type": "object",
            "properties": {
                "context": {
                    "type": "string",
                    "description": "Description of current work or question"
                },
                "issue_id": {
                    "type": "string",
                    "description": "Find work related to this issue"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results (default: 5)"
                }
            },
            "required": []
        }
    },
    {
        "name": "activity_set_group",
        "description": "Configure project group for multi-repo tracking",
        "input_schema": {
            "type": "object",
            "properties": {
                "group_name": {
                    "type": "string",
                    "description": "Project group name"
                },
                "repos": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of repo paths in group"
                },
                "description": {
                    "type": "string",
                    "description": "Group description"
                },
                "set_current": {
                    "type": "boolean",
                    "description": "Set as current group (default: true)"
                }
            },
            "required": ["group_name", "repos"]
        }
    },
    {
        "name": "activity_list_groups",
        "description": "List configured project groups",
        "input_schema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    }
]
```

#### 2.2.2 Implementation Pattern

```python
class ActivityTool(Tool):
    """Activity tracking tool for Amplifier"""
    
    def __init__(self, config):
        self.config = config
        self.beads = BeadsClient(config.get('beads_cli', 'bd'))
        self.analyzer = ActivityAnalyzer(config)
        self.group_manager = ProjectGroupManager(config)
    
    async def execute(self, name, arguments):
        """Route to specific tool implementation"""
        handlers = {
            'activity_search': self._search,
            'activity_register': self._register,
            'activity_update': self._update,
            'activity_query_related': self._query_related,
            'activity_set_group': self._set_group,
            'activity_list_groups': self._list_groups
        }
        
        handler = handlers.get(name)
        if not handler:
            return {"error": f"Unknown tool: {name}"}
        
        try:
            return await handler(**arguments)
        except Exception as e:
            return {"error": str(e), "tool": name}
```

### 2.3 Analysis Layer: `activity_analyzer.py`

**Responsibility**: LLM and embedding-based work comparison

#### 2.3.1 Core Class

```python
class ActivityAnalyzer:
    """
    Two-phase analysis:
    1. Vector embedding pre-filter (fast)
    2. LLM reasoning (accurate)
    """
    
    def __init__(self, config):
        self.config = config
        self.llm = self._init_llm_client()  # Same as Amplifier session
        self.embeddings = EmbeddingGenerator(config)
        self.cache = EmbeddingCache()  # SQLite
    
    async def find_related_work(self, session_context, open_work):
        """
        Find work items related to session context
        
        Returns: List of {issue_id, title, confidence, reasoning}
        """
        # Phase 1: Embedding similarity (fast pre-filter)
        context_text = self._format_context(session_context)
        context_embedding = await self.embeddings.generate(context_text)
        
        candidates = []
        for work_item in open_work:
            work_embedding = await self._get_embedding_cached(work_item)
            similarity = self._cosine_similarity(context_embedding, work_embedding)
            
            if similarity > self.config.get('similarity_threshold', 0.7):
                candidates.append({
                    'work_item': work_item,
                    'similarity': similarity
                })
        
        if not candidates:
            return []
        
        # Sort by similarity, take top N
        candidates.sort(key=lambda x: x['similarity'], reverse=True)
        top_candidates = candidates[:10]
        
        # Phase 2: LLM reasoning (accurate classification)
        prompt = self._build_analysis_prompt(session_context, top_candidates)
        result = await self.llm.generate(prompt, format='json')
        
        return self._parse_llm_result(result)
    
    async def analyze_session_work(self, messages):
        """
        Analyze session transcript to extract work done and ideas
        
        Returns: {
            completed: bool,
            summary: str,
            new_ideas: [{title, description, priority}]
        }
        """
        # Take last 30 messages for analysis
        recent_messages = messages[-30:]
        
        prompt = f"""
        Analyze this Amplifier coding session transcript and extract:
        
        1. Was the main task completed? (yes/no)
        2. Brief summary of work accomplished
        3. New ideas, tasks, or bugs discovered during the session
        
        For each new idea, provide:
        - title (short, clear)
        - description (detailed context)
        - suggested_priority (0-4, based on urgency/importance)
        - suggested_repo (if multi-repo context is evident)
        
        Session transcript:
        {self._format_messages(recent_messages)}
        
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
        """Build prompt for LLM relationship analysis"""
        return f"""
        A programmer is starting an Amplifier session with this context:
        
        **Prompt**: {context['prompt']}
        **Working directory**: {context['working_dir']}
        **Git status**: {context.get('git_status', 'Not a git repo')}
        **Recent files**: {', '.join(context.get('recent_files', [])[:10])}
        
        Here are potentially related open work items (pre-filtered by embeddings):
        
        {self._format_candidates(candidates)}
        
        Determine which work items are ACTUALLY related to what this programmer
        is about to do. Be conservative - only flag:
        1. True duplicates (same work, different words)
        2. Strong blockers (can't proceed without this)
        3. Close collaboration opportunities
        
        For each related item, provide:
        - issue_id
        - confidence (0.0-1.0, where 1.0 = definitely duplicate)
        - reasoning (why it's related)
        - relationship_type (duplicate | blocker | collaboration)
        
        Return JSON:
        {{
            "related": [
                {{
                    "issue_id": "bd-a1b2",
                    "confidence": 0.9,
                    "reasoning": "...",
                    "relationship_type": "duplicate"
                }}
            ]
        }}
        
        If nothing is truly related, return: {{"related": []}}
        """
```

#### 2.3.2 Embedding Cache

```python
class EmbeddingCache:
    """SQLite cache for embeddings to avoid re-generation"""
    
    def __init__(self):
        self.db_path = self._get_cache_path()
        self._init_db()
    
    def _get_cache_path(self):
        """Store in .beads/embeddings_cache.db (gitignored)"""
        return Path.cwd() / '.beads' / 'embeddings_cache.db'
    
    def _init_db(self):
        """Create cache table if not exists"""
        conn = sqlite3.connect(self.db_path)
        conn.execute("""
            CREATE TABLE IF NOT EXISTS embeddings (
                issue_id TEXT PRIMARY KEY,
                embedding BLOB,
                model TEXT,
                content_hash TEXT,
                created_at TIMESTAMP
            )
        """)
        conn.commit()
        conn.close()
    
    async def get(self, issue_id, content_hash):
        """Get cached embedding if exists and content unchanged"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.execute(
            "SELECT embedding FROM embeddings WHERE issue_id = ? AND content_hash = ?",
            (issue_id, content_hash)
        )
        row = cursor.fetchone()
        conn.close()
        
        if row:
            return pickle.loads(row[0])
        return None
    
    async def set(self, issue_id, embedding, model, content_hash):
        """Store embedding in cache"""
        conn = sqlite3.connect(self.db_path)
        conn.execute(
            """INSERT OR REPLACE INTO embeddings 
               (issue_id, embedding, model, content_hash, created_at)
               VALUES (?, ?, ?, ?, ?)""",
            (issue_id, pickle.dumps(embedding), model, content_hash, datetime.utcnow())
        )
        conn.commit()
        conn.close()
```

### 2.4 Project Group Manager

```python
class ProjectGroupManager:
    """Manage multi-repo project groups"""
    
    def __init__(self, config):
        self.config = config
        self.config_path = self._get_config_path()
        self.groups = self._load_groups()
    
    def _get_config_path(self):
        """
        Load from .amplifier/settings.yaml (project) or
        ~/.amplifier/settings.yaml (user)
        """
        project_config = Path.cwd() / '.amplifier' / 'settings.yaml'
        user_config = Path.home() / '.amplifier' / 'settings.yaml'
        
        if project_config.exists():
            return project_config
        return user_config
    
    def get_group_for_repo(self, repo_path):
        """Determine which group (if any) this repo belongs to"""
        repo_path = Path(repo_path).resolve()
        
        for group_name, group_data in self.groups.items():
            for group_repo in group_data['repos']:
                if Path(group_repo).resolve() == repo_path:
                    return group_name, group_data
        
        return None, None
    
    def get_group(self, group_name):
        """Get group configuration"""
        return self.groups.get(group_name)
    
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
    
    def _save_groups(self):
        """Persist groups to config file"""
        # Load existing config
        if self.config_path.exists():
            with open(self.config_path) as f:
                config = yaml.safe_load(f) or {}
        else:
            config = {}
        
        # Update activity section
        if 'activity' not in config:
            config['activity'] = {}
        config['activity']['project_groups'] = self.groups
        
        # Save
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.config_path, 'w') as f:
            yaml.dump(config, f, default_flow_style=False)
```

### 2.5 Beads Client Wrapper

```python
class BeadsClient:
    """Wrapper around bd CLI for Python integration"""
    
    def __init__(self, bd_path='bd', repo_path=None):
        self.bd_path = bd_path
        self.repo_path = repo_path or os.getcwd()
    
    async def list(self, status=None, json=True, **filters):
        """List issues with filters"""
        cmd = [self.bd_path, 'list']
        
        if status:
            cmd.extend(['--status', status])
        if json:
            cmd.append('--json')
        
        result = await self._run_command(cmd)
        return json.loads(result) if json else result
    
    async def create(self, title, description=None, type='idea', priority=2,
                     discovered_from=None, **kwargs):
        """Create new issue"""
        cmd = [
            self.bd_path, 'create', title,
            '-t', type,
            '-p', str(priority),
            '--json'
        ]
        
        if description:
            cmd.extend(['-d', description])
        
        result = await self._run_command(cmd)
        issue_data = json.loads(result)
        
        # Add discovered-from dependency if specified
        if discovered_from:
            await self.add_dependency(issue_data['id'], discovered_from, 
                                     type='discovered-from')
        
        return issue_data
    
    async def update(self, issue_id, status=None, notes=None, **kwargs):
        """Update issue"""
        cmd = [self.bd_path, 'update', issue_id, '--json']
        
        if status:
            cmd.extend(['--status', status])
        
        result = await self._run_command(cmd)
        
        # Add notes if provided
        if notes:
            # bd doesn't have --notes flag, so we append to description
            pass  # TODO: implement via show + update description
        
        return json.loads(result)
    
    async def close(self, issue_id, reason=None):
        """Close issue"""
        cmd = [self.bd_path, 'close', issue_id, '--json']
        if reason:
            cmd.extend(['--reason', reason])
        
        result = await self._run_command(cmd)
        return json.loads(result)
    
    async def add_dependency(self, from_id, to_id, type='related'):
        """Add dependency between issues"""
        cmd = [self.bd_path, 'dep', 'add', from_id, to_id, '--type', type]
        await self._run_command(cmd)
    
    async def _run_command(self, cmd):
        """Run bd command in repo context"""
        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self.repo_path
        )
        stdout, stderr = await proc.communicate()
        
        if proc.returncode != 0:
            raise Exception(f"bd command failed: {stderr.decode()}")
        
        return stdout.decode()
```

---

## 3. Data Model

### 3.1 Beads Schema (Existing)

Used as-is from Beads:

```sql
-- Issues (managed by Beads)
CREATE TABLE issues (
    id TEXT PRIMARY KEY,  -- e.g., bd-a1b2 (hash-based)
    title TEXT NOT NULL,
    description TEXT,
    type TEXT,  -- task | bug | feature | epic | chore | idea
    status TEXT,  -- open | in_progress | blocked | closed
    priority INTEGER,  -- 0-4 (0 = highest)
    assignee TEXT,
    labels TEXT,  -- JSON array
    source_repo TEXT,  -- For multi-repo tracking
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    closed_at TIMESTAMP
);

-- Dependencies (managed by Beads)
CREATE TABLE dependencies (
    from_id TEXT,
    to_id TEXT,
    type TEXT,  -- blocks | related | parent-child | discovered-from
    created_at TIMESTAMP,
    PRIMARY KEY (from_id, to_id),
    FOREIGN KEY (from_id) REFERENCES issues(id),
    FOREIGN KEY (to_id) REFERENCES issues(id)
);
```

### 3.2 Activity Extensions

New tables in `.beads/*.db` (per-repo):

```sql
-- Embedding cache (for performance)
CREATE TABLE activity_embeddings (
    issue_id TEXT PRIMARY KEY,
    embedding BLOB,  -- Pickled numpy array
    model TEXT,  -- e.g., 'text-embedding-3-small'
    content_hash TEXT,  -- Hash of title + description
    created_at TIMESTAMP,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

CREATE INDEX idx_embeddings_model ON activity_embeddings(model);
CREATE INDEX idx_embeddings_hash ON activity_embeddings(content_hash);

-- Session tracking
CREATE TABLE activity_sessions (
    session_id TEXT PRIMARY KEY,  -- Amplifier session ID
    issue_id TEXT,  -- Linked Beads issue
    initial_prompt TEXT,
    working_dir TEXT,
    git_status TEXT,
    started_at TIMESTAMP,
    ended_at TIMESTAMP,
    work_summary TEXT,
    FOREIGN KEY (issue_id) REFERENCES issues(id)
);

CREATE INDEX idx_sessions_issue ON activity_sessions(issue_id);
CREATE INDEX idx_sessions_started ON activity_sessions(started_at);

-- Project group metadata (stored in user/project config, not DB)
-- See ProjectGroupManager for config-based storage
```

### 3.3 Configuration Storage

Project groups stored in `.amplifier/settings.yaml` or `~/.amplifier/settings.yaml`:

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
        - /Users/dev/code/shared-libs
      description: "Core backend platform"
  
  current_group: auth-system  # Default for this workspace
  
  # Analysis settings
  embedding_model: text-embedding-3-small
  similarity_threshold: 0.7
  notify_threshold: 0.85
```

---

## 4. User Experience Flows

### 4.1 Initial Setup (Per Team Repo)

```bash
# Step 1: Initialize Beads
$ cd /team/auth-service
$ bd init
[Beads] Created .beads/ directory
[Beads] Install git hooks? [Y/n] y
[Beads] Hooks installed

# Step 2: Configure Amplifier with activity tracking
$ amplifier
> Can you set up activity tracking for this project?

[Agent uses activity_set_group tool]
> What other repos are part of this project group?

User: "auth-service, user-service, and auth-frontend"

[Agent configures group]
[Activity Tracker] Created group "auth-system" with 3 repos
[Activity Tracker] Group saved to .amplifier/settings.yaml

> Activity tracking is now enabled. I'll automatically check for 
> related work when you start sessions.
```

### 4.2 Daily Workflow: Starting New Work

```bash
$ cd /team/auth-service
$ amplifier
> I need to implement two-factor authentication

[Activity Tracker] Checking for related work in auth-system group...

[Activity Tracker] Found high-confidence match:
  • bd-x5y6 in user-service: "Add 2FA support" (in_progress, @bob)
    Confidence: 0.92 (duplicate)
    Reasoning: Both tasks involve implementing two-factor authentication
    
[React: Link as related] [Talk to @bob] [Continue separately]

> User selects "Talk to @bob"

[Activity Tracker] Created bd-a1b2 "Implement 2FA" (linked to bd-x5y6)
[Activity Tracker] Status set to: blocked (waiting for @bob)

> I've recorded your intent and linked it to @bob's work. Would you 
> like me to help you with something else while you coordinate?
```

### 4.3 Mid-Session: Discovering New Ideas

```bash
> ... working on authentication ...
> 
> Oh, I noticed we should also add rate limiting to prevent brute force

[Session continues normally - no interruption]

[At session end:]

[Activity Tracker] Analyzing session work...
[Activity Tracker] Filed bd-c2d3 "Add rate limiting to auth endpoints" (idea)
[Activity Tracker] Linked as discovered-from bd-a1b2
[Activity Tracker] Suggested priority: P1 (security-related)
```

### 4.4 Explicit Search

```bash
$ amplifier
> What authentication work is currently open across the team?

[Agent uses activity_search tool with query="authentication" across group]

Found 4 items:
1. bd-a1b2 (auth-service): "Implement 2FA" (in_progress, @you)
2. bd-x5y6 (user-service): "Add 2FA support" (in_progress, @bob)
3. bd-f7g8 (auth-frontend): "2FA UI components" (open, unassigned)
4. bd-c2d3 (auth-service): "Add rate limiting" (idea, unassigned)

> Should I take on the rate limiting task?

[Agent checks dependencies, priorities]
> The rate limiting task (bd-c2d3) is independent and high priority.
> Would you like me to update it to in_progress and assign it to you?
```

### 4.5 Session End: Automatic Updates

```bash
> exit

[Activity Tracker] Analyzing session work...
[Activity Tracker] Detected completed work on bd-a1b2
[Activity Tracker] Closing bd-a1b2: "Implemented 2FA with TOTP support"
[Activity Tracker] Filed 3 new ideas:
  • bd-h8i9: "Add SMS fallback for 2FA" (idea, P2)
  • bd-j9k0: "2FA recovery codes" (idea, P1)
  • bd-k0l1: "Audit log for auth events" (idea, P2)

Session saved. Next sync: git push
```

### 4.6 Multi-Machine Sync

```bash
# Developer's laptop
$ cd /team/auth-service
$ git add .beads/issues.jsonl
$ git commit -m "Added 2FA task and related ideas"
$ git push

# Developer's desktop (later)
$ cd /team/auth-service
$ git pull
[Beads] Auto-importing new issues...
[Beads] Imported 4 new issues

$ amplifier
[Activity Tracker] Welcome back! 4 new items since last session:
  • bd-a1b2: "Implement 2FA" (closed)
  • bd-h8i9: "Add SMS fallback for 2FA" (idea)
  • bd-j9k0: "2FA recovery codes" (idea)
  • bd-k0l1: "Audit log for auth events" (idea)
```

---

## 5. Implementation Phases

### Phase 1: MVP (Week 1-2)

**Goal**: Prove the concept with core functionality

**Deliverables**:

1. `amplifier-module-hooks-activity` (basic)
   - [x] Session start hook
   - [x] Session end hook
   - [x] Context capture (prompt, git status, files)
   - [x] Basic LLM analysis (no embeddings)
   - [x] Simple notification mechanism
   - [x] Beads CLI integration

2. `amplifier-module-tool-activity` (basic)
   - [x] activity_search tool
   - [x] activity_register tool
   - [x] activity_update tool
   - [x] Beads CLI wrapper

3. Documentation
   - [x] Setup guide
   - [x] User guide
   - [x] Configuration reference

**Test Criteria**:
- Can detect obvious duplicate work
- Can file new ideas on session end
- Syncs across 2 machines via git
- Works with single-repo projects

**Estimated Effort**: 40-60 hours

### Phase 2: Enhanced Analysis (Week 3-4)

**Goal**: Improve accuracy and performance

**Deliverables**:

1. Embedding Analysis
   - [x] Embedding generator (OpenAI API)
   - [x] Embedding cache (SQLite)
   - [x] Two-phase matching (embeddings → LLM)
   - [x] Content hash for cache invalidation

2. Improved Prompts
   - [x] Context-aware LLM prompts
   - [x] Few-shot examples
   - [x] Confidence scoring

3. Performance
   - [x] Benchmark analysis speed
   - [x] Optimize for 100+ open issues
   - [x] Async/concurrent processing

**Test Criteria**:
- Analysis completes in <5s
- False positive rate <10%
- Handles 100+ open items
- Cache hit rate >70%

**Estimated Effort**: 30-40 hours

### Phase 3: Multi-Repo Support (Week 5-6)

**Goal**: Project groups and team coordination

**Deliverables**:

1. Project Groups
   - [x] ProjectGroupManager
   - [x] Group configuration (YAML)
   - [x] Cross-repo querying
   - [x] Smart repo selection for new issues
   - [x] activity_set_group tool
   - [x] activity_list_groups tool

2. Multi-Repo Analysis
   - [x] Query all repos in group
   - [x] Aggregate results
   - [x] Repo-aware notifications

3. Documentation
   - [x] Multi-repo setup guide
   - [x] Team coordination patterns

**Test Criteria**:
- Works with 3-5 repo groups
- Can detect duplicates across repos
- Correctly assigns issues to target repo
- Group config persists correctly

**Estimated Effort**: 25-35 hours

### Phase 4: Polish & Production-Ready (Week 7-8)

**Goal**: Robust, well-tested, documented

**Deliverables**:

1. Error Handling
   - [x] Graceful degradation (if Beads unavailable)
   - [x] Retry logic for LLM calls
   - [x] Clear error messages
   - [x] Logging and diagnostics

2. Testing
   - [x] Unit tests for core components
   - [x] Integration tests with Beads
   - [x] Mock LLM for testing
   - [x] Test coverage >80%

3. Configuration
   - [x] Interactive setup wizard
   - [x] Configuration validation
   - [x] Default templates

4. Documentation
   - [x] Complete API reference
   - [x] Troubleshooting guide
   - [x] Architecture documentation
   - [x] Video walkthrough

**Test Criteria**:
- No unhandled exceptions
- Test coverage >80%
- Documentation complete
- Ready for team rollout

**Estimated Effort**: 35-45 hours

---

## 6. Technology Stack

### Core Dependencies

**Hook Module**:
- `amplifier-core` - Amplifier kernel interfaces
- Python 3.11+
- `asyncio` - Async runtime
- `subprocess` - bd CLI integration
- `pyyaml` - Configuration parsing

**Tool Module**:
- `amplifier-core` - Amplifier kernel interfaces
- Python 3.11+
- Same as hook module

**Analysis Layer**:
- `openai` - LLM and embeddings (or compatible client)
- `numpy` - Vector operations
- `sqlite3` (stdlib) - Embedding cache
- `hashlib` (stdlib) - Content hashing

**External**:
- `bd` (Beads CLI) - Storage and sync substrate
- Git - Version control and sync

### Development Tools

- `pytest` - Testing framework
- `pytest-asyncio` - Async test support
- `pytest-mock` - Mocking
- `black` - Code formatting
- `mypy` - Type checking
- `ruff` - Linting

---

## 7. Open Questions & Decisions

### Resolved (User Confirmed)

✓ Use Beads as substrate  
✓ Notification: Silent unless high-confidence (Option B)  
✓ LLM: Same as Amplifier session  
✓ Embeddings: OpenAI text-embedding-3-small (configurable)  
✓ Sync: Git push/pull (no Agent Mail initially)  
✓ Taxonomy: Beads types (task/bug/feature/epic/chore) + add "idea"  
✓ Privacy: No restrictions  
✓ Multi-repo: Project groups (custom implementation)  

### Still Open

1. **Notification UI**: How should we display related work notifications?
   - Option A: Plain text in console
   - Option B: Rich formatting with colors
   - Option C: Interactive prompt with options
   - **Recommendation**: Start with B, add C if needed

2. **Auto-assignment**: Should issues auto-assign to session creator?
   - Option A: Always auto-assign
   - Option B: Never auto-assign (let team decide)
   - Option C: Prompt/configurable
   - **Recommendation**: B (explicit assignment better for teams)

3. **Idea lifecycle**: When do "ideas" transition to "tasks"?
   - Option A: Manual (user promotes idea → task)
   - Option B: Automatic (when assigned or started)
   - Option C: LLM suggests promotions
   - **Recommendation**: A (simple, explicit)

4. **Embedding model fallback**: If OpenAI unavailable?
   - Option A: Fail gracefully (LLM-only mode)
   - Option B: Support local embeddings (sentence-transformers)
   - Option C: Require OpenAI
   - **Recommendation**: A for MVP, B for Phase 2

5. **Session issue titles**: How to title auto-created session issues?
   - Option A: Use initial prompt (up to 50 chars)
   - Option B: LLM generates title
   - Option C: Template: "Session: YYYY-MM-DD HH:MM"
   - **Recommendation**: A (simple, informative)

---

## 8. Risk Assessment

### High Risk

**Beads CLI dependency**
- Risk: Beads command not in PATH or incompatible version
- Mitigation: Check bd availability on mount, clear error messages
- Fallback: Fail gracefully, suggest installation

**LLM API costs**
- Risk: High volume of analysis calls = expensive
- Mitigation: Aggressive embedding pre-filter, cache results
- Monitoring: Track API usage, add cost alerts

**False positives**
- Risk: Too many irrelevant notifications annoy users
- Mitigation: High confidence threshold (0.85), user feedback
- Tuning: Adjust thresholds based on user feedback

### Medium Risk

**Git merge conflicts**
- Risk: Multiple developers create issues simultaneously
- Mitigation: Beads uses hash-based IDs (collision-resistant)
- Fallback: bd merge driver handles JSONL conflicts

**Multi-repo complexity**
- Risk: Project groups add configuration complexity
- Mitigation: Simple YAML config, good defaults
- Documentation: Clear setup guides

### Low Risk

**Embedding cache staleness**
- Risk: Cached embeddings don't reflect updated issues
- Mitigation: Content hashing, regenerate on cache miss
- Acceptable: Slight staleness (will catch on next analysis)

**Session tracking overhead**
- Risk: Creating/updating issues adds latency
- Mitigation: Async operations, don't block session start
- Acceptable: <1s overhead

---

## 9. Success Metrics

### Phase 1 (MVP)

- [ ] Successfully detects duplicate work in >80% of obvious cases
- [ ] Files new ideas automatically with >90% accuracy
- [ ] Syncs across machines with <5 minute lag (git push/pull)
- [ ] Zero unhandled exceptions in normal operation
- [ ] Setup time <10 minutes for new team member

### Phase 2 (Enhanced)

- [ ] Analysis completes in <5 seconds for 100 open issues
- [ ] False positive rate <10% (user feedback)
- [ ] Embedding cache hit rate >70%
- [ ] User satisfaction: 7/10 or higher

### Phase 3 (Multi-Repo)

- [ ] Successfully tracks work across 3-5 repo groups
- [ ] Cross-repo duplicate detection >80% accurate
- [ ] Group configuration <5 minutes per group
- [ ] No performance degradation with multiple repos

### Phase 4 (Production)

- [ ] Test coverage >80%
- [ ] Zero critical bugs in 1 week of team usage
- [ ] Documentation completeness: 9/10
- [ ] Ready for external team adoption

---

## 10. Next Steps

### Immediate (Now)

1. **Review this specification**
   - Confirm design decisions
   - Resolve open questions
   - Prioritize phase 2-4 features

2. **Environment setup**
   - Install Beads (`bd init`)
   - Set up development workspace
   - Create test repositories

3. **Create implementation task list**
   - Break Phase 1 into detailed tasks
   - Estimate effort per task
   - Set up project tracking (maybe in Beads!)

### Week 1 (MVP Start)

1. Scaffold both modules
2. Implement Beads CLI wrapper
3. Basic session start/end hooks
4. Simple LLM analysis (no embeddings)
5. Test with single repo

### Week 2 (MVP Complete)

1. Tool module implementation
2. Basic notification mechanism
3. Session tracking
4. Integration testing
5. Documentation

---

## Appendix A: Example Configuration

### Profile Configuration

```yaml
# ~/.amplifier/profiles/team-dev.md
---
profile:
  name: team-dev
  extends: dev
  description: "Team development with activity tracking"

hooks:
  - module: hooks-activity
    source: git+https://github.com/your-org/amplifier-module-hooks-activity@main
    config:
      beads_cli: bd
      notify_threshold: 0.85
      silent_mode: false
      embedding_model: text-embedding-3-small
      similarity_threshold: 0.7
      auto_track_sessions: true
      auto_file_ideas: true
      track_tools: false

tools:
  - module: tool-activity
    source: git+https://github.com/your-org/amplifier-module-tool-activity@main
    config:
      beads_cli: bd
---

You are a senior developer working on a team project. Activity tracking is enabled
to help coordinate work and prevent duplicates.

When starting new work, I'll automatically check for related tasks across the team.
When you discover new ideas or tasks, I'll file them for you.
```

### Settings Configuration

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
  
  # Analysis configuration
  embedding_model: text-embedding-3-small
  similarity_threshold: 0.7
  notify_threshold: 0.85
  
  # Notification preferences
  silent_mode: false
  notification_style: rich  # plain | rich | interactive
```

---

**End of Specification**

This document serves as the complete technical specification for the Activity Tracking System. Once approved, it will guide the implementation in Phase 1-4.
