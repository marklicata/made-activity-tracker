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

// Match Rust AppConfig structure
interface AppConfig {
  repositories: Repository[];
  squads: Squad[];
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
  
  setHistoryDays: (days: number) => void;
  setExcludedBots: (bots: string[]) => void;
  setBugLabels: (labels: string[]) => void;
  setFeatureLabels: (labels: string[]) => void;
  
  loadConfig: () => Promise<void>;
  saveConfig: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>()(
  persist(
    (set, get) => ({
      repositories: [],
      squads: [],
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
            set({
              repositories: config.repositories || [],
              squads: config.squads || [],
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
          const { repositories, squads, historyDays, excludedBots, bugLabels, featureLabels } = get();
          // Map to Rust's snake_case structure
          const config: AppConfig = {
            repositories,
            squads,
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
