/**
 * Unit tests for usePRMetrics hook
 *
 * Tests for:
 * - Fetches data on mount
 * - Updates when days parameter changes
 * - Handles loading state correctly
 * - Handles error state correctly
 * - Auto-refresh works if enabled
 * - Cleanup on unmount
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { usePRMetrics } from '@/hooks/usePRMetrics';
import type { DashboardMetrics } from '@/types/metrics';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}));

describe('usePRMetrics', () => {
  const mockMetrics: DashboardMetrics = {
    overview: {
      productivity_multiplier: 2.5,
      period_days: 30,
      total_prs: 150,
      active_developers: 5,
    },
    speed: {
      prs_per_day: 5.0,
      prs_per_day_per_dev: 1.0,
      pr_turnaround_hours: 12.5,
      loc_per_day: 1500,
      cycle_time_distribution: {
        under_4h: 30,
        under_4h_pct: 20.0,
        h4_to_12: 60,
        h4_to_12_pct: 40.0,
        h12_to_24: 45,
        h12_to_24_pct: 30.0,
        over_24h: 15,
        over_24h_pct: 10.0,
      },
      benchmark_comparison: {
        prs_per_day_industry: 0.8,
        prs_per_day_elite: 1.5,
        pr_turnaround_industry: 89.0,
        pr_turnaround_elite: 24.0,
      },
    },
    ease: {
      concurrent_repos: 3,
      repos_per_dev: 1.5,
      total_active_repos: 8,
      active_repos: [],
      repo_distribution: {
        org_repos: 5,
        org_repos_pct: 62.5,
        personal_repos: 3,
        personal_repos_pct: 37.5,
      },
      work_pattern: [],
      pr_switch_frequency: 45.5,
      benchmark_comparison: {
        concurrent_repos_industry: 2.1,
        concurrent_repos_elite: 3.5,
      },
    },
    quality: {
      pr_merge_rate: 85.5,
      avg_files_per_pr: 6.2,
      bug_pr_percentage: 20.0,
      feature_pr_percentage: 60.0,
      avg_review_cycle_hours: 4.5,
      avg_review_comments: 3.2,
      pr_type_distribution: [
        { pr_type: 'feature', count: 60, percentage: 60.0 },
      ],
      files_per_pr_distribution: {
        range_1_3: 40,
        range_1_3_pct: 40.0,
        range_4_8: 35,
        range_4_8_pct: 35.0,
        range_9_15: 15,
        range_9_15_pct: 15.0,
        range_16_plus: 10,
        range_16_plus_pct: 10.0,
      },
      merge_rate_trend: [],
      benchmark_comparison: {
        merge_rate_industry: 68.0,
        merge_rate_elite: 85.0,
        bug_ratio_industry: 25.0,
        bug_ratio_elite: 15.0,
        files_per_pr_industry: 8.0,
      },
    },
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Initial Data Fetching', () => {
    it('fetches data on mount with default days', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      // Initially loading
      expect(result.current.loading).toBe(true);
      expect(result.current.metrics).toBe(null);
      expect(result.current.error).toBe(null);

      // Wait for fetch to complete
      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 30 });
      expect(result.current.metrics).toEqual(mockMetrics);
      expect(result.current.error).toBe(null);
    });

    it('fetches data with custom days', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics({ days: 90 }));

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 90 });
    });

    it('fetches data with 7 days', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics({ days: 7 }));

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 7 });
    });
  });

  describe('Loading State', () => {
    it('sets loading to true while fetching', () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves

      const { result } = renderHook(() => usePRMetrics());

      expect(result.current.loading).toBe(true);
    });

    it('sets loading to false after successful fetch', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });
    });

    it('sets loading to false after failed fetch', async () => {
      mockInvoke.mockRejectedValue(new Error('Fetch failed'));

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });
    });
  });

  describe('Error Handling', () => {
    it('handles fetch errors gracefully', async () => {
      const error = new Error('Network error');
      mockInvoke.mockRejectedValue(error);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.error).toBe('Network error');
      });

      expect(result.current.loading).toBe(false);
      expect(result.current.metrics).toBe(null);
    });

    it('handles string errors', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.error).toBe('String error');
      });
    });

    it('handles unknown errors', async () => {
      mockInvoke.mockRejectedValue({ unknown: 'error' });

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.error).toBe('[object Object]');
      });
    });

    it('clears error on successful retry', async () => {
      mockInvoke
        .mockRejectedValueOnce(new Error('First error'))
        .mockResolvedValueOnce(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.error).toBe('First error');
      });

      // Call refresh to retry
      await result.current.refresh();

      await waitFor(() => {
        expect(result.current.error).toBe(null);
        expect(result.current.metrics).toEqual(mockMetrics);
      });
    });
  });

  describe('Days Parameter Updates', () => {
    it('refetches when days parameter changes', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result, rerender } = renderHook(
        ({ days }) => usePRMetrics({ days }),
        { initialProps: { days: 30 } }
      );

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 30 });

      // Change days parameter
      rerender({ days: 90 });

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 90 });
      });

      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });

    it('does not refetch when days parameter stays the same', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result, rerender } = renderHook(
        ({ days }) => usePRMetrics({ days }),
        { initialProps: { days: 30 } }
      );

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Rerender with same days
      rerender({ days: 30 });

      // Should not fetch again
      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });
  });

  describe('Manual Refresh', () => {
    it('provides refresh function that fetches new data', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Call refresh manually
      await result.current.refresh();

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(2);
      });
    });

    it('sets loading state during manual refresh', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      // Start refresh (don't await)
      const refreshPromise = result.current.refresh();

      // Should be loading immediately
      expect(result.current.loading).toBe(true);

      await refreshPromise;

      expect(result.current.loading).toBe(false);
    });
  });

  describe('Auto-refresh', () => {
    it('does not auto-refresh by default', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance time
      vi.advanceTimersByTime(61000);

      // Should not have fetched again
      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it('auto-refreshes when enabled', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() =>
        usePRMetrics({ autoRefresh: true, refreshInterval: 10000 })
      );

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      expect(mockInvoke).toHaveBeenCalledTimes(1);

      // Advance time to trigger refresh
      vi.advanceTimersByTime(10000);

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(2);
      });
    });

    it('auto-refreshes multiple times', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result } = renderHook(() =>
        usePRMetrics({ autoRefresh: true, refreshInterval: 5000 })
      );

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      // First auto-refresh
      vi.advanceTimersByTime(5000);
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(2);
      });

      // Second auto-refresh
      vi.advanceTimersByTime(5000);
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(3);
      });
    });

    it('cleans up interval on unmount', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { result, unmount } = renderHook(() =>
        usePRMetrics({ autoRefresh: true, refreshInterval: 10000 })
      );

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });

      unmount();

      // Advance time - should not trigger fetch after unmount
      vi.advanceTimersByTime(10000);

      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });

    it('does not auto-refresh with interval <= 0', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      renderHook(() =>
        usePRMetrics({ autoRefresh: true, refreshInterval: 0 })
      );

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledTimes(1);
      });

      vi.advanceTimersByTime(100000);

      expect(mockInvoke).toHaveBeenCalledTimes(1);
    });
  });

  describe('Edge Cases', () => {
    it('handles empty metrics response', async () => {
      const emptyMetrics = {
        ...mockMetrics,
        overview: {
          productivity_multiplier: 0,
          period_days: 30,
          total_prs: 0,
          active_developers: 0,
        },
      };

      mockInvoke.mockResolvedValue(emptyMetrics);

      const { result } = renderHook(() => usePRMetrics());

      await waitFor(() => {
        expect(result.current.metrics).toEqual(emptyMetrics);
      });
    });

    it('handles rapid days changes', async () => {
      mockInvoke.mockResolvedValue(mockMetrics);

      const { rerender } = renderHook(
        ({ days }) => usePRMetrics({ days }),
        { initialProps: { days: 7 } }
      );

      // Rapidly change days
      rerender({ days: 30 });
      rerender({ days: 90 });
      rerender({ days: 7 });

      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalled();
      });

      // Should have called for each unique days value
      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 7 });
      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 30 });
      expect(mockInvoke).toHaveBeenCalledWith('get_pr_based_metrics', { days: 90 });
    });
  });
});
