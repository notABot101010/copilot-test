import { test, expect } from '@playwright/test';

test.describe('Chart Title Update E2E Tests', () => {
  const baseUrl = 'http://localhost:4000';

  test.beforeEach(async ({ page }) => {
    // Wait for the page to load
    await page.goto(baseUrl);
    await page.waitForLoadState('networkidle');
  });

  test('should create a chart and update its title', async ({ page }) => {
    // First create a spreadsheet
    await page.click('button:has-text("New Spreadsheet")');
    await page.waitForSelector('input[placeholder*="spreadsheet"]');
    await page.fill('input[placeholder*="spreadsheet"]', `Chart Title Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/spreadsheets\/.+/);
    
    // Wait for spreadsheet to load
    await page.waitForTimeout(500);
    
    // Click on Insert menu
    await page.click('button:has-text("Insert")');
    
    // Click on New Chart
    await page.click('text=New Chart');
    
    // Wait for the chart creation modal
    await page.waitForSelector('text=Create New Chart');
    
    // Enter chart title
    const originalTitle = 'My Test Chart';
    await page.getByRole('textbox', { name: 'Chart Title' }).fill(originalTitle);
    
    // Click Create Chart button
    await page.click('button:has-text("Create Chart")');
    
    // Wait for chart to appear
    await page.waitForSelector('[data-testid="chart-title"]');
    
    // Verify original title is shown
    await expect(page.locator('[data-testid="chart-title"]')).toContainText(originalTitle);
    
    // Click on edit title button
    await page.click('[data-testid="edit-chart-title-button"]');
    
    // Wait for edit title modal
    await page.waitForSelector('text=Edit Chart Title');
    
    // Clear and enter new title
    const newTitle = 'Updated Chart Title';
    await page.getByTestId('chart-title-input').fill(newTitle);
    
    // Click Save button
    await page.click('[data-testid="save-chart-title-button"]');
    
    // Wait for modal to close
    await page.waitForSelector('text=Edit Chart Title', { state: 'hidden' });
    
    // Verify the chart title was updated
    await expect(page.locator('[data-testid="chart-title"]')).toContainText(newTitle);
  });

  test('should cancel title edit without saving', async ({ page }) => {
    // First create a spreadsheet with a chart
    await page.click('button:has-text("New Spreadsheet")');
    await page.waitForSelector('input[placeholder*="spreadsheet"]');
    await page.fill('input[placeholder*="spreadsheet"]', `Cancel Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/spreadsheets\/.+/);
    
    // Wait for spreadsheet to load
    await page.waitForTimeout(500);
    
    // Create a chart
    await page.click('button:has-text("Insert")');
    await page.click('text=New Chart');
    await page.waitForSelector('text=Create New Chart');
    
    const originalTitle = 'Original Title';
    await page.getByRole('textbox', { name: 'Chart Title' }).fill(originalTitle);
    await page.click('button:has-text("Create Chart")');
    
    // Wait for chart to appear
    await page.waitForSelector('[data-testid="chart-title"]');
    
    // Click on edit title button
    await page.click('[data-testid="edit-chart-title-button"]');
    await page.waitForSelector('text=Edit Chart Title');
    
    // Enter a different title but cancel
    await page.getByTestId('chart-title-input').fill('This Should Not Save');
    await page.click('button:has-text("Cancel")');
    
    // Verify the original title is still shown
    await expect(page.locator('[data-testid="chart-title"]')).toContainText(originalTitle);
  });

  test('should not allow empty title', async ({ page }) => {
    // First create a spreadsheet with a chart
    await page.click('button:has-text("New Spreadsheet")');
    await page.waitForSelector('input[placeholder*="spreadsheet"]');
    await page.fill('input[placeholder*="spreadsheet"]', `Empty Title Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/spreadsheets\/.+/);
    
    // Wait for spreadsheet to load
    await page.waitForTimeout(500);
    
    // Create a chart
    await page.click('button:has-text("Insert")');
    await page.click('text=New Chart');
    await page.waitForSelector('text=Create New Chart');
    
    const originalTitle = 'My Chart';
    await page.getByRole('textbox', { name: 'Chart Title' }).fill(originalTitle);
    await page.click('button:has-text("Create Chart")');
    
    // Wait for chart to appear
    await page.waitForSelector('[data-testid="chart-title"]');
    
    // Click on edit title button
    await page.click('[data-testid="edit-chart-title-button"]');
    await page.waitForSelector('text=Edit Chart Title');
    
    // Clear the title and try to save
    await page.getByTestId('chart-title-input').fill('');
    await page.click('[data-testid="save-chart-title-button"]');
    
    // Error message should be displayed
    await expect(page.locator('text=Please enter a chart title')).toBeVisible();
    
    // Modal should still be open
    await expect(page.locator('text=Edit Chart Title')).toBeVisible();
  });

  test('should update title via Enter key', async ({ page }) => {
    // First create a spreadsheet with a chart
    await page.click('button:has-text("New Spreadsheet")');
    await page.waitForSelector('input[placeholder*="spreadsheet"]');
    await page.fill('input[placeholder*="spreadsheet"]', `Enter Key Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/spreadsheets\/.+/);
    
    // Wait for spreadsheet to load
    await page.waitForTimeout(500);
    
    // Create a chart
    await page.click('button:has-text("Insert")');
    await page.click('text=New Chart');
    await page.waitForSelector('text=Create New Chart');
    
    const originalTitle = 'Enter Key Chart';
    await page.getByRole('textbox', { name: 'Chart Title' }).fill(originalTitle);
    await page.click('button:has-text("Create Chart")');
    
    // Wait for chart to appear
    await page.waitForSelector('[data-testid="chart-title"]');
    
    // Click on edit title button
    await page.click('[data-testid="edit-chart-title-button"]');
    await page.waitForSelector('text=Edit Chart Title');
    
    // Enter new title and press Enter
    const newTitle = 'Title Updated With Enter';
    await page.getByTestId('chart-title-input').fill(newTitle);
    await page.keyboard.press('Enter');
    
    // Modal should close and title should be updated
    await page.waitForSelector('text=Edit Chart Title', { state: 'hidden' });
    await expect(page.locator('[data-testid="chart-title"]')).toContainText(newTitle);
  });

  test('should close modal via Escape key', async ({ page }) => {
    // First create a spreadsheet with a chart
    await page.click('button:has-text("New Spreadsheet")');
    await page.waitForSelector('input[placeholder*="spreadsheet"]');
    await page.fill('input[placeholder*="spreadsheet"]', `Escape Key Test ${Date.now()}`);
    await page.click('button:has-text("Create")');
    await page.waitForURL(/\/spreadsheets\/.+/);
    
    // Wait for spreadsheet to load
    await page.waitForTimeout(500);
    
    // Create a chart
    await page.click('button:has-text("Insert")');
    await page.click('text=New Chart');
    await page.waitForSelector('text=Create New Chart');
    
    const originalTitle = 'Escape Test Chart';
    await page.getByRole('textbox', { name: 'Chart Title' }).fill(originalTitle);
    await page.click('button:has-text("Create Chart")');
    
    // Wait for chart to appear
    await page.waitForSelector('[data-testid="chart-title"]');
    
    // Click on edit title button
    await page.click('[data-testid="edit-chart-title-button"]');
    await page.waitForSelector('text=Edit Chart Title');
    
    // Enter a different title but press Escape
    await page.getByTestId('chart-title-input').fill('This Should Not Be Saved');
    await page.keyboard.press('Escape');
    
    // Modal should close and original title should remain
    await page.waitForSelector('text=Edit Chart Title', { state: 'hidden' });
    await expect(page.locator('[data-testid="chart-title"]')).toContainText(originalTitle);
  });
});
