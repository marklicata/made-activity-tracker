import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';

interface SyncProgress {
  phase: 'issues' | 'pull_requests' | 'milestones' | 'embeddings';
  current: number;
  total: number;
  message: string;
}

interface SyncState {
  isSyncing: boolean;
  lastSyncAt: string | null;
  progress: SyncProgress | null;
  error: string | null;

  // Actions
  triggerSync: () => Promise<void>;
  setProgress: (progress: SyncProgress | null) => void;
  setError: (error: string | null) => void;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  isSyncing: false,
  lastSyncAt: null,
  progress: null,
  error: null,

  triggerSync: async () => {
    if (get().isSyncing) return;

    set({ isSyncing: true, error: null, progress: null });

    try {
      // Call Tauri backend to sync
      await invoke('sync_github_data');
      
      set({
        isSyncing: false,
        lastSyncAt: new Date().toISOString(),
        progress: null,
      });
    } catch (error) {
      console.error('Sync failed:', error);
      set({
        isSyncing: false,
        error: error instanceof Error ? error.message : 'Sync failed',
        progress: null,
      });
    }
  },

  setProgress: (progress) => set({ progress }),
  
  setError: (error) => set({ error }),
}));
