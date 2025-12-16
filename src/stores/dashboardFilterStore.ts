import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { MetricsFilters, DateRange, createDateRangeFromDays, DATE_RANGE_PRESETS } from '../types/filters';

interface DashboardFilterState {
  // Current filters
  filters: MetricsFilters;

  // Actions
  setDateRange: (range: DateRange | null) => void;
  setDateRangePreset: (days: number) => void;
  setRepositories: (repositoryIds: number[] | null) => void;
  setSquad: (squadId: string | null) => void;
  setUser: (userId: number | null) => void;
  clearFilters: () => void;

  // Computed
  hasActiveFilters: () => boolean;
}

export const useDashboardFilterStore = create<DashboardFilterState>()(
  persist(
    (set, get) => ({
      // Initialize with default 90-day range
      filters: {
        dateRange: createDateRangeFromDays(DATE_RANGE_PRESETS.LAST_90_DAYS),
      },

      setDateRange: (range) => {
        set({
          filters: {
            ...get().filters,
            dateRange: range || undefined,
          },
        });
      },

      setDateRangePreset: (days) => {
        set({
          filters: {
            ...get().filters,
            dateRange: createDateRangeFromDays(days),
          },
        });
      },

      setRepositories: (repositoryIds) => {
        set({
          filters: {
            ...get().filters,
            repositoryIds: repositoryIds || undefined,
          },
        });
      },

      setSquad: (squadId) => {
        // Squad and user filters are mutually exclusive
        set({
          filters: {
            ...get().filters,
            squadId: squadId || undefined,
            userId: squadId ? undefined : get().filters.userId,
          },
        });
      },

      setUser: (userId) => {
        // Squad and user filters are mutually exclusive
        set({
          filters: {
            ...get().filters,
            userId: userId || undefined,
            squadId: userId ? undefined : get().filters.squadId,
          },
        });
      },

      clearFilters: () => {
        set({
          filters: {
            // Keep 90-day default when clearing
            dateRange: createDateRangeFromDays(DATE_RANGE_PRESETS.LAST_90_DAYS),
          },
        });
      },

      hasActiveFilters: () => {
        const { filters } = get();
        return Boolean(
          filters.repositoryIds?.length ||
          filters.squadId ||
          filters.userId
        );
      },
    }),
    {
      name: 'made-dashboard-filters',
    }
  )
);
