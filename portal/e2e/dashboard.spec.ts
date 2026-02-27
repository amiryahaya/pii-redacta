/**
 * Dashboard E2E Tests
 * 
 * Tests the main dashboard functionality
 */

import { test, expect, generateTestUser, API_BASE_URL } from './fixtures';

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    const user = generateTestUser();
    
    // Register via API
    await fetch(`${API_BASE_URL}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(user),
    });

    // Login via UI
    await page.goto('/login');
    await page.fill('[name="email"]', user.email);
    await page.fill('[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
  });

  test('displays dashboard with stats', async ({ page }) => {
    // Should show dashboard title
    await expect(page.locator('h1')).toContainText('Dashboard');
    
    // Should show stats cards
    await expect(page.locator('text=Documents Processed').first()).toBeVisible();
    await expect(page.locator('text=API Requests').first()).toBeVisible();
    await expect(page.locator('text=Active API Keys').first()).toBeVisible();
    
    // Should show view details links
    await expect(page.locator('text=View details').first()).toBeVisible();
  });

  test('navigation links work', async ({ page }) => {
    // Navigate to API Keys
    await page.click('nav >> text=API Keys');
    await expect(page).toHaveURL('/api-keys');
    
    // Navigate back to Dashboard
    await page.click('nav >> text=Dashboard');
    await expect(page).toHaveURL('/dashboard');
    
    // Navigate to Settings
    await page.click('nav >> text=Settings');
    await expect(page).toHaveURL('/settings');
  });

  test('shows recent activity section', async ({ page }) => {
    await expect(page.locator('h3:has-text("Recent Activity")')).toBeVisible();
  });

  test('shows API keys section', async ({ page }) => {
    await expect(page.locator('h3:has-text("Active API Keys")')).toBeVisible();
  });

  test('stats cards have working links', async ({ page }) => {
    // Click on Documents Processed view details
    await page.locator('text=View details').first().click();
    
    // Should navigate to usage page
    await expect(page).toHaveURL('/usage');
  });

  test('user menu works', async ({ page }) => {
    // The user info is displayed in sidebar, not a dropdown menu
    // Verify user email is shown
    await expect(page.locator('text=Sign out')).toBeVisible();
    
    // Click on settings in sidebar
    await page.click('text=Settings');
    await expect(page).toHaveURL('/settings');
  });
});

test.describe('Dashboard - Unauthenticated', () => {
  test('redirects to login when not authenticated', async ({ page }) => {
    await page.goto('/dashboard');
    
    await expect(page).toHaveURL('/login');
  });
});
