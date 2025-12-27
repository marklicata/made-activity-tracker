import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useChatStore } from '../../stores/chatStore';
import ChatMessage from './ChatMessage';
import ChatInput from './ChatInput';
import ChatSuggestions from './ChatSuggestions';
import ApiKeySetup from './ApiKeySetup';
import { X, MessageSquare, Trash2 } from 'lucide-react';

interface ApiKeyStatus {
  has_anthropic: boolean;
  has_openai: boolean;
}

const ChatPanel: React.FC = () => {
  const { isOpen, messages, isLoading, error, setOpen, clearMessages } = useChatStore();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [needsApiKey, setNeedsApiKey] = useState(false);
  const [checkingKeys, setCheckingKeys] = useState(true);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Check for API keys when panel opens
  useEffect(() => {
    if (isOpen) {
      console.log('[ChatPanel] Panel opened, checking API keys and connection...');
      checkApiKeys();
    }
  }, [isOpen]);

  const checkApiKeys = async () => {
    console.log('[ChatPanel] Starting API key check...');
    setCheckingKeys(true);
    try {
      console.log('[ChatPanel] Invoking check_api_keys command...');
      const status = await invoke<ApiKeyStatus>('check_api_keys');
      console.log('[ChatPanel] API key status:', status);

      const needsKey = !status.has_anthropic && !status.has_openai;
      setNeedsApiKey(needsKey);

      if (needsKey) {
        console.warn('[ChatPanel] ⚠ No API keys found');
      } else {
        console.log('[ChatPanel] ✓ API keys available');

        // Also check backend health
        console.log('[ChatPanel] Checking Amplifier backend health...');
        try {
          const isHealthy = await invoke<boolean>('check_amplifier_health');
          console.log('[ChatPanel] Amplifier health check result:', isHealthy);
          if (!isHealthy) {
            console.error('[ChatPanel] ✗ Amplifier backend health check failed');
          } else {
            console.log('[ChatPanel] ✓ Amplifier backend is healthy');
          }
        } catch (healthErr) {
          console.error('[ChatPanel] ✗ Amplifier health check error:', healthErr);
        }
      }
    } catch (err) {
      console.error('[ChatPanel] ✗ Failed to check API keys:', err);
      setNeedsApiKey(true);
    } finally {
      setCheckingKeys(false);
      console.log('[ChatPanel] API key check complete');
    }
  };

  const handleApiKeyComplete = () => {
    setNeedsApiKey(false);
  };

  if (!isOpen) return null;

  const handleClose = () => setOpen(false);

  return (
    <div className="fixed right-0 top-0 h-full w-96 bg-gray-900 border-l border-gray-700 flex flex-col shadow-xl z-50">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-700">
        <div className="flex items-center gap-2">
          <MessageSquare className="w-5 h-5 text-blue-400" />
          <h2 className="text-lg font-semibold text-white">AI Assistant</h2>
        </div>
        <div className="flex items-center gap-2">
          {messages.length > 0 && !needsApiKey && (
            <button
              onClick={clearMessages}
              className="p-2 hover:bg-gray-800 rounded transition-colors"
              title="Clear history"
            >
              <Trash2 className="w-4 h-4 text-gray-400" />
            </button>
          )}
          <button
            onClick={handleClose}
            className="p-2 hover:bg-gray-800 rounded transition-colors"
          >
            <X className="w-5 h-5 text-gray-400" />
          </button>
        </div>
      </div>

      {/* Content */}
      {checkingKeys ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="flex items-center gap-2 text-gray-400">
            <div className="animate-spin w-5 h-5 border-2 border-blue-400 border-t-transparent rounded-full" />
            <span>Checking setup...</span>
          </div>
        </div>
      ) : needsApiKey ? (
        <ApiKeySetup onComplete={handleApiKeyComplete} />
      ) : (
        <>
          {/* Messages */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {messages.length === 0 ? (
              <ChatSuggestions />
            ) : (
              messages.map((message, index) => (
                <ChatMessage key={`${message.timestamp}-${index}`} message={message} />
              ))
            )}
            {isLoading && (
              <div className="flex items-center gap-2 text-gray-400">
                <div className="animate-spin w-4 h-4 border-2 border-blue-400 border-t-transparent rounded-full" />
                <span>Thinking...</span>
              </div>
            )}
            {error && (
              <div className="p-3 bg-red-900/20 border border-red-700 rounded text-red-400 text-sm">
                Error: {error}
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Input */}
          <ChatInput />
        </>
      )}
    </div>
  );
};

export default ChatPanel;
