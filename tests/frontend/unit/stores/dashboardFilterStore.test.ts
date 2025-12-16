/**
 * Unit tests for Dashboard Filter Store
 *
 * Tests for:
 * - Filter state management
 * - Squad/user mutual exclusivity
 * - Date range presets
 * - Clear filters functionality
 * - Persistence
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';
import { DATE_RANGE_PRESETS } from '@types/filters';

describe('Dashboard Filter Store', () => {
  beforeEach(() => {
    // Reset store to initial state
    const store = useDashboardFilterStore.getState();
    store.clearFilters();

    // Clear localStorage mock
    localStorage.clear();
  });

  describe('Initial State', () => {
    it('should have a default 90-day date range', () => {
      const { filters } = useDashboardFilterStore.getState();

      expect(filters.dateRange).toBeDefined();
      expect(filters.dateRange?.start).toBeDefined();
      expect(filters.dateRange?.end).toBeDefined();
    });

    it('should have no other filters set initially', () => {
      const { filters } = useDashboardFilterStore.getState();

      expect(filters.repositoryIds).toBeUndefined();
      expect(filters.squadId).toBeUndefined();
      expect(filters.userId).toBeUndefined();
    });

    it('should report no active filters initially', () => {
      const { hasActiveFilters } = useDashboardFilterStore.getState();

      expect(hasActiveFilters()).toBe(false);
    });
  });

  describe('Date Range Filtering', () => {
    it('should set custom date range', () => {
      const store = useDashboardFilterStore.getState();
      const customRange = {
        start: '2024-01-01T00:00:00Z',
        end: '2024-01-31T00:00:00Z',
      };

      store.setDateRange(customRange);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toEqual(customRange);
    });

    it('should clear date range when passed null', () => {
      const store = useDashboardFilterStore.getState();

      store.setDateRange(null);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeUndefined();
    });

    it('should set date range from preset (7 days)', () => {
      const store = useDashboardFilterStore.getState();

      store.setDateRangePreset(DATE_RANGE_PRESETS.LAST_7_DAYS);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeDefined();

      // Verify it's approximately 7 days
      const start = new Date(filters.dateRange!.start);
      const end = new Date(filters.dateRange!.end);
      const diffDays = (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24);

      expect(diffDays).toBeGreaterThanOrEqual(6);
      expect(diffDays).toBeLessThanOrEqual(8);
    });

    it('should set date range from preset (90 days)', () => {
      const store = useDashboardFilterStore.getState();

      store.setDateRangePreset(DATE_RANGE_PRESETS.LAST_90_DAYS);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeDefined();

      // Verify it's approximately 90 days
      const start = new Date(filters.dateRange!.start);
      const end = new Date(filters.dateRange!.end);
      const diffDays = (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24);

      expect(diffDays).toBeGreaterThanOrEqual(89);
      expect(diffDays).toBeLessThanOrEqual(91);
    });
  });

  describe('Repository Filtering', () => {
    it('should set repository filter with multiple repos', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2, 3]);

      const { filters, hasActiveFilters } = useDashboardFilterStore.getState();
      expect(filters.repositoryIds).toEqual([1, 2, 3]);
      expect(hasActiveFilters()).toBe(true);
    });

    it('should clear repository filter when passed null', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2]);
      store.setRepositories(null);

      const { filters, hasActiveFilters } = useDashboardFilterStore.getState();
      expect(filters.repositoryIds).toBeUndefined();
      expect(hasActiveFilters()).toBe(false);
    });

    it('should clear repository filter when passed empty array', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2]);
      store.setRepositories([]);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.repositoryIds).toBeUndefined();
    });
  });

  describe('Squad/User Mutual Exclusivity', () => {
    it('should clear user when setting squad', () => {
      const store = useDashboardFilterStore.getState();

      // Set user first
      store.setUser(42);
      expect(useDashboardFilterStore.getState().filters.userId).toBe(42);

      // Set squad - should clear user
      store.setSquad('frontend');

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.squadId).toBe('frontend');
      expect(filters.userId).toBeUndefined();
    });

    it('should clear squad when setting user', () => {
      const store = useDashboardFilterStore.getState();

      // Set squad first
      store.setSquad('backend');
      expect(useDashboardFilterStore.getState().filters.squadId).toBe('backend');

      // Set user - should clear squad
      store.setUser(42);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.userId).toBe(42);
      expect(filters.squadId).toBeUndefined();
    });

    it('should allow clearing squad without setting user', () => {
      const store = useDashboardFilterStore.getState();

      store.setSquad('frontend');
      store.setSquad(null);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.squadId).toBeUndefined();
      expect(filters.userId).toBeUndefined();
    });

    it('should allow clearing user without setting squad', () => {
      const store = useDashboardFilterStore.getState();

      store.setUser(42);
      store.setUser(null);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.userId).toBeUndefined();
      expect(filters.squadId).toBeUndefined();
    });
  });

  describe('Clear Filters', () => {
    it('should clear all filters except date range', () => {
      const store = useDashboardFilterStore.getState();

      // Set all filters
      store.setRepositories([1, 2, 3]);
      store.setSquad('frontend');

      // Clear filters
      store.clearFilters();

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeDefined(); // Should keep default 90-day range
      expect(filters.repositoryIds).toBeUndefined();
      expect(filters.squadId).toBeUndefined();
      expect(filters.userId).toBeUndefined();
    });

    it('should restore 90-day default date range', () => {
      const store = useDashboardFilterStore.getState();

      // Set custom date range
      store.setDateRange({
        start: '2024-01-01T00:00:00Z',
        end: '2024-01-07T00:00:00Z',
      });

      // Clear filters
      store.clearFilters();

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeDefined();

      // Verify it's approximately 90 days
      const start = new Date(filters.dateRange!.start);
      const end = new Date(filters.dateRange!.end);
      const diffDays = (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24);

      expect(diffDays).toBeGreaterThanOrEqual(89);
      expect(diffDays).toBeLessThanOrEqual(91);
    });
  });

  describe('hasActiveFilters', () => {
    it('should return false when only date range is set', () => {
      const { hasActiveFilters } = useDashboardFilterStore.getState();

      expect(hasActiveFilters()).toBe(false);
    });

    it('should return true when repositories are set', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1]);

      expect(store.hasActiveFilters()).toBe(true);
    });

    it('should return true when squad is set', () => {
      const store = useDashboardFilterStore.getState();

      store.setSquad('frontend');

      expect(store.hasActiveFilters()).toBe(true);
    });

    it('should return true when user is set', () => {
      const store = useDashboardFilterStore.getState();

      store.setUser(42);

      expect(store.hasActiveFilters()).toBe(true);
    });

    it('should return true when multiple filters are set', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2]);
      store.setSquad('backend');

      expect(store.hasActiveFilters()).toBe(true);
    });

    it('should return false after clearing all filters', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2]);
      store.setSquad('backend');
      store.clearFilters();

      expect(store.hasActiveFilters()).toBe(false);
    });
  });

  describe('Combined Filter Scenarios', () => {
    it('should handle setting all compatible filters', () => {
      const store = useDashboardFilterStore.getState();

      store.setDateRangePreset(DATE_RANGE_PRESETS.LAST_30_DAYS);
      store.setRepositories([1, 2, 3]);
      store.setSquad('frontend');

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.dateRange).toBeDefined();
      expect(filters.repositoryIds).toEqual([1, 2, 3]);
      expect(filters.squadId).toBe('frontend');
      expect(filters.userId).toBeUndefined();
    });

    it('should handle changing between squad and user', () => {
      const store = useDashboardFilterStore.getState();

      store.setSquad('frontend');
      expect(useDashboardFilterStore.getState().filters.squadId).toBe('frontend');

      store.setUser(42);
      expect(useDashboardFilterStore.getState().filters.userId).toBe(42);
      expect(useDashboardFilterStore.getState().filters.squadId).toBeUndefined();

      store.setSquad('backend');
      expect(useDashboardFilterStore.getState().filters.squadId).toBe('backend');
      expect(useDashboardFilterStore.getState().filters.userId).toBeUndefined();
    });

    it('should maintain repository filter when switching squad/user', () => {
      const store = useDashboardFilterStore.getState();

      store.setRepositories([1, 2, 3]);
      store.setSquad('frontend');

      expect(useDashboardFilterStore.getState().filters.repositoryIds).toEqual([1, 2, 3]);

      store.setUser(42);

      const { filters } = useDashboardFilterStore.getState();
      expect(filters.repositoryIds).toEqual([1, 2, 3]);
      expect(filters.userId).toBe(42);
      expect(filters.squadId).toBeUndefined();
    });
  });
});
