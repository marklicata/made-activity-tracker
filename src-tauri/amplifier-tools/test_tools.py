"""Manual test script for tools."""
import asyncio
import os
from made_activity_tools import MetricsTool, SearchTool, UserActivityTool
from made_activity_tools import set_db_path


async def test_metrics():
    """Test metrics tool."""
    print("\n=== Testing MetricsTool ===")
    tool = MetricsTool()

    try:
        result = await tool.execute(
            metric_type="speed",
            start_date="2024-01-01",
            end_date="2024-12-31"
        )
        print("✓ Metrics query successful:")
        print(f"  Result: {result}")
    except Exception as e:
        print(f"✗ Metrics query failed: {e}")


async def test_search():
    """Test search tool."""
    print("\n=== Testing SearchTool ===")
    tool = SearchTool()

    try:
        result = await tool.execute(
            query="sync",
            item_type="issue",
            state="all",
            limit=5
        )
        print("✓ Search query successful:")
        print(f"  Found {result['total']} results")
        if result['results']:
            print(f"  First result: {result['results'][0]['title']}")
    except Exception as e:
        print(f"✗ Search query failed: {e}")


async def test_user_activity():
    """Test user activity tool."""
    print("\n=== Testing UserActivityTool ===")
    tool = UserActivityTool()

    # You'll need to replace with an actual username from your database
    try:
        result = await tool.execute(
            username="test-user",
            start_date="2024-01-01",
            end_date="2024-12-31"
        )
        print("✓ User activity query successful:")
        print(f"  Result: {result}")
    except Exception as e:
        print(f"✗ User activity query failed: {e}")


async def main():
    """Run all tests."""
    print("MADE Activity Tools - Test Suite")
    print("=" * 50)

    # Set database path
    # IMPORTANT: Update this path to your actual database location
    db_path = os.environ.get('DATABASE_PATH')

    if not db_path:
        print("\n⚠ DATABASE_PATH not set. Trying default location...")
        if os.name == 'nt':  # Windows
            base = os.environ.get('APPDATA')
            db_path = os.path.join(base, 'com.made.activity-tracker', 'activity.db')
        else:  # macOS/Linux
            base = os.path.expanduser('~/.local/share')
            db_path = os.path.join(base, 'com.made.activity-tracker', 'activity.db')

    print(f"Database path: {db_path}")

    if not os.path.exists(db_path):
        print(f"\n✗ Database not found at {db_path}")
        print("Please set DATABASE_PATH environment variable or update this script.")
        return

    set_db_path(db_path)
    print("✓ Database found")

    # Run tests
    await test_metrics()
    await test_search()
    await test_user_activity()

    print("\n" + "=" * 50)
    print("Tests complete!")


if __name__ == '__main__':
    asyncio.run(main())
