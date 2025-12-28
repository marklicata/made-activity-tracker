"""Test loading the bundle."""
import asyncio
from pathlib import Path
from amplifier_foundation import load_bundle

async def test_bundle():
    print("Testing bundle loading...")
    
    # Get bundle path
    bundle_path = Path(__file__).parent / "bundle.md"
    print(f"Bundle path: {bundle_path}")
    
    if not bundle_path.exists():
        print("✗ bundle.md not found!")
        return False
    
    try:
        # Load bundle
        print("Loading bundle...")
        bundle = await load_bundle(str(bundle_path))
        print(f"✓ Bundle loaded: {bundle.name}")
        
        # Check tools
        print(f"✓ Tools defined: {len(bundle.tools)}")
        for tool in bundle.tools:
            print(f"  - {tool.get('module')}: {tool.get('source')}")
        
        # Check includes
        print(f"✓ Includes: {len(bundle.includes)}")
        
        return True
    except Exception as e:
        print(f"✗ Failed to load bundle: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = asyncio.run(test_bundle())
    exit(0 if success else 1)
