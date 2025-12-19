export interface AppContext {
  current_page: string;
  filters: FilterState;
}

export interface FilterState {
  date_range?: {
    start: string;
    end: string;
  };
  repositories: string[];
  squads: string[];
  users: string[];
}

export interface ChatRequest {
  message: string;
  context: AppContext;
}

export interface ChatResponse {
  response: string;
  context: AppContext;
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}
