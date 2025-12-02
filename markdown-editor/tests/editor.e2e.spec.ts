import { test, expect } from '@playwright/test';

test.describe('Markdown Editor E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForLoadState('networkidle');
  });

  test('should display empty state when no documents exist', async ({ page }) => {
    await expect(page.getByText('No documents yet')).toBeVisible();
    await expect(page.getByText('Create your first document')).toBeVisible();
    await expect(page.getByText('Select or create a document')).toBeVisible();
  });

  test('should create a new document', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Document should appear in sidebar
    await expect(page.getByText('Untitled')).toBeVisible();
    
    // Editor should be visible with placeholder
    await expect(page.getByRole('textbox')).toBeVisible();
  });

  test('should type content in the editor', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    const editor = page.getByRole('textbox');
    await editor.click();
    await editor.fill('Hello, this is a test document!');
    
    await expect(page.getByText('Hello, this is a test document!')).toBeVisible();
  });

  test('should persist document content in localStorage', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    const editor = page.getByRole('textbox');
    await editor.click();
    await editor.fill('Persistent content test');
    
    // Wait for debounce
    await page.waitForTimeout(500);
    
    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');
    
    // Content should still be there
    await expect(page.getByText('Persistent content test')).toBeVisible();
  });

  test('should rename a document', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Hover over the document item to show actions
    const docItem = page.locator('[data-testid^="document-item-"]').first();
    await docItem.hover();
    
    // Click rename button
    await page.locator('[data-testid^="rename-document-"]').first().click();
    
    // Enter new title
    const titleInput = page.getByTestId('edit-title-input');
    await titleInput.fill('My Renamed Document');
    await page.getByTestId('save-title-button').click();
    
    // Verify new title
    await expect(page.getByText('My Renamed Document')).toBeVisible();
  });

  test('should delete a document', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Hover over the document item
    const docItem = page.locator('[data-testid^="document-item-"]').first();
    await docItem.hover();
    
    // Click delete button
    await page.locator('[data-testid^="delete-document-"]').first().click();
    
    // Confirm deletion
    await page.getByTestId('confirm-delete-button').click();
    
    // Document should be deleted
    await expect(page.getByText('No documents yet')).toBeVisible();
  });

  test('should apply bold formatting', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    const editor = page.getByRole('textbox');
    await editor.click();
    await editor.fill('Test text');
    
    // Select all text
    await page.keyboard.press('Control+a');
    
    // Apply bold
    await page.getByTestId('bold-button').click();
    
    // Check for bold text in editor HTML
    const editorContent = await editor.innerHTML();
    expect(editorContent).toContain('<strong>');
  });

  test('should export markdown content', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    const editor = page.getByRole('textbox');
    await editor.click();
    await editor.fill('Export test content');
    
    // Set up download handler
    const downloadPromise = page.waitForEvent('download');
    
    // Click export button
    await page.getByTestId('export-button').click();
    
    const download = await downloadPromise;
    
    // Verify file name ends with .md
    expect(download.suggestedFilename()).toMatch(/\.md$/);
  });
});

test.describe('Mobile Responsiveness', () => {
  test.use({ viewport: { width: 375, height: 667 } });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
    await page.waitForLoadState('networkidle');
  });

  test('should show document title on mobile', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Mobile title should be visible
    await expect(page.getByTestId('document-title-mobile')).toBeVisible();
    await expect(page.getByTestId('document-title-mobile')).toHaveText('Untitled');
  });

  test('should show export button on mobile', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Export button should be visible on mobile
    await expect(page.getByTestId('export-button')).toBeVisible();
  });

  test('should allow typing on mobile', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    const editor = page.getByRole('textbox');
    await editor.click();
    await editor.fill('Mobile typing test');
    
    await expect(page.getByText('Mobile typing test')).toBeVisible();
  });

  test('should show basic formatting buttons on mobile', async ({ page }) => {
    await page.getByTestId('new-document-button').click();
    
    // Bold, italic, and strikethrough should be visible
    await expect(page.getByTestId('bold-button')).toBeVisible();
    await expect(page.getByTestId('italic-button')).toBeVisible();
    await expect(page.getByTestId('strike-button')).toBeVisible();
  });
});
