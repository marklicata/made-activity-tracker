import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/tauri';

interface AuthState {
  isAuthenticated: boolean;
  user: GitHubUser | null;
  accessToken: string | null;
  
  // Actions
  login: () => Promise<void>;
  logout: () => void;
  checkAuth: () => Promise<void>;
}

interface GitHubUser {
  id: number;
  login: string;
  name: string | null;
  avatar_url: string;
}

// Check if running in frontend-only mode (no Tauri backend)
const isFrontendOnly = typeof window !== 'undefined' && !(window as any).__TAURI__;

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      // In frontend-only mode, bypass auth for development
      isAuthenticated: isFrontendOnly,
      user: isFrontendOnly ? {
        id: 1,
        login: 'dev-user',
        name: 'Development User',
        avatar_url: 'https://github.com/github.png',
      } : null,
      accessToken: null,

      login: async () => {
        try {
          // Trigger GitHub Device Flow via Tauri backend
          const result = await invoke<{ user: GitHubUser; access_token: string }>('github_login');
          
          set({
            isAuthenticated: true,
            user: result.user,
            accessToken: result.access_token,
          });
        } catch (error) {
          console.error('Login failed:', error);
          throw error;
        }
      },

      logout: () => {
        // Clear token from secure storage via Tauri
        invoke('github_logout').catch(console.error);
        
        set({
          isAuthenticated: false,
          user: null,
          accessToken: null,
        });
      },

      checkAuth: async () => {
        try {
          const result = await invoke<{ user: GitHubUser; access_token: string } | null>('check_auth');
          
          if (result) {
            set({
              isAuthenticated: true,
              user: result.user,
              accessToken: result.access_token,
            });
          } else {
            set({
              isAuthenticated: false,
              user: null,
              accessToken: null,
            });
          }
        } catch (error) {
          console.error('Auth check failed:', error);
          set({
            isAuthenticated: false,
            user: null,
            accessToken: null,
          });
        }
      },
    }),
    {
      name: 'made-auth',
      partialize: (state) => ({
        // Only persist non-sensitive data
        isAuthenticated: state.isAuthenticated,
        user: state.user,
      }),
    }
  )
);
