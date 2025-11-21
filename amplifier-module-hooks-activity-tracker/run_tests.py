#!/usr/bin/env python
"""Simple test runner to verify tests actually work."""

import sys
import subprocess

def run_command(cmd, description):
    """Run a command and print results."""
    print(f"\n{'='*60}")
    print(f"{description}")
    print(f"{'='*60}")
    print(f"Running: {cmd}")
    print()
    
    result = subprocess.run(
        cmd, 
        shell=True, 
        capture_output=True, 
        text=True
    )
    
    print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    print(f"\nReturn code: {result.returncode}")
    return result.returncode

def main():
    """Run all test checks."""
    print("ACTIVITY TRACKER - TEST VERIFICATION")
    print("=" * 60)
    
    # Check dependencies
    exit_code = run_command(
        "python -c \"import pytest; import openai; import yaml; import numpy; print('All dependencies installed successfully')\"",
        "1. Checking Dependencies"
    )
    
    if exit_code != 0:
        print("\n❌ Dependencies not installed!")
        print("Run: pip install pytest pytest-asyncio pytest-cov openai pyyaml numpy")
        return 1
    
    # Run unit tests
    exit_code = run_command(
        "python -m pytest tests/unit/ -v --tb=short",
        "2. Running Unit Tests"
    )
    
    if exit_code != 0:
        print("\n❌ Unit tests failed!")
        return 1
    
    # Run integration tests
    exit_code = run_command(
        "python -m pytest tests/integration/ -v --tb=short",
        "3. Running Integration Tests"
    )
    
    if exit_code != 0:
        print("\n❌ Integration tests failed!")
        return 1
    
    # Measure coverage
    exit_code = run_command(
        "python -m pytest tests/ --cov=amplifier_module_hooks_activity_tracker --cov-report=term-missing",
        "4. Measuring Test Coverage"
    )
    
    print("\n" + "="*60)
    if exit_code == 0:
        print("✅ ALL TESTS PASSED!")
    else:
        print("❌ TESTS FAILED")
    print("="*60)
    
    return exit_code

if __name__ == "__main__":
    sys.exit(main())
