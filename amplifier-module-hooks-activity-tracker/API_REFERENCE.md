# Actual API Reference (from code inspection)

## ActivityAnalyzer (analyzer.py)

### Properties
- `llm_client` -> lazy-loads AsyncOpenAI
- `embedding_generator` -> lazy-loads EmbeddingGenerator  
- `embedding_cache` -> lazy-loads EmbeddingCache

### Methods
- `async find_related_work(context: dict, open_issues: list) -> list[dict]`
  - Returns list of dicts with: issue, confidence, reasoning, relationship_type
  - Uses two-phase analysis (embeddings + LLM), falls back to LLM-only
  
- `async analyze_session_work(messages: list[dict]) -> dict`
  - Args: messages list with role/content dicts
  - Returns: dict with `completed` (bool), `summary` (str), `new_ideas` (list)
  
- `_cosine_similarity(vec1, vec2) -> float` (not `_compute_similarity`)
- `async _get_cached_embedding(issue_id: str, content: str) -> np.ndarray | None`

## EmbeddingGenerator (embedding_generator.py)

### Properties
- `client` -> lazy-loads AsyncOpenAI

### Methods
- `async generate(text: str) -> np.ndarray | None`
  - Returns embedding vector or None on failure
  
- `async generate_batch(texts: list[str]) -> list[np.ndarray | None]`
  - Returns list of embeddings (None for failures)
  
- `_validate_embedding(embedding) -> bool`

## ProjectGroupManager (project_group_manager.py)

### Methods
- `get_group_for_repo(repo_path: str) -> tuple[str | None, dict | None]`
  - Returns (group_name, group_config) or (None, None)
  
- `get_group(group_name: str) -> dict | None`
  
- `set_group(group_name: str, repos: list[str], description: str | None = None) -> None`
  - Creates or updates group
  
- `list_groups() -> dict[str, dict]` (NOT a list!)
  
- `delete_group(group_name: str) -> None` (NOT `remove_group`)
  
- `_save_groups() -> None` (NOT `_save_config`)

## __init__.py

### Functions
- `async mount(coordinator, config: dict | None) -> None`
  - Imports from `.hooks import ActivityTrackerHook`
  - Registers session:start and session:end handlers
