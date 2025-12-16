/**
 * Mock implementations for Tauri API
 * 
 * Used in frontend tests to simulate Tauri backend responses
 */

import { vi } from 'vitest';

// Mock Tauri invoke function
export const mockInvoke = vi.fn();

// Mock responses for each command
export const mockResponses = {
  // Auth commands
  github_login: {
    user: {
      id: 1,
      login: 'testuser',
      name: 'Test User',
      avatar_url: 'https://github.com/testuser.png',
    },
    access_token: 'mock_token_123',
  },
  
  check_auth: null, // Not authenticated by default
  
  // Sync commands
  sync_github_data: undefined, // Returns void
  
  // Config commands
  load_config: {
    repositories: [
      { owner: 'org', name: 'repo1', enabled: true },
      { owner: 'org', name: 'repo2', enabled: true },
    ],
    squads: [],
    historyDays: 90,
    excludedBots: ['dependabot[bot]'],
    bugLabels: ['bug'],
    featureLabels: ['feature'],
  },
  
  save_config: undefined,
  
  // Metrics commands
  get_dashboard_metrics: {
    speed: {
      avg_cycle_time_days: 3.2,
      avg_pr_lead_time_hours: 18.5,
      throughput_per_week: 12,
      trend: 8,
    },
    ease: {
      avg_pr_size_lines: 245,
      avg_review_rounds: 1.4,
      avg_time_to_first_review_hours: 4.2,
      rework_rate: 0.12,
    },
    quality: {
      bug_rate: 0.08,
      reopen_rate: 0.05,
      pr_rejection_rate: 0.03,
      test_coverage_trend: 2,
    },
  },
  
  // Search commands
  hybrid_search: [],
  find_duplicates: [],
  
  // Roadmap commands
  get_roadmap: [],
};

// Setup function to configure mock invoke
export function setupTauriMocks() {
  mockInvoke.mockImplementation((command: string, args?: any) => {
    const response = mockResponses[command as keyof typeof mockResponses];
    return Promise.resolve(response);
  });
  
  // Mock the @tauri-apps/api/tauri module
  vi.mock('@tauri-apps/api/tauri', () => ({
    invoke: mockInvoke,
  }));
}

// Helper to override specific mock responses
export function setMockResponse(command: string, response: any) {
  (mockResponses as any)[command] = response;
}

// Helper to simulate Tauri events
export const mockEmit = vi.fn();
export const mockListen = vi.fn();

export function setupTauriEventMocks() {
  vi.mock('@tauri-apps/api/event', () => ({
    emit: mockEmit,
    listen: mockListen,
  }));
}
