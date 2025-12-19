import { useState, useEffect } from 'react';
import { 
  Plus, 
  Trash2, 
  Users, 
  User, 
  GitBranch, 
  Clock, 
  Bot,
  Tag,
  Loader2
} from 'lucide-react';
import { useConfigStore } from '@stores/configStore';

export default function Settings() {
  const {
    repositories,
    squads,
    trackedUsers,
    historyDays,
    excludedBots,
    bugLabels,
    featureLabels,
    addRepository,
    removeRepository,
    toggleRepository,
    addSquad,
    removeSquad,
    addTrackedUser,
    removeTrackedUser,
    toggleTrackedUser,
    setHistoryDays,
    setExcludedBots,
    setBugLabels,
    setFeatureLabels,
    loadConfig,
  } = useConfigStore();

  const [loading, setLoading] = useState(true);
  const [newRepo, setNewRepo] = useState('');
  const [newSquadName, setNewSquadName] = useState('');
  const [newSquadMembers, setNewSquadMembers] = useState('');
  const [newUser, setNewUser] = useState('');
  const [newBot, setNewBot] = useState('');
  const [newBugLabel, setNewBugLabel] = useState('');
  const [newFeatureLabel, setNewFeatureLabel] = useState('');

  useEffect(() => {
    loadConfig().finally(() => setLoading(false));
  }, [loadConfig]);

  const handleAddRepo = () => {
    const match = newRepo.match(/^([^/]+)\/([^/]+)$/);
    if (match) {
      addRepository(match[1], match[2]);
      setNewRepo('');
    }
  };

  const handleAddSquad = () => {
    if (newSquadName && newSquadMembers) {
      addSquad({
        name: newSquadName,
        members: newSquadMembers.split(',').map(m => m.trim()),
        color: `#${Math.floor(Math.random()*16777215).toString(16).padStart(6, '0')}`,
      });
      setNewSquadName('');
      setNewSquadMembers('');
    }
  };

  const handleAddUser = () => {
    const username = newUser.trim().toLowerCase();
    if (!username) return;
    if (trackedUsers.some((user) => user.username === username)) return;
    addTrackedUser(username);
    setNewUser('');
  };

  const handleAddBot = () => {
    if (newBot && !excludedBots.includes(newBot)) {
      setExcludedBots([...excludedBots, newBot]);
      setNewBot('');
    }
  };

  const handleAddBugLabel = () => {
    if (newBugLabel && !bugLabels.includes(newBugLabel)) {
      setBugLabels([...bugLabels, newBugLabel]);
      setNewBugLabel('');
    }
  };

  const handleAddFeatureLabel = () => {
    if (newFeatureLabel && !featureLabels.includes(newFeatureLabel)) {
      setFeatureLabels([...featureLabels, newFeatureLabel]);
      setNewFeatureLabel('');
    }
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="flex items-center gap-2 text-gray-400">
          <Loader2 className="animate-spin" size={20} />
          Loading settings...
        </div>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-4xl">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
        <p className="text-gray-500">Configure repositories, teams, and tracking options</p>
      </div>

      {/* Repositories */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <GitBranch size={20} className="text-blue-600" />
          Repositories
        </h2>
        <div className="space-y-3 mb-4">
          {repositories.length === 0 && (
            <p className="text-gray-500 text-sm">No repositories configured. Add one below.</p>
          )}
          {repositories.map((repo) => (
            <div key={`${repo.owner}/${repo.name}`} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={repo.enabled}
                  onChange={() => toggleRepository(repo.owner, repo.name)}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="font-mono text-sm">{repo.owner}/{repo.name}</span>
              </div>
              <button
                onClick={() => removeRepository(repo.owner, repo.name)}
                className="p-1 text-gray-400 hover:text-red-600 transition-colors"
              >
                <Trash2 size={16} />
              </button>
            </div>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            value={newRepo}
            onChange={(e) => setNewRepo(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAddRepo()}
            placeholder="owner/repository"
            className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleAddRepo}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-1"
          >
            <Plus size={16} />
            Add
          </button>
        </div>
      </section>

      {/* Tracked Users */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <User size={20} className="text-blue-600" />
          Tracked Users
        </h2>
        <div className="space-y-3 mb-4">
          {trackedUsers.length === 0 && (
            <p className="text-gray-500 text-sm">No tracked users configured. Add one below.</p>
          )}
          {trackedUsers.map((user) => (
            <div key={user.username} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={user.enabled}
                  onChange={() => toggleTrackedUser(user.username)}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="font-mono text-sm">{user.username}</span>
              </div>
              <button
                onClick={() => removeTrackedUser(user.username)}
                className="p-1 text-gray-400 hover:text-red-600 transition-colors"
              >
                <Trash2 size={16} />
              </button>
            </div>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            value={newUser}
            onChange={(e) => setNewUser(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAddUser()}
            placeholder="github-username"
            className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleAddUser}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors flex items-center gap-1"
          >
            <Plus size={16} />
            Add
          </button>
        </div>
      </section>

      {/* Squads */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <Users size={20} className="text-blue-600" />
          Squads
        </h2>
        <div className="space-y-3 mb-4">
          {squads.length === 0 && (
            <p className="text-gray-500 text-sm">No squads configured. Add one below.</p>
          )}
          {squads.map((squad) => (
            <div key={squad.id} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-3">
                <div 
                  className="w-3 h-3 rounded-full" 
                  style={{ backgroundColor: squad.color }}
                />
                <span className="font-medium">{squad.name}</span>
                <span className="text-sm text-gray-500">
                  ({squad.members.join(', ')})
                </span>
              </div>
              <button
                onClick={() => removeSquad(squad.id)}
                className="p-1 text-gray-400 hover:text-red-600 transition-colors"
              >
                <Trash2 size={16} />
              </button>
            </div>
          ))}
        </div>
        <div className="grid grid-cols-2 gap-2">
          <input
            type="text"
            value={newSquadName}
            onChange={(e) => setNewSquadName(e.target.value)}
            placeholder="Squad name"
            className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <div className="flex gap-2">
            <input
              type="text"
              value={newSquadMembers}
              onChange={(e) => setNewSquadMembers(e.target.value)}
              placeholder="user1, user2, user3"
              className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              onClick={handleAddSquad}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
            >
              <Plus size={16} />
            </button>
          </div>
        </div>
      </section>

      {/* History Days */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <Clock size={20} className="text-blue-600" />
          History Range
        </h2>
        <div className="flex items-center gap-4">
          <input
            type="range"
            min="30"
            max="365"
            step="30"
            value={historyDays}
            onChange={(e) => setHistoryDays(Number(e.target.value))}
            className="flex-1"
          />
          <span className="text-lg font-medium w-24 text-right">{historyDays} days</span>
        </div>
        <p className="text-sm text-gray-500 mt-2">
          How far back to fetch GitHub data on sync
        </p>
      </section>

      {/* Excluded Bots */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <Bot size={20} className="text-blue-600" />
          Excluded Bots
        </h2>
        <div className="flex flex-wrap gap-2 mb-4">
          {excludedBots.map((bot) => (
            <span key={bot} className="inline-flex items-center gap-1 px-3 py-1 bg-gray-100 rounded-full text-sm">
              {bot}
              <button
                onClick={() => setExcludedBots(excludedBots.filter(b => b !== bot))}
                className="text-gray-400 hover:text-red-600"
              >
                <Trash2 size={12} />
              </button>
            </span>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            type="text"
            value={newBot}
            onChange={(e) => setNewBot(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleAddBot()}
            placeholder="bot-name[bot]"
            className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleAddBot}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
          >
            <Plus size={16} />
          </button>
        </div>
      </section>

      {/* Labels */}
      <section className="mb-8 bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
          <Tag size={20} className="text-blue-600" />
          Label Configuration
        </h2>
        
        {/* Bug Labels */}
        <div className="mb-6">
          <h3 className="text-sm font-medium text-gray-700 mb-2">Bug Labels</h3>
          <div className="flex flex-wrap gap-2 mb-2">
            {bugLabels.map((label) => (
              <span key={label} className="inline-flex items-center gap-1 px-3 py-1 bg-red-100 text-red-700 rounded-full text-sm">
                {label}
                <button
                  onClick={() => setBugLabels(bugLabels.filter(l => l !== label))}
                  className="text-red-400 hover:text-red-600"
                >
                  <Trash2 size={12} />
                </button>
              </span>
            ))}
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={newBugLabel}
              onChange={(e) => setNewBugLabel(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddBugLabel()}
              placeholder="bug label"
              className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button onClick={handleAddBugLabel} className="px-4 py-2 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700 transition-colors">
              <Plus size={16} />
            </button>
          </div>
        </div>

        {/* Feature Labels */}
        <div>
          <h3 className="text-sm font-medium text-gray-700 mb-2">Feature Labels</h3>
          <div className="flex flex-wrap gap-2 mb-2">
            {featureLabels.map((label) => (
              <span key={label} className="inline-flex items-center gap-1 px-3 py-1 bg-green-100 text-green-700 rounded-full text-sm">
                {label}
                <button
                  onClick={() => setFeatureLabels(featureLabels.filter(l => l !== label))}
                  className="text-green-400 hover:text-green-600"
                >
                  <Trash2 size={12} />
                </button>
              </span>
            ))}
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={newFeatureLabel}
              onChange={(e) => setNewFeatureLabel(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddFeatureLabel()}
              placeholder="feature label"
              className="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button onClick={handleAddFeatureLabel} className="px-4 py-2 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 transition-colors">
              <Plus size={16} />
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
