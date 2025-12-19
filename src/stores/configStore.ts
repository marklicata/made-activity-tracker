import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/tauri';

interface Repository {
  owner: string;
  name: string;
  enabled: boolean;
}

interface Squad {
  id: string;
  name: string;
  members: string[]; // GitHub usernames
  color: string;
}

interface TrackedUser {
  username: string;
  tracked: boolean;
  tracked_at?: string | null;
}

interface LegacyTrackedUser {
  username: string;
  enabled: boolean;
}

// Match Rust AppConfig structure (new users shape + legacy tracked_users)
interface AppConfig {
  repositories: Repository[];
  squads: Squad[];
  users?: TrackedUser[];
  tracked_users?: LegacyTrackedUser[];
  history_days: number;
  excluded_bots: string[];
  bug_labels: string[];
  feature_labels: string[];
}

interface ConfigState {
  // Repos to track
  repositories: Repository[];
  
  // Team organization
  squads: Squad[];
  
  // Users to track
  trackedUsers: TrackedUser[];
  
  // Sync settings
  historyDays: number;
  excludedBots: string[];
  
  // Labels for categorization
  bugLabels: string[];
  featureLabels: string[];
  
  // Actions
  addRepository: (owner: string, name: string) => void;
  removeRepository: (owner: string, name: string) => void;
  toggleRepository: (owner: string, name: string) => void;
  
  addSquad: (squad: Omit<Squad, 'id'>) => void;
  updateSquad: (id: string, updates: Partial<Squad>) => void;
  removeSquad: (id: string) => void;
  
  addTrackedUser: (username: string) => void;
  removeTrackedUser: (username: string) => void;
  toggleTrackedUser: (username: string) => void;
  
  setHistoryDays: (days: number) => void;
  setExcludedBots: (bots: string[]) => void;
  setBugLabels: (labels: string[]) => void;
  setFeatureLabels: (labels: string[]) => void;
  
  loadConfig: () => Promise<void>;
  saveConfig: () => Promise<void>;
}

const normalizeUsername = (username: string) => username.trim().toLowerCase();

const mapTrackedUsersFromConfig = (config: AppConfig): TrackedUser[] => {
  const normalizedUsers = (config.users ?? []).map((user) => ({
    username: normalizeUsername(user.username),
    tracked: Boolean(user.tracked),
    tracked_at: user.tracked_at ?? null,
  }));

  const normalizedLegacy = (config.tracked_users ?? []).map((user) => ({
    username: normalizeUsername(user.username),
    tracked: Boolean(user.enabled),
    tracked_at: null,
  }));

  const merged = new Map<string, TrackedUser>();
  normalizedUsers.forEach((user) => merged.set(user.username, user));
  normalizedLegacy.forEach((user) => {
    if (!merged.has(user.username)) {
      merged.set(user.username, user);
    }
  });

  return Array.from(merged.values());
};

export const useConfigStore = create<ConfigState>()(
  persist(
    (set, get) => ({
      repositories: [],
      squads: [],
      trackedUsers: [],
      historyDays: 90,
      excludedBots: [
        'dependabot[bot]',
        'dependabot-preview[bot]',
        'renovate[bot]',
        'github-actions[bot]',
        'codecov[bot]',
      ],
      bugLabels: ['bug', 'defect', 'fix'],
      featureLabels: ['feature', 'enhancement', 'feat'],

      addRepository: (owner, name) => {
        const repos = get().repositories;
        if (!repos.find((r) => r.owner === owner && r.name === name)) {
          set({ repositories: [...repos, { owner, name, enabled: true }] });
          get().saveConfig();
        }
      },

      removeRepository: (owner, name) => {
        set({
          repositories: get().repositories.filter(
            (r) => !(r.owner === owner && r.name === name)
          ),
        });
        get().saveConfig();
      },

      toggleRepository: (owner, name) => {
        set({
          repositories: get().repositories.map((r) =>
            r.owner === owner && r.name === name
              ? { ...r, enabled: !r.enabled }
              : r
          ),
        });
        get().saveConfig();
      },

      addSquad: (squad) => {
        const id = crypto.randomUUID();
        set({ squads: [...get().squads, { ...squad, id }] });
        get().saveConfig();
      },

      updateSquad: (id, updates) => {
        set({
          squads: get().squads.map((s) =>
            s.id === id ? { ...s, ...updates } : s
          ),
        });
        get().saveConfig();
      },

      removeSquad: (id) => {
        set({ squads: get().squads.filter((s) => s.id !== id) });
        get().saveConfig();
      },

      addTrackedUser: (username) => {
        const normalized = normalizeUsername(username);
        if (!normalized) return;

        const current = get().trackedUsers;
        if (current.some((user) => user.username === normalized)) return;

        set({ trackedUsers: [...current, { username: normalized, tracked: true, tracked_at: new Date().toISOString() }] });
        get().saveConfig();
      },

      removeTrackedUser: (username) => {
        const normalized = normalizeUsername(username);
        set({ trackedUsers: get().trackedUsers.filter((u) => u.username !== normalized) });
        get().saveConfig();
      },

      toggleTrackedUser: (username) => {
        const normalized = normalizeUsername(username);
        set({
          trackedUsers: get().trackedUsers.map((u) =>
            u.username === normalized
              ? {
                  ...u,
                  tracked: !u.tracked,
                  tracked_at: !u.tracked ? u.tracked_at ?? new Date().toISOString() : u.tracked_at,
                }
              : u
          ),
        });
        get().saveConfig();
      },

      setHistoryDays: (days) => {
        set({ historyDays: days });
        get().saveConfig();
      },

      setExcludedBots: (bots) => {
        set({ excludedBots: bots });
        get().saveConfig();
      },

      setBugLabels: (labels) => {
        set({ bugLabels: labels });
        get().saveConfig();
      },

      setFeatureLabels: (labels) => {
        set({ featureLabels: labels });
        get().saveConfig();
      },

      loadConfig: async () => {
        try {
          const config = await invoke<AppConfig>('load_config');
          if (config) {
            const mergedTrackedUsers = mapTrackedUsersFromConfig(config);
            set({
              repositories: config.repositories || [],
              squads: config.squads || [],
              trackedUsers: mergedTrackedUsers,
              historyDays: config.history_days || 90,
              excludedBots: config.excluded_bots || [],
              bugLabels: config.bug_labels || [],
              featureLabels: config.feature_labels || [],
            });
          }
        } catch (error) {
          console.error('Failed to load config:', error);
        }
      },

      saveConfig: async () => {
        try {
          const { repositories, squads, trackedUsers, historyDays, excludedBots, bugLabels, featureLabels } = get();
          const usersPayload = trackedUsers.map(({ username, tracked, tracked_at }) => ({
            username,
            tracked,
            tracked_at: tracked_at ?? null,
          }));
          const legacyTrackedUsers = trackedUsers.map(({ username, tracked }) => ({
            username,
            enabled: tracked,
          }));

          // Map to Rust's snake_case structure (users is preferred; tracked_users kept for backward compatibility)
          const config: AppConfig = {
            repositories,
            squads,
            users: usersPayload,
            tracked_users: legacyTrackedUsers,
            history_days: historyDays,
            excluded_bots: excludedBots,
            bug_labels: bugLabels,
            feature_labels: featureLabels,
          };
          await invoke('save_config', { config });
        } catch (error) {
          console.error('Failed to save config:', error);
        }
      },
    }),
    {
      name: 'made-config',
    }
  )
);
