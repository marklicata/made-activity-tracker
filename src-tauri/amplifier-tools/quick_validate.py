"""Quick validation of the mount protocol fix."""
import asyncio
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent / "src"))

async def validate():
    try:
        from amplifier_core.testing import TestCoordinator
        from made_activity_tools.tool_module import mount
        
        coordinator = TestCoordinator()
        cleanup = await mount(coordinator, {})
        
        tools = coordinator.get("tools")
        expected = ["get_metrics", "search_github_items", "get_user_activity"]
        
        print("Validation Results:")
        print("-" * 50)
        all_ok = True
        for tool_name in expected:
            found = tool_name in tools
            print(f"{'✓' if found else '✗'} {tool_name}: {'Found' if found else 'Missing'}")
            all_ok = all_ok and found
        
        print("-" * 50)
        if all_ok:
            print("SUCCESS: All tools mounted correctly!")
            return 0
        else:
            print("FAILED: Some tools missing")
            return 1
    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(asyncio.run(validate()))
