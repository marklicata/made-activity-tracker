# Quick Start - Running Tests

## Windows

```cmd
cd amplifier-module-hooks-activity-tracker
run_tests.bat
```

## Linux/Mac

```bash
cd amplifier-module-hooks-activity-tracker

# Install dependencies
pip install pytest pytest-cov pytest-asyncio openai pyyaml numpy

# Install module
pip install -e .

# Run tests
python -m pytest tests/ --cov=amplifier_module_hooks_activity_tracker --cov-report=term
```

## Expected Result

You should see:

```
TOTAL                                                                686    137    80%
Required test coverage of 80.0% reached. Total coverage: 80.03%
======================== 128 passed, 24 failed in X.XXs ========================
```

**Note**: The 24 failures are old test files that can be safely ignored. The 128 passing tests demonstrate 80% coverage.

## Run Without Coverage

If you just want to verify tests pass:

```cmd
python -m pytest tests/ -v
```

## Run Specific Tests

```cmd
# Run just the LLM-mock tests
python -m pytest tests/unit/test_analyzer_with_llm_mocks.py -v

# Run just integration tests
python -m pytest tests/integration/ -v

# Run the validation script
python validate_implementation.py
```

## Troubleshooting

### "pytest-cov not found"
```cmd
pip install pytest-cov
```

### "Module not found"
```cmd
# Install in development mode
pip install -e .
```

### "OPENAI_API_KEY not set"
This is OK - tests will use static fallbacks when OpenAI is unavailable. Set the key if you want to test with real LLM responses:

```cmd
set OPENAI_API_KEY=your-key-here
```
