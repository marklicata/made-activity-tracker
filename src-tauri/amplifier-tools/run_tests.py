"""Comprehensive test suite for MADE Activity Tools setup."""

import sys
import os
import asyncio
from pathlib import Path


def print_header(text):
    """Print a section header."""
    print("\n" + "=" * 70)
    print(f"  {text}")
    print("=" * 70)


def print_test(name, passed, details=None):
    """Print test result."""
    status = "✓ PASS" if passed else "✗ FAIL"
    print(f"\n{status}: {name}")
    if details:
        for line in details:
            print(f"       {line}")


def test_python_version():
    """Test Python version."""
    version = sys.version_info
    passed = version >= (3, 11)
    details = [
        f"Version: {version.major}.{version.minor}.{version.micro}",
        "Requirement: 3.11+" if not passed else "Meets requirement"
    ]
    return passed, details


def test_imports():
    """Test required imports."""
    results = []
    
    # Test tool module
    try:
        from made_activity_tools import tool_module
        results.append(("Tool module", True, ["Successfully imported"]))
    except Exception as e:
        results.append(("Tool module", False, [f"Import failed: {e}"]))
    
    # Test amplifier-foundation
    try:
        from amplifier_foundation import load_bundle, Bundle
        results.append(("amplifier-foundation", True, ["Successfully imported"]))
    except Exception as e:
        results.append(("amplifier-foundation", False, [f"Import failed: {e}", "Run: pip install -e ."]))
    
    # Test amplifier-core
    try:
        import amplifier_core
        results.append(("amplifier-core", True, ["Successfully imported"]))
    except Exception as e:
        results.append(("amplifier-core", False, [f"Import failed: {e}"]))
    
    return results


def test_file_structure():
    """Test required files exist."""
    base_path = Path(__file__).parent
    results = []
    
    # Check bundle.md
    bundle_path = base_path / "bundle.md"
    if bundle_path.exists():
        results.append(("bundle.md", True, [f"Found at: {bundle_path}"]))
    else:
        results.append(("bundle.md", False, ["File not found!"]))
    
    # Check providers directory
    providers_dir = base_path / "providers"
    if providers_dir.exists():
        anthropic = providers_dir / "anthropic.yaml"
        openai = providers_dir / "openai.yaml"
        results.append((
            "providers/", 
            anthropic.exists() and openai.exists(),
            [
                f"anthropic.yaml: {'Found' if anthropic.exists() else 'Missing'}",
                f"openai.yaml: {'Found' if openai.exists() else 'Missing'}"
            ]
        ))
    else:
        results.append(("providers/", False, ["Directory not found!"]))
    
    # Check tool module
    tool_module_path = base_path / "src" / "made_activity_tools" / "tool_module.py"
    if tool_module_path.exists():
        results.append(("tool_module.py", True, [f"Found at: {tool_module_path}"]))
    else:
        results.append(("tool_module.py", False, ["File not found!"]))
    
    return results


def test_environment():
    """Test environment variables."""
    results = []
    
    # Check API keys
    anthropic_key = os.environ.get('ANTHROPIC_API_KEY')
    openai_key = os.environ.get('OPENAI_API_KEY')
    
    if anthropic_key:
        results.append(("ANTHROPIC_API_KEY", True, ["Set (provider: anthropic)"]))
    elif openai_key:
        results.append(("OPENAI_API_KEY", True, ["Set (provider: openai)"]))
    else:
        results.append((
            "API Key",
            False,
            [
                "Neither ANTHROPIC_API_KEY nor OPENAI_API_KEY is set",
                "Set one with:",
                "  PowerShell: $env:ANTHROPIC_API_KEY = 'your-key'",
                "  CMD: set ANTHROPIC_API_KEY=your-key"
            ]
        ))
    
    # Check database path
    db_path = os.environ.get('DATABASE_PATH')
    if db_path:
        if Path(db_path).exists():
            results.append(("DATABASE_PATH", True, [f"Set and found: {db_path}"]))
        else:
            results.append(("DATABASE_PATH", False, [f"Set but not found: {db_path}"]))
    else:
        # Check default location
        if os.name == 'nt':
            base = os.environ.get('APPDATA')
            default_path = Path(base) / 'com.made.activity-tracker' / 'activity.db'
        else:
            default_path = Path.home() / '.local' / 'share' / 'com.made.activity-tracker' / 'activity.db'
        
        if default_path.exists():
            results.append(("Database", True, [f"Found at default: {default_path}"]))
        else:
            results.append((
                "Database",
                False,
                [
                    f"Not found at default: {default_path}",
                    "Set DATABASE_PATH if in custom location"
                ]
            ))
    
    return results


async def test_bundle_loading():
    """Test bundle can be loaded."""
    try:
        from amplifier_foundation import load_bundle
        
        bundle_path = Path(__file__).parent / "bundle.md"
        bundle = await load_bundle(str(bundle_path))
        
        details = [
            f"Bundle name: {bundle.name}",
            f"Tools: {len(bundle.tools)} defined",
            f"Includes: {len(bundle.includes)} referenced"
        ]
        
        # Show tools
        for tool in bundle.tools:
            details.append(f"  - {tool.get('module')}")
        
        return True, details
    except Exception as e:
        return False, [f"Failed to load bundle: {e}"]


def main():
    """Run all tests."""
    print_header("MADE Activity Tools - Test Suite")
    
    all_passed = True
    
    # Test 1: Python version
    print_header("Test 1: Python Version")
    passed, details = test_python_version()
    print_test("Python version check", passed, details)
    all_passed = all_passed and passed
    
    # Test 2: Imports
    print_header("Test 2: Python Imports")
    for name, passed, details in test_imports():
        print_test(name, passed, details)
        all_passed = all_passed and passed
    
    # Test 3: File structure
    print_header("Test 3: File Structure")
    for name, passed, details in test_file_structure():
        print_test(name, passed, details)
        all_passed = all_passed and passed
    
    # Test 4: Environment
    print_header("Test 4: Environment Variables")
    for name, passed, details in test_environment():
        print_test(name, passed, details)
        all_passed = all_passed and passed
    
    # Test 5: Bundle loading
    print_header("Test 5: Bundle Loading")
    try:
        passed, details = asyncio.run(test_bundle_loading())
        print_test("Load bundle.md", passed, details)
        all_passed = all_passed and passed
    except Exception as e:
        print_test("Load bundle.md", False, [f"Test failed: {e}"])
        all_passed = False
    
    # Final summary
    print_header("Test Summary")
    if all_passed:
        print("\n✓✓✓ ALL TESTS PASSED ✓✓✓")
        print("\nYou can now start the server:")
        print("  python src/made_activity_tools/server.py")
    else:
        print("\n✗✗✗ SOME TESTS FAILED ✗✗✗")
        print("\nPlease fix the issues above before running the server.")
    
    return 0 if all_passed else 1


if __name__ == "__main__":
    sys.exit(main())
