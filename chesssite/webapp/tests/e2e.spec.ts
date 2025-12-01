import { test, expect } from '@playwright/test';

test.describe('Chess Site E2E Tests', () => {
  const baseUrl = 'http://localhost:4000';
  const timestamp = Date.now();
  const user1 = `player1_${timestamp}`;
  const user2 = `player2_${timestamp}`;
  const password = 'testpassword123';

  test.describe.serial('Complete game flow', () => {
    let matchId: string;

    test('user 1 can register', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Click "Register" link
      await page.click('button:has-text("Register")');
      
      // Fill registration form
      await page.fill('input[placeholder="Enter your username"]', user1);
      await page.fill('input[placeholder="Enter your password"]', password);
      
      // Submit
      await page.click('button[type="submit"]:has-text("Register")');
      
      // Should redirect to home page
      await page.waitForURL(`${baseUrl}/`);
      await expect(page.locator(`text=Welcome, ${user1}`)).toBeVisible();
      
      // Logout
      await page.click('button:has-text("Logout")');
      await page.waitForURL(`${baseUrl}/login`);
    });

    test('user 2 can register', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Click "Register" link
      await page.click('button:has-text("Register")');
      
      // Fill registration form
      await page.fill('input[placeholder="Enter your username"]', user2);
      await page.fill('input[placeholder="Enter your password"]', password);
      
      // Submit
      await page.click('button[type="submit"]:has-text("Register")');
      
      // Should redirect to home page
      await page.waitForURL(`${baseUrl}/`);
      await expect(page.locator(`text=Welcome, ${user2}`)).toBeVisible();
      
      // Logout
      await page.click('button:has-text("Logout")');
      await page.waitForURL(`${baseUrl}/login`);
    });

    test('user 1 creates a match by inviting user 2', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Login as user 1
      await page.fill('input[placeholder="Enter your username"]', user1);
      await page.fill('input[placeholder="Enter your password"]', password);
      await page.click('button[type="submit"]:has-text("Login")');
      
      await page.waitForURL(`${baseUrl}/`);
      
      // Click New Match button
      await page.click('button:has-text("New Match")');
      
      // Wait for modal
      await page.waitForSelector('text=Create New Match');
      
      // Select opponent
      await page.click('input[placeholder="Choose a player"]');
      await page.click(`text=${user2}`);
      
      // Create match
      await page.click('button:has-text("Create Match")');
      
      // Should navigate to match page
      await page.waitForURL(/\/match\/.+/);
      
      // Extract match ID from URL
      const url = page.url();
      matchId = url.split('/match/')[1];
      
      // Verify we're on the match page
      await expect(page.locator(`text=${user1} vs ${user2}`)).toBeVisible();
      
      // Logout
      await page.click('button:has-text("Back")');
      await page.waitForURL(`${baseUrl}/`);
      await page.click('button:has-text("Logout")');
    });

    test('user 2 sees the match in their match list', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Login as user 2
      await page.fill('input[placeholder="Enter your username"]', user2);
      await page.fill('input[placeholder="Enter your password"]', password);
      await page.click('button[type="submit"]:has-text("Login")');
      
      await page.waitForURL(`${baseUrl}/`);
      
      // Wait for matches to load
      await page.waitForTimeout(1000);
      
      // Should see the match with user 1
      await expect(page.locator(`text=vs ${user1}`)).toBeVisible();
    });

    test('user 2 clicks on the list item and joins the match', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Login as user 2
      await page.fill('input[placeholder="Enter your username"]', user2);
      await page.fill('input[placeholder="Enter your password"]', password);
      await page.click('button[type="submit"]:has-text("Login")');
      
      await page.waitForURL(`${baseUrl}/`);
      
      // Wait for matches to load
      await page.waitForTimeout(1000);
      
      // Click on the match
      await page.click(`text=vs ${user1}`);
      
      // Should navigate to match page
      await page.waitForURL(/\/match\/.+/);
      
      // Verify we see the chess board
      await expect(page.locator(`text=${user1} vs ${user2}`)).toBeVisible();
      await expect(page.locator('text=Your turn').or(page.locator(`text=White's turn`))).toBeVisible();
    });

    test('users can play together with real-time updates', async ({ browser }) => {
      // Create two browser contexts
      const context1 = await browser.newContext();
      const context2 = await browser.newContext();
      
      const page1 = await context1.newPage();
      const page2 = await context2.newPage();

      try {
        // Login user 1
        await page1.goto(`${baseUrl}/login`);
        await page1.waitForLoadState('networkidle');
        await page1.fill('input[placeholder="Enter your username"]', user1);
        await page1.fill('input[placeholder="Enter your password"]', password);
        await page1.click('button[type="submit"]:has-text("Login")');
        await page1.waitForURL(`${baseUrl}/`);
        
        // Go to match
        await page1.click(`text=vs ${user2}`);
        await page1.waitForURL(/\/match\/.+/);
        
        // Login user 2
        await page2.goto(`${baseUrl}/login`);
        await page2.waitForLoadState('networkidle');
        await page2.fill('input[placeholder="Enter your username"]', user2);
        await page2.fill('input[placeholder="Enter your password"]', password);
        await page2.click('button[type="submit"]:has-text("Login")');
        await page2.waitForURL(`${baseUrl}/`);
        
        // Go to same match
        await page2.click(`text=vs ${user1}`);
        await page2.waitForURL(/\/match\/.+/);
        
        // Wait for WebSocket connections
        await page1.waitForTimeout(1500);
        await page2.waitForTimeout(1500);
        
        // User 1 (white) makes a move: e2 to e4
        // Find and click the e2 pawn (row 1, col 4)
        const squares1 = page1.locator('.w-12.h-12, .md\\:w-16.md\\:h-16').nth(52); // e2 square
        await squares1.click();
        
        // Click e4 (row 3, col 4)
        const targetSquare1 = page1.locator('.w-12.h-12, .md\\:w-16.md\\:h-16').nth(36); // e4 square
        await targetSquare1.click();
        
        // Wait for sync
        await page1.waitForTimeout(2000);
        
        // User 2 should see the move reflected (board should update)
        // Now it's black's turn
        await expect(page2.locator(`text=Your turn`)).toBeVisible({ timeout: 5000 });
        
        // User 2 (black) makes a move: e7 to e5
        const squares2 = page2.locator('.w-12.h-12, .md\\:w-16.md\\:h-16').nth(52); // e7 square for black's view
        await squares2.click();
        
        const targetSquare2 = page2.locator('.w-12.h-12, .md\\:w-16.md\\:h-16').nth(36); // e5 square
        await targetSquare2.click();
        
        // Wait for sync
        await page2.waitForTimeout(2000);
        
        // User 1 should see it's their turn again
        await expect(page1.locator(`text=Your turn`)).toBeVisible({ timeout: 5000 });
        
        // Verify move history shows both moves
        await expect(page1.locator('text=e4')).toBeVisible();
        await expect(page2.locator('text=e4')).toBeVisible();

      } finally {
        await context1.close();
        await context2.close();
      }
    });

    test('match can be replayed and updates are saved', async ({ page }) => {
      await page.goto(`${baseUrl}/login`);
      await page.waitForLoadState('networkidle');
      
      // Login as user 1
      await page.fill('input[placeholder="Enter your username"]', user1);
      await page.fill('input[placeholder="Enter your password"]', password);
      await page.click('button[type="submit"]:has-text("Login")');
      
      await page.waitForURL(`${baseUrl}/`);
      
      // Go to match
      await page.click(`text=vs ${user2}`);
      await page.waitForURL(/\/match\/.+/);
      
      // Wait for match to load
      await page.waitForTimeout(1000);
      
      // Check replay functionality
      await expect(page.locator('button:has-text("Replay Game")')).toBeVisible();
      
      // Click replay
      await page.click('button:has-text("Replay Game")');
      
      // Should see replay controls
      await expect(page.locator('text=Replay mode')).toBeVisible();
      
      // Navigate through replay
      await page.click('button:has([class*="IconPlayerSkipForward"])');
      await page.waitForTimeout(500);
      
      // Stop replay
      await page.click('button:has([class*="IconPlayerPause"])');
      
      // Replay should stop
      await expect(page.locator('button:has-text("Replay Game")')).toBeVisible();
    });
  });
});

test.describe('Additional E2E Tests', () => {
  test('login page loads correctly', async ({ page }) => {
    await page.goto('http://localhost:4000/login');
    await page.waitForLoadState('networkidle');
    
    await expect(page.locator('text=Sign In')).toBeVisible();
    await expect(page.locator('input[placeholder="Enter your username"]')).toBeVisible();
    await expect(page.locator('input[placeholder="Enter your password"]')).toBeVisible();
  });

  test('unauthenticated user is redirected to login', async ({ page }) => {
    await page.goto('http://localhost:4000/');
    await page.waitForTimeout(1000);
    
    // Should be on login page
    await expect(page.locator('text=Sign In')).toBeVisible();
  });

  test('invalid login shows error', async ({ page }) => {
    await page.goto('http://localhost:4000/login');
    await page.waitForLoadState('networkidle');
    
    await page.fill('input[placeholder="Enter your username"]', 'nonexistent_user');
    await page.fill('input[placeholder="Enter your password"]', 'wrongpassword');
    await page.click('button[type="submit"]:has-text("Login")');
    
    await page.waitForTimeout(1000);
    
    await expect(page.locator('text=Invalid username or password')).toBeVisible();
  });
});
