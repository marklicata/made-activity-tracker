"""Test server can start (doesn't actually start it, just checks setup)."""
import os
import sys
from pathlib import Path

print("Checking server prerequisites...")

# Check API key
anthropic_key = os.environ.get('ANTHROPIC_API_KEY')
openai_key = os.environ.get('OPENAI_API_KEY')

if anthropic_key:
    print("✓ ANTHROPIC_API_KEY is set")
    provider = "anthropic"
elif openai_key:
    print("✓ OPENAI_API_KEY is set")
    provider = "openai"
else:
    print("✗ No API key set!")
    print("\nPlease set one:")
    print("  PowerShell: $env:ANTHROPIC_API_KEY = 'your-key'")
    print("  CMD: set ANTHROPIC_API_KEY=your-key")
    sys.exit(1)

# Check database
db_path = os.environ.get('DATABASE_PATH')
if db_path:
    if Path(db_path).exists():
        print(f"✓ Database found at {db_path}")
    else:
        print(f"⚠ DATABASE_PATH set but file not found: {db_path}")
else:
    # Check default location
    if os.name == 'nt':
        base = os.environ.get('APPDATA')
        default_path = Path(base) / 'com.made.activity-tracker' / 'activity.db'
    else:
        default_path = Path.home() / '.local' / 'share' / 'com.made.activity-tracker' / 'activity.db'
    
    if default_path.exists():
        print(f"✓ Database found at default location: {default_path}")
    else:
        print(f"⚠ Database not found at default location: {default_path}")
        print("  Server will work but tools will fail without database")

# Check bundle files
bundle_path = Path(__file__).parent / "bundle.md"
provider_path = Path(__file__).parent / "providers" / f"{provider}.yaml"

if bundle_path.exists():
    print(f"✓ bundle.md found")
else:
    print(f"✗ bundle.md not found!")
    sys.exit(1)

if provider_path.exists():
    print(f"✓ Provider config found: {provider_path.name}")
else:
    print(f"✗ Provider config not found: {provider_path}")
    sys.exit(1)

print("\n✓ All prerequisites met! Ready to start server.")
print("\nRun:")
print("  python src/made_activity_tools/server.py")
