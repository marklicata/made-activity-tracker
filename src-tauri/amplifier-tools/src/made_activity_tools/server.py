#!/usr/bin/env python3
"""HTTP server that wraps Amplifier for Tauri integration."""

import asyncio
import os
import sys
from flask import Flask, request, jsonify, Response
from flask_cors import CORS
from amplifier_core import AmplifierSession
from made_activity_tools import set_db_path

app = Flask(__name__)
CORS(app)

# Configuration
AUTH_TOKEN = os.environ.get('AMPLIFIER_AUTH_TOKEN', 'dev-token')
DB_PATH = os.environ.get('DATABASE_PATH')
API_KEY = os.environ.get('ANTHROPIC_API_KEY') or os.environ.get('OPENAI_API_KEY')
PROVIDER = 'anthropic' if os.environ.get('ANTHROPIC_API_KEY') else 'openai'

# Configure database path
if DB_PATH:
    set_db_path(DB_PATH)

# Amplifier configuration
AMPLIFIER_CONFIG = {
    "session": {
        "orchestrator": "loop-basic",
        "context": "context-simple"
    },
    "providers": [
        {
            "module": f"provider-{PROVIDER}",
            "config": {
                "api_key": API_KEY
            }
        }
    ],
    "tools": [
        {"module": "made_activity_tools.metrics_tool"},
        {"module": "made_activity_tools.search_tool"},
        {"module": "made_activity_tools.user_activity_tool"}
    ]
}


def check_auth():
    """Verify request authentication."""
    token = request.headers.get('X-Auth-Token')
    return token == AUTH_TOKEN


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({
        'status': 'ok',
        'provider': PROVIDER,
        'has_api_key': bool(API_KEY),
        'db_path': DB_PATH
    })


@app.route('/chat', methods=['POST'])
async def chat():
    """Process chat message through Amplifier."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401

    if not API_KEY:
        return jsonify({'error': 'No API key configured'}), 500

    try:
        data = request.json
        user_message = data.get('message', '')
        app_context = data.get('context', {})

        # Build system prompt with app context
        system_prompt = build_system_prompt(app_context)

        # Create Amplifier session and execute
        async with AmplifierSession(config=AMPLIFIER_CONFIG) as session:
            # Add system context as first message if needed
            full_prompt = f"{system_prompt}\n\nUser: {user_message}"

            response = await session.execute(full_prompt)

            return jsonify({
                'response': response,
                'context': app_context
            })

    except Exception as e:
        print(f"Error in chat: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return jsonify({'error': str(e)}), 500


def build_system_prompt(context: dict) -> str:
    """Build system prompt from app context."""
    parts = ["You are an AI assistant helping users analyze GitHub activity data."]

    if context.get('current_page'):
        parts.append(f"The user is currently viewing: {context['current_page']}")

    filters = context.get('filters', {})

    if date_range := filters.get('date_range'):
        parts.append(f"Date range: {date_range['start']} to {date_range['end']}")

    if repos := filters.get('repositories'):
        parts.append(f"Filtered repositories: {', '.join(repos)}")

    if squads := filters.get('squads'):
        parts.append(f"Filtered squads: {', '.join(squads)}")

    if users := filters.get('users'):
        parts.append(f"Filtered users: {', '.join(users)}")

    parts.append("\nUse the available tools to query the database and provide accurate answers.")

    return "\n".join(parts)


@app.route('/shutdown', methods=['POST'])
def shutdown():
    """Shutdown server."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401

    func = request.environ.get('werkzeug.server.shutdown')
    if func:
        func()

    return jsonify({'status': 'shutting down'})


if __name__ == '__main__':
    port = int(os.environ.get('AMPLIFIER_PORT', 5000))

    print(f"Starting Amplifier server on port {port}", file=sys.stderr)
    print(f"Provider: {PROVIDER}", file=sys.stderr)
    print(f"Database: {DB_PATH or 'default location'}", file=sys.stderr)
    print(f"API Key configured: {bool(API_KEY)}", file=sys.stderr)

    app.run(
        host='127.0.0.1',
        port=port,
        debug=False,
        use_reloader=False
    )
