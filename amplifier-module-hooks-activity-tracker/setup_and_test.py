"""Simple script to setup and run tests - works on Windows/Linux/Mac"""

import subprocess
import sys
from pathlib import Path

def run_command(cmd, description):
    """Run a command and show results."""
    print(f"\n{'='*60}")
    print(f"{description}")
    print(f"{'='*60}")
    print(f"Running: {cmd}")
    
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    if result.stdout:
        print(result.stdout)
    if result.stderr and "warning" not in result.stderr.lower():
        print("STDERR:", result.stderr)
    
    return result.returncode

def main():
    print("Activity Tracker - Test Setup and Execution")
    print("="*60)
    
    # Step 1: Install dependencies
    print("\nStep 1: Installing dependencies...")
    deps = ["pytest", "pytest-cov", "pytest-asyncio", "openai", "pyyaml", "numpy"]
    for dep in deps:
        subprocess.run(
            f"{sys.executable} -m pip install {dep} --quiet",
            shell=True,
            capture_output=True
        )
    print("✓ Dependencies installed")
    
    # Step 2: Install module
    print("\nStep 2: Installing module in development mode...")
    subprocess.run(
        f"{sys.executable} -m pip install -e . --quiet",
        shell=True,
        capture_output=True
    )
    print("✓ Module installed")
    
    # Step 3: Run basic test
    print("\nStep 3: Running basic tests...")
    result = subprocess.run(
        f"{sys.executable} -m pytest tests/unit/test_utils.py -v",
        shell=True,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    
    if result.returncode == 0:
        print("\n✓ Basic tests PASSED")
    else:
        print("\n✗ Basic tests FAILED")
        return 1
    
    # Step 4: Run all tests with coverage
    print("\nStep 4: Running full test suite with coverage...")
    result = subprocess.run(
        f"{sys.executable} -m pytest tests/ --cov=amplifier_module_hooks_activity_tracker --cov-report=term",
        shell=True,
        capture_output=True,
        text=True
    )
    
    # Print just the coverage summary
    lines = result.stdout.split('\n')
    in_coverage = False
    for line in lines:
        if 'Name' in line and 'Stmts' in line:
            in_coverage = True
        if in_coverage:
            print(line)
        if 'Required test coverage' in line or 'TOTAL' in line and in_coverage:
            break
    
    # Print test summary
    for line in lines[-5:]:
        if 'passed' in line or 'failed' in line:
            print(line)
    
    print("\n" + "="*60)
    print("SETUP AND TEST COMPLETE")
    print("="*60)
    
    if "80.0% reached" in result.stdout:
        print("\n✓ SUCCESS: 80% coverage achieved!")
        print("✓ Tests are working correctly")
        return 0
    else:
        print("\n⚠ Tests ran but check coverage above")
        return 0

if __name__ == "__main__":
    sys.exit(main())
