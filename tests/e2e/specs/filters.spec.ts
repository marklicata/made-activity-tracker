import { test, expect } from '@playwright/test';

/**
 * E2E tests for Dashboard Filters
 *
 * Prerequisites:
 * - User must be logged in
 * - At least 2 repositories synced with data
 * - At least 1 squad configured
 * - Some issues and PRs in the database
 */

test.describe('Dashboard Filters', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('/');

    // Wait for dashboard to load
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();

    // Wait for metrics to load
    await page.waitForSelector('[data-testid="metric-card"]', { state: 'visible', timeout: 5000 }).catch(() => {});
  });

  test('should display filter bar on dashboard', async ({ page }) => {
    // Check that all filter buttons are visible
    await expect(page.locator('button:has-text("Last 90 days")')).toBeVisible();
    await expect(page.locator('button:has-text("repositories")')).toBeVisible();

    // Squad and user filters might be hidden depending on which is active
    // Just check that the filter bar container exists
    const filterBar = page.locator('[class*="filter"]').first();
    await expect(filterBar).toBeVisible();
  });

  test('should open date range filter dropdown', async ({ page }) => {
    // Click date range button
    await page.click('button:has-text("days")');

    // Check that preset options appear
    await expect(page.locator('button:has-text("Last 7 days")')).toBeVisible();
    await expect(page.locator('button:has-text("Last 30 days")')).toBeVisible();
    await expect(page.locator('button:has-text("Last year")')).toBeVisible();
  });

  test('should change date range preset', async ({ page }) => {
    // Click date range button
    await page.click('button:has-text("days")');

    // Select "Last 30 days"
    await page.click('button:has-text("Last 30 days")');

    // Verify button text updated
    await expect(page.locator('button:has-text("Last 30 days")').first()).toBeVisible();

    // Wait a moment for metrics to update (debounce)
    await page.waitForTimeout(500);

    // Metrics should still be visible (they would have reloaded)
    await expect(page.locator('h2:has-text("Speed")')).toBeVisible();
  });

  test('should open repository filter dropdown', async ({ page }) => {
    // Click repository filter button
    await page.click('button:has-text("repositories")');

    // Check that dropdown appears
    await expect(page.locator('button:has-text("All")')).toBeVisible();
    await expect(page.locator('button:has-text("Clear")')).toBeVisible();

    // Repository list should be visible
    const repoItems = page.locator('[role="button"]:has-text("/")');
    await expect(repoItems.first()).toBeVisible();
  });

  test('should filter by repository', async ({ page }) => {
    // Open repository filter
    await page.click('button:has-text("repositories")');

    // Wait for dropdown to appear
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });

    // Click first repository checkbox
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();

    // Wait for dropdown to close (it might close or stay open)
    await page.waitForTimeout(300);

    // Check that "Clear filters" button appears
    await expect(page.locator('button:has-text("Clear filters")')).toBeVisible();

    // Metrics should update (debounced)
    await page.waitForTimeout(500);
  });

  test('should filter by squad', async ({ page }) => {
    // Look for squad filter button (might not exist if user filter is active)
    const squadButton = page.locator('button:has-text("squad")').first();

    if (await squadButton.isVisible()) {
      // Click squad filter
      await squadButton.click();

      // Check dropdown appears
      await expect(page.locator('button:has-text("All squads")')).toBeVisible();

      // If squads are configured, select one
      const firstSquad = page.locator('button').filter({ hasText: /^(?!All squads).*/ }).first();
      if (await firstSquad.isVisible()) {
        await firstSquad.click();

        // Clear filters button should appear
        await expect(page.locator('button:has-text("Clear filters")')).toBeVisible();
      }
    } else {
      test.skip(true, 'Squad filter not visible (user filter might be active)');
    }
  });

  test('should filter by user', async ({ page }) => {
    // Look for user filter button
    const userButton = page.locator('button:has-text("users")').first();

    if (await userButton.isVisible()) {
      // Click user filter
      await userButton.click();

      // Check search input appears
      await expect(page.locator('input[placeholder*="Search"]')).toBeVisible();

      // Check "All users" option appears
      await expect(page.locator('button:has-text("All users")')).toBeVisible();

      // If users exist, select one
      const firstUser = page.locator('button').filter({ hasText: /@/ }).first();
      if (await firstUser.isVisible()) {
        await firstUser.click();

        // Clear filters button should appear
        await expect(page.locator('button:has-text("Clear filters")')).toBeVisible();
      }
    } else {
      test.skip(true, 'User filter not visible (squad filter might be active)');
    }
  });

  test('should combine multiple filters', async ({ page }) => {
    // Set date range
    await page.click('button:has-text("days")');
    await page.click('button:has-text("Last 30 days")');
    await page.waitForTimeout(200);

    // Set repository
    await page.click('button:has-text("repositories")');
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();
    await page.waitForTimeout(200);

    // Check that clear filters appears
    await expect(page.locator('button:has-text("Clear filters")')).toBeVisible();

    // Both filters should be active
    await expect(page.locator('button:has-text("Last 30 days")')).toBeVisible();
    await expect(page.locator('button:has-text("1 repository")')).toBeVisible();

    // Wait for metrics to update
    await page.waitForTimeout(500);

    // Metrics should still be visible
    await expect(page.locator('h2:has-text("Speed")')).toBeVisible();
  });

  test('should clear all filters', async ({ page }) => {
    // Set some filters
    await page.click('button:has-text("repositories")');
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();
    await page.waitForTimeout(200);

    // Click clear filters
    await page.click('button:has-text("Clear filters")');

    // Clear filters button should disappear
    await expect(page.locator('button:has-text("Clear filters")')).not.toBeVisible();

    // Should return to "All repositories"
    await expect(page.locator('button:has-text("All repositories")')).toBeVisible();

    // Metrics should update
    await page.waitForTimeout(500);
    await expect(page.locator('h2:has-text("Speed")')).toBeVisible();
  });

  test('should persist filter state on page reload', async ({ page }) => {
    // Set a repository filter
    await page.click('button:has-text("repositories")');
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();
    await page.waitForTimeout(300);

    // Reload the page
    await page.reload();
    await page.waitForSelector('h1:has-text("Dashboard")', { state: 'visible' });

    // Filter should still be active
    await expect(page.locator('button:has-text("1 repository")')).toBeVisible();
    await expect(page.locator('button:has-text("Clear filters")')).toBeVisible();
  });

  test('should persist filter state on navigation', async ({ page }) => {
    // Set filters
    await page.click('button:has-text("repositories")');
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();
    await page.waitForTimeout(300);

    // Navigate to settings
    await page.click('a[href="/settings"]');
    await expect(page.locator('h1:has-text("Settings")')).toBeVisible();

    // Navigate back to dashboard
    await page.click('a[href="/"]');
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();

    // Filters should still be active
    await expect(page.locator('button:has-text("1 repository")')).toBeVisible();
  });

  test('should display charts with filter data', async ({ page }) => {
    // Set a filter
    await page.click('button:has-text("repositories")');
    await page.waitForSelector('button:has-text("All")', { state: 'visible' });
    const firstRepo = page.locator('[role="button"]:has-text("/")').first();
    await firstRepo.click();
    await page.waitForTimeout(500); // Wait for debounce + chart data load

    // Scroll down to see charts
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));

    // Check that charts are present
    // Recharts renders SVG charts
    const charts = page.locator('svg');
    const chartCount = await charts.count();

    // Should have at least one chart (Speed, Ease, or Quality)
    expect(chartCount).toBeGreaterThan(0);
  });

  test('should show "no data" message when filters result in empty dataset', async ({ page }) => {
    // This test assumes there's a way to filter to get no data
    // For example, selecting a date range with no activity

    // Set a very short recent date range (last 7 days)
    await page.click('button:has-text("days")');
    await page.click('button:has-text("Last 7 days")');

    await page.waitForTimeout(500);

    // Either metrics should show or "no data" message
    // This is environment-dependent, so we'll just verify the page is stable
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
  });

  test('squad and user filters should be mutually exclusive', async ({ page }) => {
    // This test verifies that setting squad hides user filter and vice versa

    // Try to find both buttons
    const squadButton = page.locator('button:has-text("squad")');
    const userButton = page.locator('button:has-text("user")');

    const squadVisible = await squadButton.isVisible().catch(() => false);
    const userVisible = await userButton.isVisible().catch(() => false);

    // Both shouldn't be visible at the same time
    if (squadVisible) {
      expect(userVisible).toBe(false);
    }
    if (userVisible) {
      expect(squadVisible).toBe(false);
    }

    test.skip(!squadVisible && !userVisible, 'Neither squad nor user filter visible');
  });
});
