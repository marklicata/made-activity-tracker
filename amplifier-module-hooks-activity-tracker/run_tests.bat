@echo off
REM Install dependencies
echo Installing test dependencies...
python -m pip install pytest pytest-cov pytest-asyncio openai pyyaml numpy --quiet

REM Install the module
echo Installing module...
python -m pip install -e . --quiet

REM Run tests with coverage
echo.
echo Running tests...
echo =====================================
python -m pytest tests/ --cov=amplifier_module_hooks_activity_tracker --cov-report=term -v

echo.
echo =====================================
echo Done!
