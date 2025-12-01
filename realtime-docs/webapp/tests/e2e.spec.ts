import { test, expect } from '@playwright/test';

test.describe('Realtime Docs E2E Tests', () => {
  const baseUrl = 'http://localhost:4000';
  const apiUrl = 'http://localhost:4001';

  test.beforeEach(async ({ page }) => {
    // Wait for the page to load
    await page.goto(baseUrl);
    await page.waitForLoadState('networkidle');
  });

  test('should display the home page with document list', async ({ page }) => {
    await expect(page.locator('h1')).toContainText('Realtime Docs');
    await expect(page.locator('button:has-text("New Document")')).toBeVisible();
  });

  test('should create a new document', async ({ page }) => {
    // Click on New Document button
    await page.click('button:has-text("New Document")');
    
    // Wait for modal to open
    await page.waitForSelector('input[placeholder="Enter a title for your document"]');
    
    // Enter document title
    const docTitle = `Test Doc ${Date.now()}`;
    await page.fill('input[placeholder="Enter a title for your document"]', docTitle);
    
    // Click Create button
    await page.click('button:has-text("Create")');
    
    // Should navigate to the document editor
    await page.waitForURL(/\/documents\/.+/);
    
    // Should display the document title
    await expect(page.locator('h1')).toContainText(docTitle);
  });

  test('should edit document content and see markdown preview', async ({ page }) => {
    // Create a new document first
    await page.click('button:has-text("New Document")');
    await page.waitForSelector('input[placeholder="Enter a title for your document"]');
    await page.fill('input[placeholder="Enter a title for your document"]', `Markdown Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/documents\/.+/);
    
    // Wait for editor to load
    await page.waitForSelector('textarea');
    
    // Type some markdown content
    await page.fill('textarea', '# Hello World\n\nThis is **bold** text.');
    
    // Wait for preview to update
    await page.waitForTimeout(500);
    
    // Check that the preview shows the rendered markdown
    await expect(page.locator('h1:has-text("Hello World")')).toBeVisible();
    await expect(page.locator('strong:has-text("bold")')).toBeVisible();
  });

  test('should delete a document', async ({ page }) => {
    // Create a new document first
    const docTitle = `Delete Test ${Date.now()}`;
    await page.click('button:has-text("New Document")');
    await page.waitForSelector('input[placeholder="Enter a title for your document"]');
    await page.fill('input[placeholder="Enter a title for your document"]', docTitle);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/documents\/.+/);
    
    // Go back to home
    await page.click('button:has-text("←")');
    await page.waitForURL(baseUrl + '/');
    
    // Wait for document list to load
    await page.waitForTimeout(2000);
    
    // Find the document card and click the delete button
    const docText = page.locator(`p:has-text("${docTitle}")`);
    await expect(docText).toBeVisible();
    
    // Get the parent card and find the delete button within it
    const card = docText.locator('xpath=ancestor::div[contains(@class, "mantine-Card-root")]');
    await card.locator('button').click();
    
    // Confirm deletion in modal
    await page.waitForSelector('text=Delete Document');
    await page.click('button.mantine-Button-root:has-text("Delete")');
    
    // Wait for deletion
    await page.waitForTimeout(1000);
    
    // Document should no longer be visible
    await expect(page.locator(`p:has-text("${docTitle}")`)).not.toBeVisible();
  });
});

test.describe('Real-time Sync Tests', () => {
  const baseUrl = 'http://localhost:4000';
  const apiUrl = 'http://localhost:4001';

  test('should sync changes between two browser tabs in real-time', async ({ browser }) => {
    // Create two browser contexts (simulating two different tabs/browsers)
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    
    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      // Create a new document in page1
      await page1.goto(baseUrl);
      await page1.waitForLoadState('networkidle');
      
      await page1.click('button:has-text("New Document")');
      await page1.waitForSelector('input[placeholder="Enter a title for your document"]');
      
      const docTitle = `Sync Test ${Date.now()}`;
      await page1.fill('input[placeholder="Enter a title for your document"]', docTitle);
      await page1.click('button:has-text("Create")');
      
      // Wait for navigation to document editor
      await page1.waitForURL(/\/documents\/.+/);
      
      // Get the document URL
      const docUrl = page1.url();
      
      // Open the same document in page2
      await page2.goto(docUrl);
      await page2.waitForLoadState('networkidle');
      
      // Wait for WebSocket connection in page2
      await page2.waitForSelector('textarea');
      await page2.waitForTimeout(1000);
      
      // Type content in page1
      await page1.fill('textarea', '# Hello from Tab 1');
      
      // Wait for sync
      await page1.waitForTimeout(1500);
      
      // Check that page2 received the update
      const page2Content = await page2.locator('textarea').inputValue();
      expect(page2Content).toContain('Hello from Tab 1');
      
      // Now type in page2
      await page2.fill('textarea', '# Hello from Tab 1\n\n## Added from Tab 2');
      
      // Wait for sync
      await page2.waitForTimeout(1500);
      
      // Check that page1 received the update
      const page1Content = await page1.locator('textarea').inputValue();
      expect(page1Content).toContain('Added from Tab 2');
      
    } finally {
      await context1.close();
      await context2.close();
    }
  });

  test('should broadcast updates via WebSocket to multiple clients', async ({ browser }) => {
    // Create three browser contexts
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    const context3 = await browser.newContext();
    
    const page1 = await context1.newPage();
    const page2 = await context2.newPage();
    const page3 = await context3.newPage();

    try {
      // Create a new document in page1
      await page1.goto(baseUrl);
      await page1.waitForLoadState('networkidle');
      
      await page1.click('button:has-text("New Document")');
      await page1.waitForSelector('input[placeholder="Enter a title for your document"]');
      
      const docTitle = `Multi-Client Sync ${Date.now()}`;
      await page1.fill('input[placeholder="Enter a title for your document"]', docTitle);
      await page1.click('button:has-text("Create")');
      
      await page1.waitForURL(/\/documents\/.+/);
      const docUrl = page1.url();
      
      // Open the same document in page2 and page3
      await page2.goto(docUrl);
      await page3.goto(docUrl);
      
      await page2.waitForLoadState('networkidle');
      await page3.waitForLoadState('networkidle');
      
      // Wait for WebSocket connections
      await page2.waitForSelector('textarea');
      await page3.waitForSelector('textarea');
      await page1.waitForTimeout(1500);
      
      // Type content in page1
      await page1.fill('textarea', '# Shared Document\n\nAll clients should see this.');
      
      // Wait for sync to all clients
      await page1.waitForTimeout(2000);
      
      // Check that both page2 and page3 received the update
      const page2Content = await page2.locator('textarea').inputValue();
      const page3Content = await page3.locator('textarea').inputValue();
      
      expect(page2Content).toContain('Shared Document');
      expect(page3Content).toContain('Shared Document');
      expect(page2Content).toContain('All clients should see this');
      expect(page3Content).toContain('All clients should see this');
      
    } finally {
      await context1.close();
      await context2.close();
      await context3.close();
    }
  });
});
