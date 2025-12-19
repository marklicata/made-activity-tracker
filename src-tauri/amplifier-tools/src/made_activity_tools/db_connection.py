"""Database connection utility for MADE Activity Tracker."""

import sqlite3
import os
from typing import Optional

class DatabaseConnection:
    """Manages SQLite database connection for MADE Activity Tracker."""

    def __init__(self, db_path: Optional[str] = None):
        if db_path is None:
            # Default to user's data directory
            # This path should match where your Tauri app stores the database
            if os.name == 'nt':  # Windows
                base = os.environ.get('APPDATA')
            else:  # macOS/Linux
                base = os.path.expanduser('~/.local/share')

            db_path = os.path.join(base, 'com.made.activity-tracker', 'activity.db')

        self.db_path = db_path
        self._conn = None

    def connect(self) -> sqlite3.Connection:
        """Get or create database connection."""
        if self._conn is None:
            if not os.path.exists(self.db_path):
                raise FileNotFoundError(f"Database not found at {self.db_path}")

            self._conn = sqlite3.connect(self.db_path)
            self._conn.row_factory = sqlite3.Row  # Access columns by name

        return self._conn

    def close(self):
        """Close database connection."""
        if self._conn:
            self._conn.close()
            self._conn = None

    def __enter__(self):
        return self.connect()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


# Global instance that can be configured
db = DatabaseConnection()


def set_db_path(path: str):
    """Configure database path (called by server on startup)."""
    global db
    db = DatabaseConnection(path)
