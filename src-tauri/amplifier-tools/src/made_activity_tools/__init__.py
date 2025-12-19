"""Custom Amplifier tools for MADE Activity Tracker."""

from .metrics_tool import MetricsTool, mount as mount_metrics
from .search_tool import SearchTool, mount as mount_search
from .user_activity_tool import UserActivityTool, mount as mount_user_activity
from .db_connection import set_db_path

__all__ = [
    'MetricsTool',
    'SearchTool',
    'UserActivityTool',
    'set_db_path',
    'mount_metrics',
    'mount_search',
    'mount_user_activity'
]

__version__ = "0.1.0"
