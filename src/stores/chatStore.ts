import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ChatMessage, AppContext } from '../types/ai';
import { invoke } from '@tauri-apps/api/tauri';

interface ChatStore {
  isOpen: boolean;
  messages: ChatMessage[];
  isLoading: boolean;
  error: string | null;

  togglePanel: () => void;
  setOpen: (open: boolean) => void;
  sendMessage: (message: string, context: AppContext) => Promise<void>;
  clearMessages: () => void;
}

export const useChatStore = create<ChatStore>()(
  persist(
    (set, get) => ({
      isOpen: false,
      messages: [],
      isLoading: false,
      error: null,

      togglePanel: () => set((state) => ({ isOpen: !state.isOpen })),

      setOpen: (open: boolean) => set({ isOpen: open }),

      sendMessage: async (message: string, context: AppContext) => {
        const { messages } = get();

        // Add user message immediately
        const userMessage: ChatMessage = {
          role: 'user',
          content: message,
          timestamp: Date.now(),
        };

        set({
          messages: [...messages, userMessage],
          isLoading: true,
          error: null,
        });

        try {
          const response = await invoke<{ response: string }>('send_chat_message', {
            request: { message, context },
          });

          // Add assistant response
          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: response.response,
            timestamp: Date.now(),
          };

          set((state) => ({
            messages: [...state.messages, assistantMessage],
            isLoading: false,
          }));
        } catch (error) {
          console.error('Chat error:', error);
          set({
            error: error instanceof Error ? error.message : 'Unknown error',
            isLoading: false,
          });
        }
      },

      clearMessages: () => set({ messages: [] }),
    }),
    {
      name: 'chat-storage',
      partialize: (state) => ({ isOpen: state.isOpen }),
    }
  )
);
