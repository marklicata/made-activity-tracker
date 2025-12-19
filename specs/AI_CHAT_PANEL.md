# Feature Spec: AI Chat Panel

## Overview
Add a collapsible chat interface that allows users to query and explore GitHub activity data using natural language. The AI agent (powered by Amplifier via amplifier-foundation) answers questions about repositories, users, metrics, trends, and patterns across all tracked data.

## Priority
**LOW** - Implement after Phase 3 (Semantic Search Infrastructure) is complete

## Problem Statement
Users currently interact with data through:
- Fixed dashboard views and charts
- Manual filtering (date ranges, repos, squads, users)
- Navigation between different pages
- Reading static metrics

This requires users to:
- Know what filters to apply to answer their questions
- Understand the data model and where information lives
- Navigate between multiple views to piece together insights
- Interpret metrics and trends manually

Users need a way to ask natural questions like:
- "Why did our cycle time increase last week?"
- "Who's working on authentication?"
- "Show me all high-priority bugs assigned to the backend squad"
- "Which repositories have the most unmerged PRs?"
- "What's the correlation between PR size and review time?"

## User Story
As a team lead or developer, I want to ask natural language questions about my team's GitHub activity, so that I can quickly get insights without manually filtering data or navigating between views.

## Requirements

### Functional Requirements

#### 1. UI Components
- **FR-1.1**: Collapsible right sidebar panel (300-400px width when open)
- **FR-1.2**: Toggle button in top navigation bar (chat icon)
- **FR-1.3**: Panel state persisted across sessions (localStorage)
- **FR-1.4**: Resizable panel with drag handle
- **FR-1.5**: Mobile/small screen: full-screen overlay instead of sidebar
- **FR-1.6**: Smooth open/close animation
- **FR-1.7**: Chat history displayed in scrollable area
- **FR-1.8**: Message input field at bottom with send button
- **FR-1.9**: Clear chat history button

#### 2. Context Awareness
- **FR-2.1**: AI knows which page user is currently on (Dashboard, Team View, Project Deep Dive, etc.)
- **FR-2.2**: AI can access current filter state:
  - Active date range
  - Selected repositories
  - Selected squads
  - Selected users
  - Any other active filters
- **FR-2.3**: AI can reference visible data: "Why is this metric red?"
- **FR-2.4**: Context summary shown at top of chat (optional)
- **FR-2.5**: User can explicitly set/change context: "Focus on backend squad only"

#### 3. Query Capabilities
The AI should be able to answer questions about:

**3.1 Metrics & Aggregates**
- **FR-3.1.1**: Speed metrics (cycle time, PR lead time, throughput)
- **FR-3.1.2**: Ease metrics (PR size, review rounds, rework rate)
- **FR-3.1.3**: Quality metrics (bug rate, reopen rate, rejection rate)
- **FR-3.1.4**: Comparisons across time periods: "How does this month compare to last month?"
- **FR-3.1.5**: Trend explanations: "Why is this going up/down?"

**3.2 User Activity**
- **FR-3.2.1**: Individual user activity: "What's Sarah working on?"
- **FR-3.2.2**: User comparisons: "Who reviewed the most PRs?"
- **FR-3.2.3**: Collaboration patterns: "Who does John collaborate with most?"
- **FR-3.2.4**: User workload: "Is anyone overloaded?"

**3.3 Repository Insights**
- **FR-3.3.1**: Repository metrics: "Which repo has the longest cycle time?"
- **FR-3.3.2**: Repository activity: "What happened in repo X last week?"
- **FR-3.3.3**: Repository health: "Which repos need attention?"
- **FR-3.3.4**: Repository comparisons: "Compare repos A and B"

**3.4 Issues & PRs**
- **FR-3.4.1**: Search by content: "Find issues about authentication"
- **FR-3.4.2**: Search by metadata: "Show me all open bugs"
- **FR-3.4.3**: Search by state: "Which PRs are stuck in review?"
- **FR-3.4.4**: Search by relationships: "Find PRs reviewed by Alice"
- **FR-3.4.5**: Semantic search: "Find similar issues to #123"

**3.5 Trends & Patterns**
- **FR-3.5.1**: Identify patterns: "What patterns do you see in our data?"
- **FR-3.5.2**: Anomaly detection: "Are there any unusual trends?"
- **FR-3.5.3**: Predictions: "Based on current trends, what's our projected velocity?"
- **FR-3.5.4**: Recommendations: "What should we focus on improving?"

#### 4. Response Formats
- **FR-4.1**: Text explanations with context
- **FR-4.2**: Inline data tables (small datasets)
- **FR-4.3**: Links to relevant pages: "See full details in [Team View](/team)"
- **FR-4.4**: Code snippets for queries (if SQL/technical context needed)
- **FR-4.5**: "No results found" with suggestions for alternative queries
- **FR-4.6**: Clarifying questions when query is ambiguous
- **FR-4.7**: Loading indicator while AI processes query

#### 5. Actions & Interactions
- **FR-5.1**: Click suggested filter: AI can propose filters and apply them to current view
- **FR-5.2**: Navigate to data: "Show me" → navigates to relevant page
- **FR-5.3**: Copy response text
- **FR-5.4**: Share/export chat transcript
- **FR-5.5**: Thumbs up/down feedback on responses
- **FR-5.6**: Follow-up questions maintain conversation context

#### 6. Data Access & Permissions
- **FR-6.1**: AI can query SQLite database (read-only)
- **FR-6.2**: AI can access embeddings for semantic search
- **FR-6.3**: AI respects current user's data scope (no cross-user data leakage if auth is added)
- **FR-6.4**: AI cannot modify data (read-only mode)
- **FR-6.5**: Query timeout limits (max 30s per query)

### Non-Functional Requirements

#### Performance
- **NFR-1.1**: Chat panel opens in <200ms
- **NFR-1.2**: Simple queries return in <2s
- **NFR-1.3**: Complex queries (aggregations, semantic search) return in <10s
- **NFR-1.4**: Show streaming responses for long queries
- **NFR-1.5**: Chat history loads instantly from cache

#### Usability
- **NFR-2.1**: Chat input supports multiline (Shift+Enter for new line)
- **NFR-2.2**: Up arrow recalls previous query for editing
- **NFR-2.3**: Example queries shown when chat is empty
- **NFR-2.4**: Syntax highlighting for technical terms
- **NFR-2.5**: Accessible keyboard navigation

#### Reliability
- **NFR-3.1**: Graceful degradation if AI service unavailable
- **NFR-3.2**: Clear error messages for failed queries
- **NFR-3.3**: Retry mechanism for transient failures
- **NFR-3.4**: Chat history persists across app restarts
- **NFR-3.5**: No data loss if app crashes during conversation

#### Security
- **NFR-4.1**: API keys stored securely (not in frontend)
- **NFR-4.2**: Rate limiting to prevent abuse
- **NFR-4.3**: Input sanitization to prevent injection attacks
- **NFR-4.4**: No sensitive data logged
- **NFR-4.5**: User can disable/opt-out of AI features

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────┐
│                    React Frontend                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Main App Layout                                  │  │
│  │  ┌────────────────┐  ┌─────────────────────────┐│  │
│  │  │                │  │  AI Chat Panel          ││  │
│  │  │  Dashboard/    │  │  ┌───────────────────┐ ││  │
│  │  │  Team View/    │  │  │ Chat Messages     │ ││  │
│  │  │  etc.          │  │  │ (scrollable)      │ ││  │
│  │  │                │  │  └───────────────────┘ ││  │
│  │  │                │  │  ┌───────────────────┐ ││  │
│  │  │                │  │  │ Input Field       │ ││  │
│  │  │                │  │  └───────────────────┘ ││  │
│  │  └────────────────┘  └─────────────────────────┘│  │
│  └──────────────────────────────────────────────────┘  │
│              ↓                        ↑                  │
│         Tauri IPC (invoke commands)                     │
└─────────────────────────────────────────────────────────┘
              ↓                        ↑
┌─────────────────────────────────────────────────────────┐
│                   Rust Backend (Tauri)                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │  AI Service Layer                                 │  │
│  │  - Message handling                               │  │
│  │  - Context building                               │  │
│  │  - Query routing                                  │  │
│  └──────────────────────────────────────────────────┘  │
│              ↓                        ↑                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Query Engine                                     │  │
│  │  - SQL query builder                              │  │
│  │  - Semantic search (embeddings)                   │  │
│  │  - Aggregation logic                              │  │
│  └──────────────────────────────────────────────────┘  │
│              ↓                        ↑                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Database Access                                  │  │
│  │  - SQLite queries                                 │  │
│  │  - Embedding lookups                              │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
              ↓                        ↑
┌─────────────────────────────────────────────────────────┐
│              External AI Service (Amplifier)             │
│  - Understands user intent                               │
│  - Decides which queries to run                          │
│  - Generates natural language responses                  │
└─────────────────────────────────────────────────────────┘
```

### Component Breakdown

#### Frontend Components
1. **ChatPanel.tsx** - Main sidebar container
2. **ChatMessage.tsx** - Individual message display (user/AI)
3. **ChatInput.tsx** - Message input with send button
4. **ChatContext.tsx** - Context display (current page, filters)
5. **ChatSuggestions.tsx** - Example queries for new users

#### Rust Backend Modules
1. **ai/mod.rs** - Main AI integration module
   - `chat_query()` command - Handles incoming chat messages
   - `get_chat_history()` command - Retrieves past messages
   - `clear_chat_history()` command - Clears conversation

2. **ai/context.rs** - Context building
   - `build_context()` - Assembles current page, filters, visible data
   - `serialize_context()` - Formats context for AI

3. **ai/query_engine.rs** - Query execution
   - `execute_metrics_query()` - Runs metric calculations
   - `execute_search_query()` - Runs text/semantic search
   - `execute_aggregation_query()` - Runs group-by queries

4. **ai/amplifier.rs** - Amplifier integration
   - `send_to_amplifier()` - API calls to Amplifier
   - `stream_response()` - Handle streaming responses
   - `parse_tool_calls()` - Extract tool/function calls from AI

5. **ai/tools.rs** - Tool definitions
   - Define available functions AI can call
   - Map to Rust query functions

### Database Schema Extensions

No new tables required. AI will query existing tables:
- `issues`
- `pull_requests`
- `commits`
- `reviews`
- `users`
- `repositories`
- `embeddings` (Phase 3)

Optional: Add chat history table
```sql
CREATE TABLE chat_messages (
  id INTEGER PRIMARY KEY,
  session_id TEXT NOT NULL,
  role TEXT NOT NULL, -- 'user' or 'assistant'
  content TEXT NOT NULL,
  context TEXT, -- JSON of current app state
  timestamp INTEGER NOT NULL,
  feedback INTEGER -- NULL, 1 (thumbs up), -1 (thumbs down)
);

CREATE INDEX idx_chat_session ON chat_messages(session_id, timestamp);
```

### AI Integration (Amplifier-first)
- **Backend dependency:** Use the `amplifier-foundation` library to manage Amplifier bundles, providers, and session lifecycle.
- **Execution model:** Tauri backend calls a local Amplifier runner (Python/CLI sidecar) over HTTP. The sidecar loads the foundation bundle (`git+https://github.com/microsoft/amplifier-foundation@main`) plus a provider bundle (Anthropic/OpenAI/Azure/Ollama) and prepares a session once, then executes chat turns.
- **API key resolution:** Prefer Anthropic. Resolution order: `ANTHROPIC_API_KEY` → `OPENAI_API_KEY`. If neither is set, prompt the user to supply either key (store securely, never log). Expose a backend setting to override via config/env for headless runs.
- **Context + tools:** For each chat turn, backend sends user message + serialized context + allowed tool list to Amplifier. Amplifier returns messages and tool calls; backend executes tool calls (SQLite, embeddings) and streams responses back.
- **Provider selection:** Default to Anthropic provider config; keep provider bundle path configurable. Model choice is handled by the provider bundle, not by the frontend.
- **Sidecar packaging & IPC:** Package as a Python venv with a small HTTP JSON endpoint (loopback, random high port, authenticated via shared secret). Start/stop lifecycle managed by Tauri backend; stream responses over chunked HTTP/SSE for partials. Avoid unix domain sockets for Windows portability.
- **Offline/guardrails:** If Amplifier is unavailable, fall back to basic keyword/SQL search with a clear error. No multi-provider switcher in UI; only backend configuration.
- **Observability:** Log Amplifier request/response envelopes (without PII) and latency for performance dashboards.

### Tool Definitions for AI

The AI will have access to these tools/functions:

```rust
// Metrics tools
fn get_speed_metrics(date_range, repos, users) -> MetricsResult
fn get_ease_metrics(date_range, repos, users) -> MetricsResult
fn get_quality_metrics(date_range, repos, users) -> MetricsResult
fn compare_metrics(period1, period2) -> ComparisonResult

// Search tools
fn search_issues(query, filters) -> Vec<Issue>
fn search_pull_requests(query, filters) -> Vec<PullRequest>
fn semantic_search(query, limit) -> Vec<SearchResult> // Phase 3

// User tools
fn get_user_activity(username, date_range) -> UserActivity
fn get_user_contributions(username, repos) -> Vec<Contribution>
fn get_collaboration_matrix(usernames) -> CollaborationMatrix

// Repository tools
fn get_repo_metrics(owner, name, date_range) -> RepoMetrics
fn get_repo_contributors(owner, name) -> Vec<User>
fn get_repo_timeline(owner, name, filters) -> Vec<Event>

// Aggregation tools
fn aggregate_by_user(metric, date_range) -> Vec<(User, Value)>
fn aggregate_by_repo(metric, date_range) -> Vec<(Repo, Value)>
fn aggregate_by_time(metric, granularity) -> Vec<(Time, Value)>
```

### Context Passing

When user sends a message, backend constructs context:

```json
{
  "current_page": "team_view",
  "filters": {
    "date_range": {
      "start": "2024-11-01",
      "end": "2024-11-30"
    },
    "repositories": ["owner/repo1", "owner/repo2"],
    "squads": ["backend"],
    "users": ["alice", "bob"]
  },
  "visible_data": {
    "user_summaries": [
      {
        "username": "alice",
        "total_prs": 15,
        "total_reviews": 23,
        ...
      }
    ]
  },
  "available_tools": [
    "get_speed_metrics",
    "search_issues",
    ...
  ]
}
```

This context is sent to AI along with user's message.

## Implementation Phases

### Phase 3.5a: Backend Infrastructure (Week 1-2)
- [ ] Create `ai/` module structure
- [ ] Implement context building
- [ ] Implement query engine with tool functions
- [ ] Add chat history table and queries
- [ ] Implement Amplifier API client
- [ ] Create Tauri commands: `chat_query`, `get_chat_history`, `clear_chat_history`
- [ ] Write unit tests for query functions

### Phase 3.5b: Frontend UI (Week 2-3)
- [ ] Create ChatPanel component with collapsible sidebar
- [ ] Create ChatMessage component with styling
- [ ] Create ChatInput component with send functionality
- [ ] Add toggle button to navigation
- [ ] Implement localStorage persistence
- [ ] Add example queries for first-time users
- [ ] Responsive design (mobile overlay)

### Phase 3.5c: Integration & Polish (Week 3-4)
- [ ] Connect frontend to backend commands
- [ ] Implement streaming responses (if supported)
- [ ] Add loading states and error handling
- [ ] Implement feedback mechanism (thumbs up/down)
- [ ] Add "navigate to data" interactions
- [ ] Performance optimization (caching, debouncing)
- [ ] Write integration tests

### Phase 3.5d: Documentation & Settings (Week 4)
- [ ] Add AI settings page (API key configuration)
- [ ] Write user documentation
- [ ] Add example queries documentation
- [ ] Implement rate limiting
- [ ] Add usage analytics (opt-in)
- [ ] Final testing and bug fixes

## User Experience Flow

### Example Interaction 1: Simple Metric Query
```
User: "What's our average cycle time this month?"

AI: "Based on 47 issues closed in November 2024:

     Average Cycle Time: 4.2 business days

     This is 15% better than last month (4.9 days).

     Breakdown by repository:
     - backend-api: 3.1 days
     - frontend-app: 5.8 days
     - mobile-app: 3.9 days

     Would you like to see which issues took longest?"
```

### Example Interaction 2: Semantic Search
```
User: "Find bugs related to authentication"

AI: "I found 8 issues matching 'authentication bugs':

     Open (3):
     - #245: Login fails with SSO (opened 2 days ago)
     - #198: JWT refresh token expired (opened 5 days ago)
     - #187: Password reset email not sent (opened 1 week ago)

     Closed (5):
     - #240: OAuth callback URL mismatch (closed yesterday)
     - #231: Session timeout too short (closed 3 days ago)
     ...

     See all results in [Search](/search?q=authentication+bug)"
```

### Example Interaction 3: User Activity
```
User: "What's Alice been working on?"

AI: "Alice (@alice) has been active in 3 repositories this month:

     backend-api (60% of time):
     - Opened 8 PRs (all merged)
     - Reviewed 15 PRs
     - Closed 4 issues
     - Focus: Authentication refactor, API optimization

     frontend-app (30% of time):
     - Opened 3 PRs (2 merged, 1 in review)
     - Reviewed 8 PRs
     - Focus: Login UI updates

     docs (10% of time):
     - Opened 2 PRs (merged)
     - Updated API documentation

     See full details in [Team View → Alice](/team/alice)"
```

### Example Interaction 4: Trend Analysis
```
User: "Why did our PR review time increase last week?"

AI: "PR review time increased from 18 hours to 31 hours (+72%).

     I identified 3 contributing factors:

     1. Larger PRs (+45% avg lines changed)
        - 3 PRs had >500 lines changed vs usual 200-300

     2. Reviewer availability (-30%)
        - Bob (usually 40% of reviews) was on PTO
        - Only Alice and Charlie reviewing

     3. More back-and-forth (+20% review rounds)
        - Average rounds increased from 1.8 to 2.2

     Recommendation: Consider breaking up large PRs and distributing
     review load more evenly.

     See PR details in [Dashboard](/)"
```

## Testing Strategy

### Unit Tests
- Query engine functions return correct results
- Context building includes all necessary data
- Tool definitions match actual function signatures
- Chat history persistence works correctly

### Integration Tests
- Frontend sends correct commands to backend
- Backend executes queries and returns results
- Amplifier API integration works end-to-end
- Error handling for network failures

### E2E Tests
- User can open chat panel
- User can send message and receive response
- User can navigate to data from AI suggestions
- Chat history persists across sessions
- Panel state (open/closed) persists

### Manual Testing Scenarios
1. Ask various types of questions (metrics, search, users, repos)
2. Test with different page contexts (dashboard, team view, project deep dive)
3. Test with different filter states
4. Test error cases (no data, ambiguous query, service down)
5. Test performance with large datasets

## Success Metrics

### Adoption Metrics
- **Target**: 50% of active users try chat feature within first week
- **Target**: 30% of users use chat at least once per session
- **Target**: Average 5+ queries per user per week

### Satisfaction Metrics
- **Target**: 80%+ thumbs-up rate on AI responses
- **Target**: <10% "I don't understand" or error responses
- **Target**: 70%+ of queries result in user taking action (navigating, filtering, etc.)

### Performance Metrics
- **Target**: 95% of queries return in <5s
- **Target**: <1% query timeout rate
- **Target**: <5% error rate

### Business Impact
- Reduced time to insight (measured via user surveys)
- Increased engagement with advanced features
- Fewer support questions about "how do I find X?"

## Future Enhancements

### Phase 4+
- **Voice input**: Speak queries instead of typing
- **Proactive insights**: AI suggests things to look at based on anomalies
- **Scheduled reports**: "Send me a weekly summary of team activity"
- **Custom queries**: Save and reuse complex queries
- **Export to reports**: Convert chat insights to shareable reports
- **Multi-user chat**: Collaborative analysis sessions
- **Plugin system**: Community-contributed query types
- **Visualizations in chat**: Generate charts inline based on queries
- **Code explanations**: "Explain the metrics calculation for X"
- **What-if analysis**: "What would happen if we reduced PR size by 20%?"

## Open Questions

1. **API Key Management**: Should users provide their own Amplifier API key, or should the app include one?
   - Recommendation: App provides key initially, allow power users to BYO key in settings

2. **Data Privacy**: Should chat transcripts be stored locally or could they be sent to cloud for training?
   - Recommendation: All data stays local, no telemetry unless user explicitly opts in

3. **Offline Mode**: What happens when Amplifier is unreachable?
   - Recommendation: Show clear error, allow user to switch to basic keyword search fallback

4. **Context Window Limits**: How much data to include in context? Full dataset could exceed token limits.
   - Recommendation: Include current filters + summary stats, fetch detailed data via tool calls

5. **Multi-turn Conversations**: Should AI remember previous turns in conversation?
   - Recommendation: Yes, maintain conversation context for natural follow-ups

## Dependencies

### External Libraries
- **Frontend**:
  - No new major dependencies (React already included)
  - Optional: `react-markdown` for formatting AI responses

- **Backend**:
  - `reqwest` or `hyper` for HTTP requests to Amplifier
  - `serde_json` for JSON serialization (already included)
  - Optional: `tokio-stream` for response streaming

### Infrastructure
- Amplifier API access (requires account + API key)
- No additional database changes until chat history feature

### Feature Dependencies
- **Required**: Phase 3 semantic search (for FR-3.4.5)
- **Recommended**: User tracking features (for user-specific queries)
- **Optional**: Historical snapshots (for trend comparisons)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Amplifier API costs too high | Medium | High | Implement caching, rate limiting, allow users to BYO key |
| AI gives incorrect answers | Medium | Medium | Show confidence scores, allow feedback, include "verify" links |
| Users don't understand natural language | Low | Medium | Provide example queries, suggest corrections |
| Performance too slow for large datasets | Medium | Medium | Implement query timeouts, progressive loading, caching |
| Privacy concerns about data leaving local machine | Low | High | Clear documentation, local-only mode option |
| Feature complexity delays other priorities | Medium | Medium | Make it opt-in, can be disabled if problematic |

## Conclusion

The AI Chat Panel is a natural evolution of the MADE Activity Tracker, transforming it from a passive dashboard into an interactive analytics assistant. By leveraging existing infrastructure (embeddings, SQLite, Tauri) and building on Phase 3's semantic search capabilities, this feature can provide significant value with manageable complexity.

The key to success is:
1. **Start simple**: Basic Q&A before advanced features
2. **Leverage existing work**: Build on Phase 3 semantic search
3. **Design for extensibility**: Tool-based architecture allows easy additions
4. **Focus on UX**: Natural language is hard; excellent error handling is critical
5. **Measure impact**: Track usage and satisfaction to validate investment

This feature has the potential to differentiate MADE from other GitHub analytics tools and significantly improve user productivity.
