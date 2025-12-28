"""Simple import test."""
print("Testing imports...")

try:
    from made_activity_tools import tool_module
    print("✓ Tool module imported")
except Exception as e:
    print(f"✗ Tool module failed: {e}")
    exit(1)

try:
    from amplifier_foundation import load_bundle
    print("✓ Foundation imported")
except Exception as e:
    print(f"✗ Foundation failed: {e}")
    exit(1)

print("\n✓ All imports successful!")
