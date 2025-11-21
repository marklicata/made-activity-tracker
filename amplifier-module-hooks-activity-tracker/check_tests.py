"""Direct test checker - bypasses bash issues."""

import sys
import os
from pathlib import Path

# Add current directory to path
sys.path.insert(0, str(Path(__file__).parent))

print("="*60)
print("ACTIVITY TRACKER - DIRECT TEST CHECK")
print("="*60)

# 1. Check dependencies
print("\n1. Checking Dependencies...")
print("-" * 40)

dependencies = {
    'pytest': False,
    'pytest_asyncio': False,
    'openai': False,
    'yaml': False,
    'numpy': False,
}

for dep in dependencies:
    try:
        __import__(dep)
        dependencies[dep] = True
        print(f"✓ {dep}")
    except ImportError as e:
        print(f"✗ {dep} - NOT INSTALLED: {e}")

all_deps_installed = all(dependencies.values())

if not all_deps_installed:
    print("\n❌ Missing dependencies!")
    print("\nInstall with:")
    print("  pip install pytest pytest-asyncio pytest-cov openai pyyaml numpy")
    sys.exit(1)

print("\n✓ All dependencies installed")

# 2. Check module imports
print("\n2. Checking Module Imports...")
print("-" * 40)

try:
    from amplifier_module_hooks_activity_tracker import mount
    print("✓ Main module imports")
except ImportError as e:
    print(f"✗ Failed to import main module: {e}")
    sys.exit(1)

try:
    from amplifier_module_hooks_activity_tracker.utils import compute_content_hash
    print("✓ Utils module imports")
except ImportError as e:
    print(f"✗ Failed to import utils: {e}")
    sys.exit(1)

try:
    from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook
    print("✓ Hooks module imports")
except ImportError as e:
    print(f"✗ Failed to import hooks: {e}")
    sys.exit(1)

# 3. Test simple functionality
print("\n3. Testing Basic Functionality...")
print("-" * 40)

try:
    # Test content hashing
    from amplifier_module_hooks_activity_tracker.utils import compute_content_hash
    hash1 = compute_content_hash("test")
    hash2 = compute_content_hash("test")
    hash3 = compute_content_hash("different")
    
    assert hash1 == hash2, "Same input should produce same hash"
    assert hash1 != hash3, "Different input should produce different hash"
    assert len(hash1) == 64, "SHA-256 should produce 64 char hex"
    
    print("✓ Content hashing works correctly")
except Exception as e:
    print(f"✗ Content hashing failed: {e}")
    sys.exit(1)

try:
    # Test notification formatting
    from amplifier_module_hooks_activity_tracker.utils import format_notification
    from unittest.mock import Mock
    
    issue = Mock()
    issue.id = "test-1"
    issue.title = "Test Issue"
    
    notification = format_notification([{
        'issue': issue,
        'confidence': 0.95,
        'reasoning': 'Test reason',
        'relationship_type': 'duplicate'
    }])
    
    assert '[Activity Tracker]' in notification
    assert 'test-1' in notification
    assert 'Test Issue' in notification
    
    print("✓ Notification formatting works")
except Exception as e:
    print(f"✗ Notification formatting failed: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)

# 4. Run pytest
print("\n4. Running Pytest...")
print("-" * 40)

try:
    import pytest
    
    # Run with minimal output first
    exit_code = pytest.main([
        'tests/',
        '-v',
        '--tb=short',
        '--no-header',
        '-x'  # Stop on first failure
    ])
    
    if exit_code == 0:
        print("\n✓ All tests passed!")
    else:
        print(f"\n✗ Tests failed with exit code: {exit_code}")
        print("\nRerun with more detail:")
        print("  python -m pytest tests/ -vv")
        sys.exit(exit_code)
        
except Exception as e:
    print(f"✗ Pytest execution failed: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)

print("\n" + "="*60)
print("✅ ALL CHECKS PASSED")
print("="*60)
