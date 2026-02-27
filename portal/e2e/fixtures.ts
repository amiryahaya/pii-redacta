/**
 * Test fixtures and utilities for Playwright E2E tests
 */

import { test as base, expect, Page } from '@playwright/test';

// API base URL
export const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:8080';

/**
 * Generate a unique test user
 */
export function generateTestUser() {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 8);
  return {
    email: `test-${timestamp}-${random}@example.com`,
    password: 'SecurePass123!',
    displayName: 'Test User',
    companyName: 'Test Company',
  };
}

/**
 * Register a user via API
 */
export async function registerUserViaAPI(user = generateTestUser()) {
  const response = await fetch(`${API_BASE_URL}/api/v1/auth/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(user),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(`Failed to register user: ${JSON.stringify(error)}`);
  }

  return user;
}

/**
 * Login a user via API
 */
export async function loginUserViaAPI(email: string, password: string) {
  const response = await fetch(`${API_BASE_URL}/api/v1/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(`Failed to login: ${JSON.stringify(error)}`);
  }

  return await response.json();
}

/**
 * Extended test fixture with authentication utilities
 */
export const test = base.extend<{
  authenticatedPage: Page;
  registerUserViaAPI: () => Promise<{ email: string; password: string; displayName: string; companyName: string }>;
}>({
  // Register user via API fixture
  registerUserViaAPI: async ({}, use) => {
    await use(async () => {
      return await registerUserViaAPI();
    });
  },

  // Authenticated page fixture
  authenticatedPage: async ({ page }, use) => {
    const user = generateTestUser();
    
    // Register user via API
    await registerUserViaAPI(user);
    
    // Login via UI
    await page.goto('/login');
    await page.fill('[name="email"]', user.email);
    await page.fill('[name="password"]', user.password);
    await page.click('button[type="submit"]');
    
    // Wait for navigation to dashboard
    await page.waitForURL('/dashboard');
    
    await use(page);
    
    // Cleanup: Could delete user via API here
  },
});

export { expect };
