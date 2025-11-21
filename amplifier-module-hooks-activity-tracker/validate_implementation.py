"""
Direct validation of implementation WITHOUT pytest.
This proves the code actually works before worrying about test infrastructure.
"""

import sys
from pathlib import Path

# Add module to path
sys.path.insert(0, str(Path(__file__).parent))

print("=" * 70)
print("ACTIVITY TRACKER - DIRECT VALIDATION (NO PYTEST)")
print("=" * 70)

failures = []
successes = []

def test(name, fn):
    """Run a test function and track results."""
    try:
        fn()
        print(f"[PASS] {name}")
        successes.append(name)
    except Exception as e:
        print(f"[FAIL] {name}: {e}")
        failures.append((name, e))

# ============================================================================
# TEST: Utils Module
# ============================================================================
print("\n1. Testing utils.py")
print("-" * 70)

def test_content_hash():
    from amplifier_module_hooks_activity_tracker.utils import compute_content_hash
    
    h1 = compute_content_hash("test")
    h2 = compute_content_hash("test")
    h3 = compute_content_hash("different")
    
    assert h1 == h2, "Same input should produce same hash"
    assert h1 != h3, "Different inputs should produce different hashes"
    assert len(h1) == 64, f"SHA-256 should be 64 chars, got {len(h1)}"
    assert all(c in '0123456789abcdef' for c in h1), "Should be hex"

test("compute_content_hash", test_content_hash)

def test_format_notification():
    from amplifier_module_hooks_activity_tracker.utils import format_notification
    from unittest.mock import Mock
    
    issue = Mock()
    issue.id = "test-1"
    issue.title = "Test Issue"
    
    result = format_notification([{
        'issue': issue,
        'confidence': 0.95,
        'reasoning': 'Test',
        'relationship_type': 'duplicate'
    }])
    
    assert '[Activity Tracker]' in result
    assert 'test-1' in result
    assert '95%' in result or '0.95' in result

test("format_notification", test_format_notification)

def test_parse_git_status():
    from amplifier_module_hooks_activity_tracker.utils import parse_git_status
    
    # Git status short format: XY filename (2 chars for status, 1 space, then filename)
    # Testing with correct format
    output = "M  file1.py\nA  file2.py\nD  file3.py\n?? file4.py"
    result = parse_git_status(output)
    
    assert 'file1.py' in result['modified'], f"Expected file1.py in modified, got {result['modified']}"
    assert 'file2.py' in result['added'], f"Expected file2.py in added, got {result['added']}"
    assert 'file3.py' in result['deleted'], f"Expected file3.py in deleted, got {result['deleted']}"
    assert 'file4.py' in result['untracked'], f"Expected file4.py in untracked, got {result['untracked']}"

test("parse_git_status", test_parse_git_status)

def test_sanitize_llm_response():
    from amplifier_module_hooks_activity_tracker.utils import sanitize_llm_response
    
    # Test with code fence
    result = sanitize_llm_response('```json\n{"test": true}\n```')
    assert result == '{"test": true}'
    
    # Test without fence
    result = sanitize_llm_response('{"test": true}')
    assert result == '{"test": true}'

test("sanitize_llm_response", test_sanitize_llm_response)

# ============================================================================
# TEST: Embedding Cache
# ============================================================================
print("\n2. Testing embedding_cache.py")
print("-" * 70)

def test_embedding_cache_basic():
    from amplifier_module_hooks_activity_tracker.embedding_cache import EmbeddingCache
    import numpy as np
    import tempfile
    import asyncio
    
    with tempfile.NamedTemporaryFile(suffix='.db', delete=False) as f:
        cache_path = Path(f.name)
    
    try:
        cache = EmbeddingCache(cache_path)
        
        # Store embedding (using correct method name 'set')
        embedding = np.array([0.1, 0.2, 0.3])
        
        # Run async operations
        async def test_cache_ops():
            # Correct signature: set(issue_id, embedding, model, content_hash)
            await cache.set("test-1", embedding, "test-model", "hash123")
            
            # Retrieve with correct hash
            retrieved = await cache.get("test-1", "hash123")
            assert retrieved is not None, "Should retrieve cached embedding"
            assert np.array_equal(retrieved, embedding), "Retrieved embedding should match"
            
            # Wrong hash should return None
            retrieved = await cache.get("test-1", "wrong-hash")
            assert retrieved is None, "Wrong hash should return None"
        
        asyncio.run(test_cache_ops())
        
    finally:
        if cache_path.exists():
            cache_path.unlink(missing_ok=True)

test("embedding_cache_basic", test_embedding_cache_basic)

# ============================================================================
# TEST: Project Group Manager
# ============================================================================
print("\n3. Testing project_group_manager.py")
print("-" * 70)

def test_project_group_manager():
    from amplifier_module_hooks_activity_tracker.project_group_manager import ProjectGroupManager
    
    # Should handle missing config gracefully
    manager = ProjectGroupManager({})
    assert manager.groups == {}
    
    # get_group_for_repo should return (None, None) if no groups
    group_name, group_config = manager.get_group_for_repo(str(Path.cwd()))
    assert group_name is None
    assert group_config is None

test("project_group_manager_init", test_project_group_manager)

# ============================================================================
# TEST: Hooks Module
# ============================================================================
print("\n4. Testing hooks.py")
print("-" * 70)

def test_hooks_init():
    from amplifier_module_hooks_activity_tracker.hooks import ActivityTrackerHook
    
    config = {
        'notify_threshold': 0.85,
        'auto_track_sessions': True,
    }
    
    hook = ActivityTrackerHook(config)
    assert hook.config == config
    assert hook.session_issues == {}

test("hooks_init", test_hooks_init)

# ============================================================================
# TEST: Analyzer Module
# ============================================================================
print("\n5. Testing analyzer.py")
print("-" * 70)

def test_analyzer_init():
    from amplifier_module_hooks_activity_tracker.analyzer import ActivityAnalyzer
    
    config = {'similarity_threshold': 0.7}
    analyzer = ActivityAnalyzer(config)
    assert analyzer.config == config

test("analyzer_init", test_analyzer_init)

# ============================================================================
# TEST: Embedding Generator
# ============================================================================
print("\n6. Testing embedding_generator.py")
print("-" * 70)

def test_embedding_generator_init():
    from amplifier_module_hooks_activity_tracker.embedding_generator import EmbeddingGenerator
    
    config = {'embedding_model': 'test-model'}
    generator = EmbeddingGenerator(config)
    assert generator.model == 'test-model'

test("embedding_generator_init", test_embedding_generator_init)

# ============================================================================
# TEST: Module Entry Point
# ============================================================================
print("\n7. Testing __init__.py")
print("-" * 70)

def test_module_import():
    from amplifier_module_hooks_activity_tracker import mount
    assert callable(mount)

test("module_import", test_module_import)

# ============================================================================
# SUMMARY
# ============================================================================
print("\n" + "=" * 70)
print("VALIDATION RESULTS")
print("=" * 70)

total = len(successes) + len(failures)
print(f"\nTotal tests: {total}")
print(f"Passed: {len(successes)}")
print(f"Failed: {len(failures)}")

if failures:
    print("\nFAILURES:")
    for name, error in failures:
        print(f"  - {name}: {error}")
    print("\n[FAIL] VALIDATION FAILED")
    sys.exit(1)
else:
    print("\n[PASS] ALL VALIDATIONS PASSED")
    print("\nCore functionality verified:")
    for name in successes:
        print(f"  + {name}")
    sys.exit(0)
