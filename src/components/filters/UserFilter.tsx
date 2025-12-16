import { useState, useRef, useEffect } from 'react';
import { User, ChevronDown, Search } from 'lucide-react';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';

interface UserData {
  id: number;
  github_id: number;
  login: string;
  name?: string;
  avatar_url?: string;
  is_bot: boolean;
}

export default function UserFilter() {
  const [isOpen, setIsOpen] = useState(false);
  const [users, setUsers] = useState<UserData[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const { filters, setUser } = useDashboardFilterStore();

  // Load users on mount
  useEffect(() => {
    loadUsers();
  }, []);

  // Focus search input when dropdown opens
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setSearchQuery('');
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const loadUsers = async () => {
    try {
      setLoading(true);
      const loadedUsers = await invoke<UserData[]>('get_all_users');
      setUsers(loadedUsers);
    } catch (error) {
      console.error('Failed to load users:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSelectUser = (userId: number | null) => {
    setUser(userId);
    setIsOpen(false);
    setSearchQuery('');
  };

  // Filter users based on search query
  const filteredUsers = users.filter((user) => {
    const query = searchQuery.toLowerCase();
    return (
      user.login.toLowerCase().includes(query) ||
      user.name?.toLowerCase().includes(query)
    );
  });

  const selectedUser = users.find((u) => u.id === filters.userId);
  const displayText = selectedUser
    ? selectedUser.name || selectedUser.login
    : 'All users';

  // Don't show if squad filter is active (mutually exclusive)
  if (filters.squadId) {
    return null;
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors',
          isOpen || filters.userId
            ? 'bg-primary-50 border-primary-300 text-primary-700'
            : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
        )}
      >
        {selectedUser?.avatar_url ? (
          <img
            src={selectedUser.avatar_url}
            alt={selectedUser.login}
            className="w-5 h-5 rounded-full"
          />
        ) : (
          <User size={16} />
        )}
        <span>{displayText}</span>
        <ChevronDown
          size={16}
          className={clsx('transition-transform', isOpen && 'rotate-180')}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-72 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
          {/* Search input */}
          <div className="p-2 border-b border-gray-200">
            <div className="relative">
              <Search
                size={16}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
              />
              <input
                ref={searchInputRef}
                type="text"
                placeholder="Search users..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-9 pr-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
            </div>
          </div>

          {/* User list */}
          <div className="max-h-80 overflow-y-auto">
            {loading ? (
              <div className="p-4 text-sm text-gray-500 text-center">
                Loading users...
              </div>
            ) : (
              <div className="p-2">
                {/* All users option */}
                <button
                  onClick={() => handleSelectUser(null)}
                  className={clsx(
                    'w-full text-left px-3 py-2 rounded-lg text-sm transition-colors',
                    !filters.userId
                      ? 'bg-primary-50 text-primary-700 font-medium'
                      : 'text-gray-700 hover:bg-gray-100'
                  )}
                >
                  All users
                </button>

                {/* Filtered user options */}
                {filteredUsers.length === 0 ? (
                  <div className="px-3 py-2 text-sm text-gray-500 text-center">
                    No users found
                  </div>
                ) : (
                  filteredUsers.map((user) => (
                    <button
                      key={user.id}
                      onClick={() => handleSelectUser(user.id)}
                      className={clsx(
                        'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors',
                        filters.userId === user.id
                          ? 'bg-primary-50 text-primary-700 font-medium'
                          : 'text-gray-700 hover:bg-gray-100'
                      )}
                    >
                      {user.avatar_url ? (
                        <img
                          src={user.avatar_url}
                          alt={user.login}
                          className="w-6 h-6 rounded-full"
                        />
                      ) : (
                        <div className="w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center">
                          <User size={12} className="text-gray-500" />
                        </div>
                      )}
                      <div className="flex-1 text-left">
                        <div className="font-medium">{user.name || user.login}</div>
                        {user.name && (
                          <div className="text-xs text-gray-500">@{user.login}</div>
                        )}
                      </div>
                    </button>
                  ))
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
