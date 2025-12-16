import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Github, Loader2, Copy, CheckCircle, ExternalLink } from 'lucide-react';
import { useAuthStore } from '@stores/authStore';
import { listen } from '@tauri-apps/api/event';

type LoginState = 'idle' | 'waiting_for_code' | 'polling' | 'success' | 'error';

export default function Login() {
  const [loginState, setLoginState] = useState<LoginState>('idle');
  const [deviceCode, setDeviceCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  
  const login = useAuthStore((state) => state.login);
  const checkAuth = useAuthStore((state) => state.checkAuth);
  const navigate = useNavigate();

  // Check if already authenticated on mount
  useEffect(() => {
    checkAuth().then(() => {
      const isAuthenticated = useAuthStore.getState().isAuthenticated;
      if (isAuthenticated) {
        navigate('/');
      }
    });
  }, [checkAuth, navigate]);

  // Listen for device code events from backend
  useEffect(() => {
    const unlisten = listen<string>('device-code', (event) => {
      setDeviceCode(event.payload);
      setLoginState('polling');
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleLogin = async () => {
    setLoginState('waiting_for_code');
    setError(null);
    setDeviceCode(null);

    try {
      await login();
      setLoginState('success');
      setTimeout(() => navigate('/'), 1000);
    } catch (err) {
      setLoginState('error');
      setError(err instanceof Error ? err.message : 'Login failed. Please try again.');
    }
  };

  const copyCode = async () => {
    if (deviceCode) {
      await navigator.clipboard.writeText(deviceCode);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-900 to-slate-800">
      <div className="bg-white rounded-2xl shadow-xl p-8 w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            <span className="text-blue-600">MADE</span> Tracker
          </h1>
          <p className="text-gray-500">
            Metrics for Activity, Delivery & Efficiency
          </p>
        </div>

        {/* Device Code Display */}
        {deviceCode && loginState === 'polling' && (
          <div className="mb-6 p-6 bg-blue-50 border border-blue-200 rounded-xl">
            <p className="text-sm text-blue-700 mb-3 text-center">
              Enter this code on GitHub:
            </p>
            <div className="flex items-center justify-center gap-3">
              <code className="text-3xl font-mono font-bold text-blue-900 tracking-widest">
                {deviceCode}
              </code>
              <button
                onClick={copyCode}
                className="p-2 text-blue-600 hover:bg-blue-100 rounded-lg transition-colors"
                title="Copy code"
              >
                {copied ? <CheckCircle size={20} /> : <Copy size={20} />}
              </button>
            </div>
            <div className="mt-4 text-center">
              <a
                href="https://github.com/login/device"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 text-sm text-blue-600 hover:text-blue-800"
              >
                Open github.com/login/device
                <ExternalLink size={14} />
              </a>
            </div>
            <div className="mt-4 flex items-center justify-center gap-2 text-sm text-blue-600">
              <Loader2 size={16} className="animate-spin" />
              Waiting for authorization...
            </div>
          </div>
        )}

        {/* Success State */}
        {loginState === 'success' && (
          <div className="mb-6 p-4 bg-green-50 border border-green-200 rounded-lg">
            <div className="flex items-center justify-center gap-2 text-green-700">
              <CheckCircle size={20} />
              <span className="font-medium">Authenticated successfully!</span>
            </div>
          </div>
        )}

        {/* Error Message */}
        {error && (
          <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-sm text-red-600">{error}</p>
          </div>
        )}

        {/* Description (shown when idle) */}
        {loginState === 'idle' && (
          <div className="mb-8 text-center">
            <p className="text-gray-600">
              Track your team's GitHub activity with comprehensive metrics for speed, ease, and quality.
            </p>
          </div>
        )}

        {/* Login Button */}
        <button
          onClick={handleLogin}
          disabled={loginState !== 'idle' && loginState !== 'error'}
          className="w-full flex items-center justify-center gap-3 px-6 py-3 bg-gray-900 text-white rounded-lg font-medium hover:bg-gray-800 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loginState === 'waiting_for_code' ? (
            <>
              <Loader2 size={20} className="animate-spin" />
              Starting authentication...
            </>
          ) : loginState === 'polling' ? (
            <>
              <Loader2 size={20} className="animate-spin" />
              Waiting for GitHub...
            </>
          ) : loginState === 'success' ? (
            <>
              <CheckCircle size={20} />
              Success! Redirecting...
            </>
          ) : (
            <>
              <Github size={20} />
              Sign in with GitHub
            </>
          )}
        </button>

        {/* Device Flow Info */}
        {loginState === 'idle' && (
          <p className="mt-6 text-xs text-gray-400 text-center">
            Uses GitHub Device Flow for secure authentication.
            <br />
            A browser window will open for you to authorize.
          </p>
        )}

        {/* Features Preview */}
        {loginState === 'idle' && (
          <div className="mt-8 pt-8 border-t border-gray-200">
            <p className="text-sm font-medium text-gray-700 mb-4">What you'll get:</p>
            <ul className="space-y-2 text-sm text-gray-600">
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-blue-500 rounded-full"></span>
                Speed metrics — cycle time, throughput
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                Ease metrics — PR size, review rounds
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-purple-500 rounded-full"></span>
                Quality metrics — bug rates, rework
              </li>
            </ul>
          </div>
        )}
      </div>
    </div>
  );
}
