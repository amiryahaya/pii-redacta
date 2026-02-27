# PII Redacta Portal E2E Tests

This directory contains end-to-end tests using Playwright.

## Prerequisites

- Node.js 18+
- Backend API running on http://localhost:8080
- PostgreSQL database with test data
- Chrome/Chromium browser (Playwright will install this)

## Installation

```bash
# Install dependencies (from portal directory)
npm install

# Install Playwright browsers
npx playwright install chromium
```

## Running Tests

### Run all tests
```bash
npm run test:e2e
```

### Run specific test file
```bash
npx playwright test auth.spec.ts
```

### Run with UI mode (for debugging)
```bash
npx playwright test --ui
```

### Run in headed mode (see browser)
```bash
npx playwright test --headed
```

### Run with specific project
```bash
npx playwright test --project=chromium
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PLAYWRIGHT_BASE_URL` | Frontend URL | http://localhost:5173 |
| `API_BASE_URL` | Backend API URL | http://localhost:8080 |
| `SKIP_WEBSERVER` | Skip starting dev server | false |
| `CI` | Run in CI mode | false |

Example:
```bash
PLAYWRIGHT_BASE_URL=http://localhost:3000 API_BASE_URL=http://localhost:8080 npm run test:e2e
```

## Test Structure

```
e2e/
├── fixtures.ts          # Test fixtures and utilities
├── auth.spec.ts         # Authentication tests
├── dashboard.spec.ts    # Dashboard tests
├── api-keys.spec.ts     # API key management tests
├── settings.spec.ts     # Settings/profile tests
└── README.md           # This file
```

## Writing Tests

### Basic Test Structure
```typescript
import { test, expect } from './fixtures';

test('description', async ({ page }) => {
  await page.goto('/some-page');
  await page.click('text=Button');
  await expect(page).toHaveURL('/new-page');
});
```

### Using Authenticated Fixture
```typescript
import { test, expect } from './fixtures';

test('test with logged in user', async ({ authenticatedPage }) => {
  // User is already logged in
  await authenticatedPage.goto('/dashboard');
  await expect(authenticatedPage.locator('h1')).toContainText('Dashboard');
});
```

### Generating Test Users
```typescript
import { generateTestUser } from './fixtures';

const user = generateTestUser();
// user.email, user.password, etc.
```

## Debugging

1. **Use UI Mode**: `npx playwright test --ui`
2. **Slow motion**: Set `slowMo: 1000` in playwright.config.ts
3. **Screenshots/Videos**: Automatically captured on failure
4. **Trace viewer**: `npx playwright show-trace trace.zip`

## Troubleshooting

### Tests fail with timeout
- Ensure backend is running: `cargo run --bin pii-redacta-api`
- Check API_BASE_URL is correct
- Increase timeout in config

### Database errors
- Ensure PostgreSQL is running
- Database should have migrations applied
- Test isolation relies on unique email addresses

### Element not found
- Check selector is correct
- Add `await page.waitForLoadState('networkidle')`
- Use Playwright's codegen: `npx playwright codegen`

## CI Integration

For CI environments, set:
```bash
export CI=true
export PLAYWRIGHT_BASE_URL=http://localhost:3000
```

This will:
- Use 1 worker (sequential tests)
- Retry failed tests 2 times
- Not start dev server if `SKIP_WEBSERVER=true`
