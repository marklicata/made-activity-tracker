"""Quick test script to verify the new setup works."""

import sys
import os
from pathlib import Path

print("=" * 60)
print("MADE Activity Tools - Setup Verification")
print("=" * 60)

# Test 1: Python version
print("\n1. Python Version:")
print(f"   {sys.version}")
if sys.version_info < (3, 11):
    print("   ⚠ WARNING: Python 3.11+ recommended")

# Test 2: Import tool module
print("\n2. Tool Module Import:")
try:
    from made_activity_tools import tool_module
    print("   ✓ Tool module imported successfully")
    print(f"   ✓ Mount function exists: {hasattr(tool_module, 'mount')}")
    print(f"   ✓ Capabilities defined: {hasattr(tool_module, 'get_metrics')}")
except Exception as e:
    print(f"   ✗ FAILED: {e}")
    sys.exit(1)

# Test 3: Import amplifier-foundation
print("\n3. Amplifier Foundation:")
try:
    from amplifier_foundation import load_bundle, Bundle
    print("   ✓ amplifier-foundation imported")
except Exception as e:
    print(f"   ✗ FAILED: {e}")
    print("   Run: pip install -e .")
    sys.exit(1)

# Test 4: Check bundle.md exists
print("\n4. Bundle Configuration:")
bundle_path = Path(__file__).parent / "bundle.md"
if bundle_path.exists():
    print(f"   ✓ bundle.md found at {bundle_path}")
    with open(bundle_path, 'r') as f:
        content = f.read()
        has_includes = "includes:" in content
        has_tools = "tools:" in content
        print(f"   ✓ Has includes: {has_includes}")
        print(f"   ✓ Has tools: {has_tools}")
else:
    print(f"   ✗ bundle.md not found at {bundle_path}")
    sys.exit(1)

# Test 5: Check provider files
print("\n5. Provider Configuration:")
providers_dir = Path(__file__).parent / "providers"
if providers_dir.exists():
    anthropic = providers_dir / "anthropic.yaml"
    openai = providers_dir / "openai.yaml"
    print(f"   ✓ Providers directory exists")
    print(f"   ✓ anthropic.yaml: {anthropic.exists()}")
    print(f"   ✓ openai.yaml: {openai.exists()}")
else:
    print(f"   ✗ providers/ directory not found")
    sys.exit(1)

# Test 6: Check environment variables
print("\n6. Environment Variables:")
anthropic_key = os.environ.get('ANTHROPIC_API_KEY')
openai_key = os.environ.get('OPENAI_API_KEY')
db_path = os.environ.get('DATABASE_PATH')

has_key = bool(anthropic_key or openai_key)
print(f"   ANTHROPIC_API_KEY: {'✓ Set' if anthropic_key else '✗ Not set'}")
print(f"   OPENAI_API_KEY: {'✓ Set' if openai_key else '✗ Not set'}")
print(f"   DATABASE_PATH: {db_path if db_path else '(using default)'}")

if not has_key:
    print("\n   ⚠ WARNING: No API key set!")
    print("   Set one with:")
    print("   - Windows: $env:ANTHROPIC_API_KEY = 'your-key'")
    print("   - Linux/Mac: export ANTHROPIC_API_KEY='your-key'")

# Test 7: Database check
print("\n7. Database:")
if db_path and Path(db_path).exists():
    print(f"   ✓ Database found at {db_path}")
else:
    # Try default location
    if os.name == 'nt':  # Windows
        base = os.environ.get('APPDATA')
        default_path = Path(base) / 'com.made.activity-tracker' / 'activity.db'
    else:
        base = Path.home() / '.local' / 'share'
        default_path = base / 'com.made.activity-tracker' / 'activity.db'
    
    if default_path.exists():
        print(f"   ✓ Database found at default location: {default_path}")
    else:
        print(f"   ✗ Database not found")
        print(f"   Checked: {db_path if db_path else default_path}")
        print("   Set DATABASE_PATH environment variable if in custom location")

print("\n" + "=" * 60)
print("Setup verification complete!")
print("=" * 60)

if has_key:
    print("\n✓ Ready to test! Run:")
    print("  python src/made_activity_tools/server.py")
else:
    print("\n⚠ Set API key before running server")
