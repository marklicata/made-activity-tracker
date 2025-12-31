#!/usr/bin/env python3
"""HTTP server that wraps Amplifier for Tauri integration."""

import asyncio
import os
import sys
from pathlib import Path
from flask import Flask, request, jsonify
from flask_cors import CORS
from amplifier_foundation import load_bundle
from made_activity_tools import set_db_path

app = Flask(__name__)
CORS(app)

# Configuration
AUTH_TOKEN = os.environ.get('AMPLIFIER_AUTH_TOKEN', 'dev-token')
DB_PATH = os.environ.get('DATABASE_PATH')
API_KEY = None
if 'ANTHROPIC_API_KEY' in os.environ:
    PROVIDER = 'anthropic'
    API_KEY = os.environ['ANTHROPIC_API_KEY']
elif 'OPENAI_API_KEY' in os.environ:
    PROVIDER = 'openai'
    API_KEY = os.environ['OPENAI_API_KEY']
if API_KEY is None:
    keys_path = Path(os.path.abspath(__file__)).parent.parent.parent.parent.parent / 'keys.env'
    if keys_path.exists():
        with open(keys_path) as f:
            for line in f:
                if line.startswith('ANTHROPIC_API_KEY='):
                    API_KEY = line.strip().split('=', 1)[1].strip().strip('"').strip("'")
                    PROVIDER = 'anthropic'
                    break
                elif line.startswith('OPENAI_API_KEY='):
                    API_KEY = line.strip().split('=', 1)[1].strip().strip('"').strip("'")
                    PROVIDER = 'openai'
                    break

# Global Amplifier session
_amplifier_session = None

# Configure database path
if DB_PATH:
    set_db_path(DB_PATH)

def check_auth():
    """Verify request authentication."""
    token = request.headers.get('X-Auth-Token')
    return token == AUTH_TOKEN


@app.route('/health', methods=['GET'])
@app.route('/', methods=['GET'])
def health():
    """Health check endpoint."""
    return jsonify({
        'status': 'ok',
        'provider': PROVIDER,
        'has_api_key': bool(API_KEY),
        'db_path': DB_PATH
    })

async def get_amplifier_session():
    """Get or create Amplifier session."""
    global _amplifier_session
    if _amplifier_session is None:
        # Ensure API key is in environment for provider modules to pick up
        if PROVIDER == 'anthropic' and API_KEY:
            os.environ['ANTHROPIC_API_KEY'] = API_KEY
        elif PROVIDER == 'openai' and API_KEY:
            os.environ['OPENAI_API_KEY'] = API_KEY

        # Get the directory where this script is located
        server_dir = Path(__file__).parent.parent.parent.resolve()

        # Load local bundle (includes foundation)
        bundle_path = server_dir / "bundle.md"
        base_bundle = await load_bundle(str(bundle_path))

        # Load provider configuration
        provider_path = server_dir / "providers" / f"{PROVIDER}.yaml"
        provider_bundle = await load_bundle(str(provider_path))

        # Compose bundles (foundation from includes + provider + tools)
        composed = base_bundle.compose(provider_bundle)

        # Prepare and create session
        prepared = await composed.prepare()
        _amplifier_session = await prepared.create_session()

    return _amplifier_session


async def get_amplifier_session():
    """Get or create Amplifier session."""
    global _amplifier_session
    if _amplifier_session is None:
        # Ensure API key is in environment for provider modules to pick up
        if PROVIDER == 'anthropic' and API_KEY:
            os.environ['ANTHROPIC_API_KEY'] = API_KEY
        elif PROVIDER == 'openai' and API_KEY:
            os.environ['OPENAI_API_KEY'] = API_KEY

        # Get the directory where this script is located
        server_dir = Path(__file__).parent.parent.parent.resolve()
        
        # Load local bundle
        bundle_path = server_dir / "bundle.md"
        base_bundle = await load_bundle(str(bundle_path))
        
        # Load provider configuration
        provider_path = server_dir / "providers" / f"{PROVIDER}.yaml"
        provider_bundle = await load_bundle(str(provider_path))
        
        # Compose bundles (foundation from includes + provider + tools)
        composed = base_bundle.compose(provider_bundle)
        
        # Prepare and create session
        prepared = await composed.prepare()
        _amplifier_session = await prepared.create_session()

    return _amplifier_session


@app.route('/chat', methods=['POST'])
async def chat():
    """Process chat message through Amplifier."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401

    if not API_KEY:
        return jsonify({'error': 'No API key configured'}), 500

    global _amplifier_session
    try:
        data = request.json
        user_message = data.get('message', '')
        app_context = data.get('context', {})

        # Build context message with app state
        context_parts = []
        
        if app_context.get('current_page'):
            context_parts.append(f"User is viewing: {app_context['current_page']}")

        filters = app_context.get('filters', {})
        if date_range := filters.get('date_range'):
            context_parts.append(f"Date range: {date_range['start']} to {date_range['end']}")
        if repos := filters.get('repositories'):
            context_parts.append(f"Filtered repositories: {', '.join(repos)}")
        if squads := filters.get('squads'):
            context_parts.append(f"Filtered squads: {', '.join(squads)}")
        if users := filters.get('users'):
            context_parts.append(f"Filtered users: {', '.join(users)}")

        if _amplifier_session is None:
            print("Creating new Amplifier session...", file=sys.stderr)
            _amplifier_session = await get_amplifier_session()
            print("Amplifier session created.", file=sys.stderr)
        
        # Add context as a user message if we have any
        context_message = ""
        if context_parts:
            context_message = "Current app context:\n" + "\n".join(context_parts)
        context_message += "\n\n" + "Current user message: " + user_message
        
        # Execute user message
        response = await _amplifier_session.execute(context_message)

        return jsonify({
            'response': response,
            'context': app_context
        })

    except Exception as e:
        print(f"Error in chat: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return jsonify({'error': str(e)}), 500


@app.route('/shutdown', methods=['POST'])
async def shutdown():
    """Shutdown server and cleanup."""
    if not check_auth():
        return jsonify({'error': 'Unauthorized'}), 401
    
    global _amplifier_session
    if _amplifier_session:
        try:
            await _amplifier_session.cleanup()
        except Exception as e:
            print(f"Error during cleanup: {e}", file=sys.stderr)
        _amplifier_session = None

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

    # Get session and execute
    print(f"Initializing Amplifier session...", file=sys.stderr)
    _amplifier_session = asyncio.run(get_amplifier_session())
    print(f"Amplifier session initialized.", file=sys.stderr)

    app.run(
        host='127.0.0.1',
        port=port,
        debug=False,
        use_reloader=False
    )