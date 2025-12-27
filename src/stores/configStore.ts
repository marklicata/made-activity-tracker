import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

// Match SQLite database models
interface Repository {
  id: number;
  owner: string;
  name: string;
  github_id?: number;
  enabled: boolean;
  last_synced_at?: string;
}

interface Squad {
  id: string;
  name: string;
  members: string[];
  color?: string;
}

interface User {
  id: number;
  github_id: number;
  login: string;
  name?: string;
  avatar_url?: string;
  is_bot: boolean;
  tracked: boolean;
  tracked_at?: string;
}

interface Settings {
  id: number;
  history_days: number;
  excluded_bots: string[];
  bug_labels: string[];
  feature_labels: string[];
  created_at: string;
  updated_at: string;
}

interface ConfigState {
  // Data
  repositories: Repository[];
  squads: Squad[];
  users: User[];
  settings: Settings | null;

  // Loading state
  isLoading: boolean;

  // Actions
  loadAll: () => Promise<void>;

  // Repository actions
  addRepository: (owner: string, name: string) => Promise<void>;
  removeRepository: (owner: string, name: string) => Promise<void>;
  toggleRepository: (owner: string, name: string) => Promise<void>;

  // Squad actions
  addSquad: (name: string, members: string[], color: string) => Promise<void>;
  updateSquad: (
    id: string,
    updates: { name?: string; members?: string[]; color?: string }
  ) => Promise<void>;
  removeSquad: (id: string) => Promise<void>;

  // User actions
  toggleUserTracked: (username: string) => Promise<void>;

  // Settings actions
  updateSettings: (settings: {
    history_days?: number;
    excluded_bots?: string[];
    bug_labels?: string[];
    feature_labels?: string[];
  }) => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  repositories: [],
  squads: [],
  users: [],
  settings: null,
  isLoading: false,

  loadAll: async () => {
    set({ isLoading: true });
    try {
      const [repositories, squads, users, settings] = await Promise.all([
        invoke<Repository[]>('get_all_repositories'),
        invoke<Squad[]>('get_all_squads_command'),
        invoke<User[]>('get_all_users'),
        invoke<Settings>('get_settings'),
      ]);

      set({ repositories, squads, users, settings, isLoading: false });
    } catch (error) {
      console.error('Failed to load config:', error);
      set({ isLoading: false });
    }
  },

  addRepository: async (owner, name) => {
    try {
      await invoke('add_repository', { owner, name });
      await get().loadAll(); // Reload to get fresh data
    } catch (error) {
      console.error('Failed to add repository:', error);
      throw error;
    }
  },

  removeRepository: async (owner, name) => {
    try {
      await invoke('remove_repository', { owner, name });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to remove repository:', error);
      throw error;
    }
  },

  toggleRepository: async (owner, name) => {
    try {
      await invoke('toggle_repository', { owner, name });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to toggle repository:', error);
      throw error;
    }
  },

  addSquad: async (name, members, color) => {
    try {
      await invoke('add_squad', { name, members, color });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to add squad:', error);
      throw error;
    }
  },

  updateSquad: async (id, updates) => {
    try {
      await invoke('update_squad', { id, ...updates });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to update squad:', error);
      throw error;
    }
  },

  removeSquad: async (id) => {
    try {
      await invoke('remove_squad', { id });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to remove squad:', error);
      throw error;
    }
  },

  toggleUserTracked: async (username) => {
    try {
      await invoke('toggle_user_tracked', { username });
      await get().loadAll();
    } catch (error) {
      console.error('Failed to toggle user tracked:', error);
      throw error;
    }
  },

  updateSettings: async (updates) => {
    const current = get().settings;
    if (!current) return;

    try {
      await invoke('update_settings', {
        history_days: updates.history_days ?? current.history_days,
        excluded_bots: updates.excluded_bots ?? current.excluded_bots,
        bug_labels: updates.bug_labels ?? current.bug_labels,
        feature_labels: updates.feature_labels ?? current.feature_labels,
      });

      await get().loadAll();
    } catch (error) {
      console.error('Failed to update settings:', error);
      throw error;
    }
  },
}));
