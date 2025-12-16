/**
 * E2E tests for dashboard
 * 
 * Tests:
 * - Metrics display
 * - Navigation
 * - Sync functionality
 */

import { test, expect } from '@playwright/test';

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // TODO: Setup authenticated state
    await page.goto('/');
  });

  test.skip('displays all metric sections', async ({ page }) => {
    await expect(page.getByText('Speed')).toBeVisible();
    await expect(page.getByText('Ease')).toBeVisible();
    await expect(page.getByText('Quality')).toBeVisible();
  });

  test.skip('shows metric values', async ({ page }) => {
    await expect(page.getByText(/Avg Cycle Time/)).toBeVisible();
    await expect(page.getByText(/PR Lead Time/)).toBeVisible();
    await expect(page.getByText(/Throughput/)).toBeVisible();
  });

  test.skip('triggers sync on button click', async ({ page }) => {
    await page.getByRole('button', { name: 'Sync Now' }).click();
    await expect(page.getByText('Syncing')).toBeVisible();
  });

  test.skip('updates last sync time after sync', async ({ page }) => {
    // TODO: Mock sync completion
  });

  test.skip('navigates to other pages', async ({ page }) => {
    await page.getByRole('link', { name: 'Roadmap' }).click();
    await expect(page).toHaveURL('/roadmap');
    
    await page.getByRole('link', { name: 'Search' }).click();
    await expect(page).toHaveURL('/search');
    
    await page.getByRole('link', { name: 'Settings' }).click();
    await expect(page).toHaveURL('/settings');
  });
});
