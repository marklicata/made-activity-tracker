import { Outlet, NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Github,
  Users,
  Map,
  Search,
  Settings,
  RefreshCw,
  LogOut,
  MessageSquare
} from 'lucide-react';
import { useSyncStore } from '@stores/syncStore';
import { useAuthStore } from '@stores/authStore';
import { useChatStore } from '@stores/chatStore';
import ChatPanel from './ai/ChatPanel';
import clsx from 'clsx';

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/projects', icon: Github, label: 'Projects' },
  { to: '/team', icon: Users, label: 'Team' },
  { to: '/roadmap', icon: Map, label: 'Roadmap' },
  { to: '/search', icon: Search, label: 'Search' },
  { to: '/settings', icon: Settings, label: 'Settings' },
];

export default function Layout() {
  const { isSyncing, lastSyncAt, triggerSync } = useSyncStore();
  const logout = useAuthStore((state) => state.logout);
  const { isOpen, togglePanel } = useChatStore();

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-64 bg-white border-r border-gray-200 flex flex-col">
        {/* Logo */}
        <div className="p-6 border-b border-gray-200">
          <h1 className="text-xl font-bold text-gray-900">
            <span className="text-primary-600">MADE</span> Tracker
          </h1>
          <p className="text-xs text-gray-500 mt-1">
            Metrics for Activity, Delivery & Efficiency
          </p>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4 space-y-1">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === '/'}
              className={({ isActive }) =>
                clsx(
                  'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-primary-50 text-primary-700'
                    : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'
                )
              }
            >
              <Icon size={20} />
              {label}
            </NavLink>
          ))}
        </nav>

        {/* AI Chat Toggle */}
        <div className="p-4 border-t border-gray-200">
          <button
            onClick={togglePanel}
            className={clsx(
              'w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors',
              isOpen
                ? 'bg-blue-600 text-white'
                : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'
            )}
          >
            <MessageSquare size={16} />
            AI Assistant
          </button>
        </div>

        {/* Sync Status */}
        <div className="p-4 border-t border-gray-200">
          <button
            onClick={triggerSync}
            disabled={isSyncing}
            className={clsx(
              'w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors',
              isSyncing
                ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                : 'bg-primary-600 text-white hover:bg-primary-700'
            )}
          >
            <RefreshCw size={16} className={clsx(isSyncing && 'animate-spin')} />
            {isSyncing ? 'Syncing...' : 'Sync Now'}
          </button>
          {lastSyncAt && (
            <p className="text-xs text-gray-500 text-center mt-2">
              Last full sync: {new Date(lastSyncAt).toLocaleString()}
            </p>
          )}
        </div>

        {/* Logout */}
        <div className="p-4 border-t border-gray-200">
          <button
            onClick={logout}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-gray-600 hover:bg-gray-100 hover:text-gray-900 transition-colors"
          >
            <LogOut size={16} />
            Logout
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>

      {/* AI Chat Panel */}
      <ChatPanel />
    </div>
  );
}
