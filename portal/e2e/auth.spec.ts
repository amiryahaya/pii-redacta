/**
 * Authentication E2E Tests
 * 
 * Tests user registration, login, and logout flows
 */

import { test, expect, generateTestUser, API_BASE_URL } from './fixtures';

test.describe('Authentication', () => {
  test.describe('Registration', () => {
    test('user can register with valid credentials', async ({ page }) => {
      const user = generateTestUser();

      await page.goto('/register');
      
      // Fill registration form
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      await page.fill('[name="displayName"]', user.displayName);
      await page.fill('[name="companyName"]', user.companyName);
      await page.check('[name="terms"]');
      
      // Submit form
      await page.click('button[type="submit"]');
      
      // Should redirect to dashboard
      await expect(page).toHaveURL('/dashboard');
      
      // Should show welcome toast
      await expect(page.locator('[role="alert"]')).toContainText('Account created');
    });

    test('shows error for weak password', async ({ page }) => {
      await page.goto('/register');
      
      await page.fill('[name="email"]', generateTestUser().email);
      await page.fill('[name="password"]', 'weak');
      await page.check('[name="terms"]');
      
      await page.click('button[type="submit"]');
      
      // Should stay on register page with error indicator
      await expect(page).toHaveURL('/register');
      await expect(page.locator('input[name="password"][aria-invalid="true"]')).toBeVisible();
    });

    test('shows error for invalid email', async ({ page }) => {
      await page.goto('/register');
      
      await page.fill('[name="email"]', 'not-an-email');
      await page.fill('[name="password"]', 'SecurePass123!');
      await page.check('[name="terms"]');
      
      await page.click('button[type="submit"]');
      
      // Should show validation error on email field
      await expect(page.locator('input[name="email"][aria-invalid="true"]')).toBeVisible();
    });

    test('shows error for missing terms acceptance', async ({ page }) => {
      await page.goto('/register');
      
      await page.fill('[name="email"]', generateTestUser().email);
      await page.fill('[name="password"]', 'SecurePass123!');
      // Don't check terms
      
      await page.click('button[type="submit"]');
      
      // Should stay on register page
      await expect(page).toHaveURL('/register');
    });

    test('shows error for duplicate email', async ({ page }) => {
      const user = generateTestUser();
      
      // Register first time
      await page.goto('/register');
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      await page.check('[name="terms"]');
      await page.click('button[type="submit"]');
      await page.waitForURL('/dashboard');
      
      // Logout
      await page.click('text=Sign out');
      await page.waitForURL('/login');
      
      // Try to register again with same email
      await page.goto('/register');
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      await page.check('[name="terms"]');
      await page.click('button[type="submit"]');
      
      // Should show error
      await expect(page.locator('[role="alert"]')).toContainText('already exists');  // Backend returns "An account with this email already exists"
    });

    test('has link to login page', async ({ page }) => {
      await page.goto('/register');
      
      await expect(page.locator('a[href="/login"]')).toBeVisible();
      await expect(page.locator('text=Already have an account')).toBeVisible();
    });
  });

  test.describe('Login', () => {
    test('user can login with valid credentials', async ({ page, registerUserViaAPI }) => {
      const user = await registerUserViaAPI();

      await page.goto('/login');
      
      // Fill login form
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      
      // Submit form
      await page.click('button[type="submit"]');
      
      // Should redirect to dashboard
      await expect(page).toHaveURL('/dashboard');
      
      // Should show success message
      await expect(page.locator('[role="alert"]')).toContainText('Signed in successfully');
    });

    test('shows error for invalid credentials', async ({ page }) => {
      await page.goto('/login');
      
      await page.fill('[name="email"]', 'nonexistent@example.com');
      await page.fill('[name="password"]', 'WrongPassword123!');
      await page.click('button[type="submit"]');
      
      // Should stay on login page (API returns 401, error shown to user)
      await expect(page).toHaveURL('/login');
    });

    test('has link to register page', async ({ page }) => {
      await page.goto('/login');
      
      await expect(page.locator('a[href="/register"]')).toBeVisible();
      await expect(page.locator('text=start your 14-day free trial')).toBeVisible();
    });

    test('has link to forgot password page', async ({ page }) => {
      await page.goto('/login');
      
      await expect(page.locator('a[href="/forgot-password"]')).toBeVisible();
      await expect(page.locator('text=Forgot your password?')).toBeVisible();
    });
  });

  test.describe('Logout', () => {
    test('user can logout', async ({ page, registerUserViaAPI }) => {
      const user = await registerUserViaAPI();

      // Login first
      await page.goto('/login');
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      await page.click('button[type="submit"]');
      await page.waitForURL('/dashboard');
      
      // Logout
      await page.click('text=Sign out');
      
      // Should redirect to login
      await expect(page).toHaveURL('/login');
    });

    test('authenticated user redirected from login to dashboard', async ({ page, registerUserViaAPI }) => {
      const user = await registerUserViaAPI();

      // Login
      await page.goto('/login');
      await page.fill('[name="email"]', user.email);
      await page.fill('[name="password"]', user.password);
      await page.click('button[type="submit"]');
      await page.waitForURL('/dashboard');
      
      // Try to go to login page again
      await page.goto('/login');
      
      // Should be redirected to dashboard
      await expect(page).toHaveURL('/dashboard');
    });
  });

  test.describe('Protected Routes', () => {
    // TODO: Enable these tests after JWT middleware is implemented
    test.skip('unauthenticated user redirected to login', async ({ page }) => {
      await page.goto('/dashboard');
      
      // Should be redirected to login
      await expect(page).toHaveURL('/login');
      
      // Should show authentication required message
      await expect(page.locator('[role="alert"]')).toContainText('sign in');
    });

    test.skip('unauthenticated user cannot access settings', async ({ page }) => {
      await page.goto('/settings');
      
      await expect(page).toHaveURL('/login');
    });

    test.skip('unauthenticated user cannot access api keys', async ({ page }) => {
      await page.goto('/api-keys');
      
      await expect(page).toHaveURL('/login');
    });
  });
});
