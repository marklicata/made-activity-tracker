import '@testing-library/jest-dom';
import { setupTauriMocks, setupTauriEventMocks } from './mocks/tauri';
import { beforeEach, afterEach, vi } from 'vitest';

// Setup Tauri mocks before each test
beforeEach(() => {
  setupTauriMocks();
  setupTauriEventMocks();
});

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
});
