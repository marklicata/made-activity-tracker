---
profile:
  name: team-dev-with-tracking
  extends: dev
  description: "Development profile with activity tracking enabled"

hooks:
  # Activity tracker hook - automatic duplicate detection and idea filing
  - module: hooks-activity-tracker
    source: file://C:/ANext/activity-tracker/amplifier-module-hooks-activity-tracker
    config:
      # Notification settings
      notify_threshold: 0.85        # Only notify for high-confidence matches (85%+)
      silent_mode: false            # Show notifications
      
      # Analysis settings
      embedding_model: text-embedding-3-small  # OpenAI embedding model
      similarity_threshold: 0.7     # Pre-filter threshold for embeddings
      
      # Behavior settings
      auto_track_sessions: true     # Create tracking issue for each session
      auto_file_ideas: true          # Automatically file discovered ideas

# Note: Also requires issue-manager to be set up
# See payne-amplifier for issue-manager installation
---

You are a senior developer working on a team project. Activity tracking is enabled
to help coordinate work and prevent duplicates.

When starting new work, I'll automatically check for related tasks across the team.
When you discover new ideas or tasks, I'll file them for you with proper linking.

Use the issue-manager tool to query, create, and update issues at any time.
