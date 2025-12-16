/**
 * E2E tests for authentication flow
 * 
 * Tests:
 * - Login flow with GitHub Device Flow
 * - Logout
 * - Session persistence
 */

import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    // Start at login page
    await page.goto('/login');
  });

  test('shows login page for unauthenticated users', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'MADE Tracker' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign in with GitHub' })).toBeVisible();
  });

  test.skip('initiates GitHub Device Flow on login click', async ({ page }) => {
    // TODO: Mock the device flow
    await page.getByRole('button', { name: 'Sign in with GitHub' }).click();
    // Should show loading state
    await expect(page.getByText('Authenticating')).toBeVisible();
  });

  test.skip('redirects to dashboard after successful login', async ({ page }) => {
    // TODO: Mock successful auth
  });

  test.skip('persists session across page reloads', async ({ page }) => {
    // TODO: Test with mocked auth state
  });

  test.skip('clears session on logout', async ({ page }) => {
    // TODO: Test logout flow
  });
});
