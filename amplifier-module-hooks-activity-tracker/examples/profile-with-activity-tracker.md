---
profile:
  name: team-dev-with-tracking
  extends: dev
  description: "Development profile with activity tracking enabled"

tools:
  # GitHub tools for issue management
  - module: tool-github
    source: git+https://github.com/microsoft/amplifier-module-tool-github@main
    config:
      token: ${GITHUB_TOKEN}  # Set via environment variable

hooks:
  # Activity tracker hook - automatic duplicate detection and idea filing
  - module: hooks-activity-tracker
    source: file://C:/ANext/activity-tracker/amplifier-module-hooks-activity-tracker
    config:
      # Repository settings
      repository: owner/repo        # GitHub repository (required)
      
      # Notification settings
      notify_threshold: 0.85        # Only notify for high-confidence matches (85%+)
      silent_mode: false            # Show notifications
      
      # Analysis settings
      embedding_model: text-embedding-3-small  # OpenAI embedding model
      similarity_threshold: 0.7     # Pre-filter threshold for embeddings
      
      # Behavior settings
      auto_track_sessions: true     # Create tracking issue for each session
      auto_file_ideas: true          # Automatically file discovered ideas

# Note: Requires GitHub token with repo access
# Set GITHUB_TOKEN environment variable before starting
---

You are a senior developer working on a team project. Activity tracking is enabled
to help coordinate work and prevent duplicates.

When starting new work, I'll automatically check for related GitHub issues across the team.
When you discover new ideas or tasks, I'll file them as GitHub issues with proper linking.

Use the GitHub tools (github_list_issues, github_create_issue, etc.) to manage issues at any time.
