import React from 'react';
import { useChatStore } from '../../stores/chatStore';
import { useConfigStore } from '../../stores/configStore';
import { Lightbulb } from 'lucide-react';
import { AppContext } from '../../types/ai';

const suggestions = [
  "What's our average cycle time this month?",
  "Show me all open bugs",
  "Who reviewed the most PRs?",
  "Which repositories need attention?",
  "What are the current trends in our activity?",
];

const ChatSuggestions: React.FC = () => {
  const { sendMessage } = useChatStore();
  const { dateRange, selectedRepos } = useConfigStore();

  const buildContext = (): AppContext => ({
    current_page: window.location.pathname,
    filters: {
      date_range: dateRange
        ? {
            start: dateRange.start,
            end: dateRange.end,
          }
        : undefined,
      repositories: selectedRepos || [],
      squads: [],
      users: [],
    },
  });

  const handleSuggestionClick = (suggestion: string) => {
    const context = buildContext();
    sendMessage(suggestion, context);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 text-gray-400">
        <Lightbulb className="w-5 h-5" />
        <p className="text-sm">Try asking:</p>
      </div>

      <div className="space-y-2">
        {suggestions.map((suggestion, index) => (
          <button
            key={index}
            onClick={() => handleSuggestionClick(suggestion)}
            className="w-full text-left p-3 bg-gray-800 hover:bg-gray-700 rounded-lg text-sm text-gray-300 transition-colors"
          >
            {suggestion}
          </button>
        ))}
      </div>

      <div className="text-xs text-gray-500 p-3 bg-gray-800/50 rounded">
        <p>
          I can help you analyze GitHub activity, search for issues/PRs, and
          provide insights about your team's metrics.
        </p>
      </div>
    </div>
  );
};

export default ChatSuggestions;
