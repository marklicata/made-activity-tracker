import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Search as SearchIcon, AlertTriangle, ExternalLink, Loader2 } from 'lucide-react';
import clsx from 'clsx';

interface SearchResult {
  id: string;
  item_type: 'issue' | 'pull_request';
  title: string;
  body_preview: string;
  repo: string;
  number: number;
  state: string;
  author: string;
  created_at: string;
  url: string;
  score: number;
  duplicates?: DuplicateMatch[];
}

interface DuplicateMatch {
  id: string;
  title: string;
  repo: string;
  number: number;
  similarity: number;
  url: string;
}

export default function Search() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [showDuplicates, setShowDuplicates] = useState(true);

  const handleSearch = async () => {
    if (!query.trim()) return;
    
    setLoading(true);
    try {
      const data = await invoke<SearchResult[]>('hybrid_search', { 
        query, 
        includeDuplicates: showDuplicates 
      });
      setResults(data);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-8">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900">Search</h1>
        <p className="text-gray-500">Semantic search across all issues and PRs</p>
      </div>

      {/* Search Input */}
      <div className="mb-6">
        <div className="flex gap-4">
          <div className="flex-1 relative">
            <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400" size={20} />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              placeholder="Search by meaning, not just keywords..."
              className="w-full pl-12 pr-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={loading || !query.trim()}
            className="px-6 py-3 bg-blue-600 text-white rounded-xl font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
          >
            {loading ? (
              <>
                <Loader2 size={18} className="animate-spin" />
                Searching...
              </>
            ) : (
              'Search'
            )}
          </button>
        </div>
        
        {/* Options */}
        <div className="mt-3 flex items-center gap-4">
          <label className="flex items-center gap-2 text-sm text-gray-600 cursor-pointer">
            <input
              type="checkbox"
              checked={showDuplicates}
              onChange={(e) => setShowDuplicates(e.target.checked)}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            Show potential duplicates
          </label>
        </div>
      </div>

      {/* Results */}
      <div className="space-y-4">
        {results.map((result) => (
          <div key={result.id} className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <div className="p-6">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className={clsx(
                      'px-2 py-0.5 text-xs font-medium rounded-full',
                      result.item_type === 'issue' ? 'bg-green-100 text-green-700' : 'bg-purple-100 text-purple-700'
                    )}>
                      {result.item_type === 'issue' ? 'Issue' : 'PR'}
                    </span>
                    <span className="text-sm text-gray-500">{result.repo} #{result.number}</span>
                    <span className={clsx(
                      'px-2 py-0.5 text-xs rounded-full',
                      result.state === 'open' ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'
                    )}>
                      {result.state}
                    </span>
                  </div>
                  <h3 className="text-lg font-medium text-gray-900 mb-1">{result.title}</h3>
                  <p className="text-gray-600 text-sm">{result.body_preview}</p>
                  <div className="mt-3 flex items-center gap-4 text-xs text-gray-400">
                    <span>by {result.author}</span>
                    <span>{new Date(result.created_at).toLocaleDateString()}</span>
                    <span>Match: {(result.score * 100).toFixed(0)}%</span>
                  </div>
                </div>
                <a
                  href={result.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                >
                  <ExternalLink size={18} />
                </a>
              </div>
            </div>

            {/* Duplicates Warning */}
            {result.duplicates && result.duplicates.length > 0 && (
              <div className="px-6 py-4 bg-amber-50 border-t border-amber-100">
                <div className="flex items-center gap-2 mb-3">
                  <AlertTriangle className="text-amber-600" size={16} />
                  <span className="text-sm font-medium text-amber-800">
                    Potential duplicates found
                  </span>
                </div>
                <div className="space-y-2">
                  {result.duplicates.map((dup) => (
                    <div key={dup.id} className="flex items-center justify-between text-sm">
                      <div className="flex items-center gap-2">
                        <span className="text-gray-500">{dup.repo} #{dup.number}</span>
                        <span className="text-gray-700">{dup.title}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-amber-600 font-medium">
                          {(dup.similarity * 100).toFixed(0)}% similar
                        </span>
                        <a
                          href={dup.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="p-1 text-gray-400 hover:text-gray-600"
                        >
                          <ExternalLink size={14} />
                        </a>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ))}

        {results.length === 0 && query && !loading && (
          <div className="text-center py-12 text-gray-500">
            No results found for "{query}"
          </div>
        )}

        {!query && (
          <div className="text-center py-12 text-gray-400">
            Enter a search query to find issues and PRs by meaning
          </div>
        )}
      </div>
    </div>
  );
}
