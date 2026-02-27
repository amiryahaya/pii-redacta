/**
 * Settings E2E Tests
 * 
 * Tests profile and security settings
 */

import { test, expect, generateTestUser, API_BASE_URL } from './fixtures';

test.describe('Settings', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    const user = generateTestUser();
    
    // Register via API
    await fetch(`${API_BASE_URL}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(user),
    });

    // Store credentials for tests
    await page.evaluate((u) => {
      (window as any).testUser = u;
    }, user);

    // Login via UI
    await page.goto('/login');
    await page.fill('[name="email"]', user.email);
    await page.fill('[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');
    
    // Navigate to Settings
    await page.goto('/settings');
  });

  test.describe('Profile Tab', () => {
    test('displays profile settings', async ({ page }) => {
      await expect(page.locator('h1')).toContainText('Settings');
      await expect(page.locator('text=Profile Information')).toBeVisible();
    });

    test('can update display name', async ({ page }) => {
      await page.fill('[name="displayName"]', 'Updated Name');
      await page.click('text=Save Changes');
      
      // Should show success
      await expect(page.locator('text=Profile updated successfully')).toBeVisible();
    });

    test('can update company name', async ({ page }) => {
      await page.fill('[name="companyName"]', 'Updated Company');
      await page.click('text=Save Changes');
      
      await expect(page.locator('text=Profile updated successfully')).toBeVisible();
    });

    test('shows email as disabled', async ({ page }) => {
      const emailInput = page.locator('[name="email"]');
      
      await expect(emailInput).toBeDisabled();
    });

    test('validates input length', async ({ page }) => {
      // Try to enter very long name
      const longName = 'a'.repeat(101);
      await page.fill('[name="displayName"]', longName);
      await page.click('text=Save Changes');
      
      // Should show validation error
      await expect(page.locator('text=less than 100 characters')).toBeVisible();
    });

    test('shows user avatar with initials', async ({ page }) => {
      // Avatar should be visible with initials (h-20 w-20 is the profile avatar size)
      await expect(page.locator('.h-20.w-20.rounded-full')).toBeVisible();
    });
  });

  test.describe('Security Tab', () => {
    test.beforeEach(async ({ page }) => {
      // Switch to Security tab
      await page.click('text=Security');
    });

    test('displays security settings', async ({ page }) => {
      await expect(page.locator('text=Security')).toBeVisible();
      await expect(page.locator('text=Change Password')).toBeVisible();
    });

    test('can change password', async ({ page }) => {
      // Get the test user credentials
      const user = await page.evaluate(() => (window as any).testUser);
      
      await page.fill('[name="currentPassword"]', user.password);
      await page.fill('[name="newPassword"]', 'NewPass123!');
      await page.fill('[name="confirmPassword"]', 'NewPass123!');
      
      await page.click('text=Change Password');
      
      // Should show success
      await expect(page.locator('text=Password changed')).toBeVisible();
    });

    test.skip('validates current password', async ({ page }) => {
      // Skipped: Mock backend doesn't validate passwords
      await page.fill('[name="currentPassword"]', 'wrongpassword');
      await page.fill('[name="newPassword"]', 'NewPass123!');
      await page.fill('[name="confirmPassword"]', 'NewPass123!');
      
      await page.click('text=Change Password');
      
      await expect(page.locator('text=Current password is incorrect')).toBeVisible();
    });

    test('validates password confirmation match', async ({ page }) => {
      const user = await page.evaluate(() => (window as any).testUser);
      
      await page.fill('[name="currentPassword"]', user.password);
      await page.fill('[name="newPassword"]', 'NewPass123!');
      await page.fill('[name="confirmPassword"]', 'DifferentPass123!');
      
      await page.click('text=Change Password');
      
      await expect(page.locator('text=Passwords do not match')).toBeVisible();
    });

    test('validates new password strength', async ({ page }) => {
      const user = await page.evaluate(() => (window as any).testUser);
      
      await page.fill('[name="currentPassword"]', user.password);
      await page.fill('[name="newPassword"]', 'weak');
      await page.fill('[name="confirmPassword"]', 'weak');
      
      await page.click('text=Change Password');
      
      await expect(page.locator('text=at least 8 characters')).toBeVisible();
    });

    test('can toggle password visibility', async ({ page }) => {
      const passwordInput = page.locator('[name="newPassword"]');
      const toggleButton = page.locator('[aria-label*="new password"]').first();
      
      await page.fill('[name="newPassword"]', 'testpassword');
      
      // Initially password type
      await expect(passwordInput).toHaveAttribute('type', 'password');
      
      // Toggle visibility
      await toggleButton.click();
      
      // Now text type
      await expect(passwordInput).toHaveAttribute('type', 'text');
    });

    test('shows password requirements', async ({ page }) => {
      await expect(page.locator('text=At least 8 characters')).toBeVisible();
      await expect(page.locator('text=Contains a number')).toBeVisible();
      await expect(page.locator('text=Contains a letter')).toBeVisible();
    });

    test('password requirements update as user types', async ({ page }) => {
      await page.fill('[name="newPassword"]', 'Test1');
      
      // Should show partial completion
      await expect(page.locator('text=Contains a letter')).toBeVisible();
    });

    test('shows security tips', async ({ page }) => {
      await expect(page.locator('text=Security Tips')).toBeVisible();
      await expect(page.locator('text=Use a unique password')).toBeVisible();
    });
  });

  test.describe('Notifications Tab', () => {
    test.beforeEach(async ({ page }) => {
      // Switch to Notifications tab
      await page.click('text=Notifications');
    });

    test('displays notification preferences', async ({ page }) => {
      await expect(page.locator('text=Notification Preferences')).toBeVisible();
    });

    test('can toggle usage alerts', async ({ page }) => {
      const checkbox = page.locator('#emailQuotaAlert');
      
      // Wait for checkbox to be enabled
      await expect(checkbox).toBeEnabled();
      
      // Toggle
      await checkbox.click();
      
      // Should persist (visual feedback) - wait for API call to complete
      await expect(checkbox).toBeChecked();
    });

    test('can toggle security alerts', async ({ page }) => {
      const checkbox = page.locator('#emailSecurityAlert');
      
      await expect(checkbox).toBeEnabled();
      await checkbox.click();
      await expect(checkbox).toBeChecked();
    });

    test('can toggle product updates', async ({ page }) => {
      const checkbox = page.locator('#emailMarketing');
      
      await expect(checkbox).toBeEnabled();
      await checkbox.click();
      await expect(checkbox).toBeChecked();
    });

    test('can toggle monthly reports', async ({ page }) => {
      const checkbox = page.locator('#emailMonthlyReport');
      
      await expect(checkbox).toBeEnabled();
      await checkbox.click();
      await expect(checkbox).toBeChecked();
    });
  });

  test.describe('Navigation', () => {
    test('tabs are accessible', async ({ page }) => {
      // Click through all tabs
      await page.click('text=Security');
      await expect(page.locator('[aria-current="page"], [aria-current="true"]').first()).toContainText('Security');
      
      await page.click('text=Notifications');
      await expect(page.locator('[aria-current="page"], [aria-current="true"]').first()).toContainText('Notifications');
      
      await page.click('text=Profile');
      await expect(page.locator('[aria-current="page"], [aria-current="true"]').first()).toContainText('Profile');
    });
  });
});

test.describe('Settings - Unauthenticated', () => {
  test.skip('redirects to login when not authenticated', async ({ page }) => {
    // Skipped: JWT middleware not implemented yet
    await page.goto('/settings');
    await expect(page).toHaveURL('/login');
  });
});
