/**
 * End-to-end tests for MLS Chat application
 */

import { test, expect, Page } from '@playwright/test';

function generateUniqueUsername(): string {
  return `user_${Date.now()}_${Math.random().toString(36).substring(2, 8)}`;
}

async function registerUser(page: Page, username: string, password: string): Promise<void> {
  await page.goto('/register');
  await page.fill('input#username', username);
  await page.fill('input#password', password);
  await page.fill('input#confirmPassword', password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/groups', { timeout: 15000 });
}

async function loginUser(page: Page, username: string, password: string): Promise<void> {
  await page.goto('/login');
  await page.fill('input#username', username);
  await page.fill('input#password', password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/groups', { timeout: 10000 });
}

async function logout(page: Page): Promise<void> {
  await page.click('button:has-text("Logout")');
  await expect(page).toHaveURL('/login', { timeout: 5000 });
}

test.describe('User Registration and Login', () => {
  test('should allow a new user to register', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';

    await registerUser(page, username, password);
    
    await expect(page.locator('h1:has-text("MLS Chat")')).toBeVisible();
    await expect(page.locator(`text=${username}`)).toBeVisible();
  });

  test('should allow login after registration', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';

    await registerUser(page, username, password);
    await logout(page);
    await loginUser(page, username, password);
    
    await expect(page.locator(`text=${username}`)).toBeVisible();
  });
});

test.describe('Group Creation and Management', () => {
  test('should create a new group', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';
    const groupName = 'Test Group';

    await registerUser(page, username, password);
    
    await page.click('a:has-text("Create Group")');
    await expect(page).toHaveURL('/groups/create');
    
    await page.fill('input#name', groupName);
    await page.click('button[type="submit"]');
    
    await expect(page.locator(`h1:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });
  });

  test('should list created groups', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';
    const groupName = 'My Test Group';

    await registerUser(page, username, password);
    
    await page.click('a:has-text("Create Group")');
    await page.fill('input#name', groupName);
    await page.click('button[type="submit"]');
    await expect(page.locator(`h1:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });
    
    await page.click('a:has-text("←")');
    await expect(page).toHaveURL('/groups');
    
    await expect(page.locator(`text=${groupName}`)).toBeVisible();
  });
});

test.describe('Group Invitations', () => {
  test('user1 creates group and invites user2 and user3', async ({ browser }) => {
    const user1 = generateUniqueUsername();
    const user2 = generateUniqueUsername();
    const user3 = generateUniqueUsername();
    const password = 'TestPassword123!';
    const groupName = 'Test';

    // Create separate browser contexts to simulate different users with isolated storage
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    const context3 = await browser.newContext();
    
    const page1 = await context1.newPage();
    const page2 = await context2.newPage();
    const page3 = await context3.newPage();
    
    // Register all users (this generates their MLS key packages in their respective contexts)
    await registerUser(page2, user2, password);
    await registerUser(page3, user3, password);
    await registerUser(page1, user1, password);
    
    // User1 creates group
    await page1.click('a:has-text("Create Group")');
    await page1.fill('input#name', groupName);
    await page1.click('button[type="submit"]');
    await expect(page1.locator(`h1:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });
    
    // Invite user2
    await page1.click('a:has-text("Invite")');
    await expect(page1).toHaveURL(/\/invite$/);
    
    await page1.click(`button[data-invite-user="${user2}"]`);
    await expect(page1.locator(`text=Invited ${user2}`)).toBeVisible({ timeout: 5000 });
    
    // Invite user3
    await page1.click(`button[data-invite-user="${user3}"]`);
    await expect(page1.locator(`text=Invited ${user3}`)).toBeVisible({ timeout: 5000 });

    // User2 accepts invitation - reload to see pending invitations
    await page2.goto('/groups');
    await page2.waitForTimeout(1000);
    await expect(page2.locator('text=Pending Invitations')).toBeVisible({ timeout: 10000 });
    
    // Click Accept and wait for the page to update
    await page2.click('button:has-text("Accept")');
    await page2.waitForTimeout(3000);
    
    // The group should now appear in the list
    await expect(page2.locator(`a:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });

    // User3 accepts invitation - reload to see pending invitations
    await page3.goto('/groups');
    await page3.waitForTimeout(1000);
    await expect(page3.locator('text=Pending Invitations')).toBeVisible({ timeout: 10000 });
    await page3.click('button:has-text("Accept")');
    await page3.waitForTimeout(3000);
    
    await expect(page3.locator(`a:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });
    
    // Cleanup
    await context1.close();
    await context2.close();
    await context3.close();
  });
});

test.describe('Messaging', () => {
  test('user can send message in group', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';
    const groupName = 'Message Test';
    const testMessage = 'Hello, World!';

    await registerUser(page, username, password);
    
    // Create group
    await page.click('a:has-text("Create Group")');
    await page.fill('input#name', groupName);
    await page.click('button[type="submit"]');
    await expect(page.locator(`h1:has-text("${groupName}")`)).toBeVisible({ timeout: 10000 });
    
    // Send a message
    await page.fill('input[placeholder="Type a message..."]', testMessage);
    await page.click('button:has-text("Send")');
    
    // Message should appear
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Channels', () => {
  test('should create a channel', async ({ page }) => {
    const username = generateUniqueUsername();
    const password = 'TestPassword123!';
    const channelName = 'Test Channel';

    await registerUser(page, username, password);
    
    await page.click('a:has-text("Channels")');
    await expect(page).toHaveURL('/channels');
    
    await page.click('a:has-text("Create Channel")');
    await expect(page).toHaveURL('/channels/create');
    
    await page.fill('input#name', channelName);
    await page.click('button[type="submit"]');
    
    await expect(page.locator(`h1:has-text("${channelName}")`)).toBeVisible({ timeout: 10000 });
  });

  test('users can subscribe to channels', async ({ browser }) => {
    const admin = generateUniqueUsername();
    const subscriber = generateUniqueUsername();
    const password = 'TestPassword123!';
    const channelName = 'Public Channel';

    // Admin creates channel
    const adminPage = await browser.newPage();
    await registerUser(adminPage, admin, password);
    await adminPage.click('a:has-text("Channels")');
    await adminPage.click('a:has-text("Create Channel")');
    await adminPage.fill('input#name', channelName);
    await adminPage.click('button[type="submit"]');
    await expect(adminPage.locator(`h1:has-text("${channelName}")`)).toBeVisible({ timeout: 10000 });
    await adminPage.close();

    // Subscriber subscribes
    const subPage = await browser.newPage();
    await registerUser(subPage, subscriber, password);
    await subPage.click('a:has-text("Channels")');
    await expect(subPage).toHaveURL('/channels');
    
    await expect(subPage.locator(`text=${channelName}`)).toBeVisible({ timeout: 5000 });
    
    await subPage.click(`button[data-subscribe-channel="${channelName}"]`);
    await subPage.waitForTimeout(2000);
    
    await expect(subPage.locator(`a:has-text("${channelName}")`)).toBeVisible({ timeout: 5000 });
    await subPage.close();
  });

  test('only admins can post in channels', async ({ browser }) => {
    const admin = generateUniqueUsername();
    const subscriber = generateUniqueUsername();
    const password = 'TestPassword123!';
    const channelName = 'Admin Only Channel';

    // Admin creates channel and posts
    const adminPage = await browser.newPage();
    await registerUser(adminPage, admin, password);
    await adminPage.click('a:has-text("Channels")');
    await adminPage.click('a:has-text("Create Channel")');
    await adminPage.fill('input#name', channelName);
    await adminPage.click('button[type="submit"]');
    await expect(adminPage.locator(`h1:has-text("${channelName}")`)).toBeVisible({ timeout: 10000 });

    const adminMessage = 'Announcement from admin!';
    await adminPage.fill('input[placeholder="Type a message..."]', adminMessage);
    await adminPage.click('button:has-text("Send")');
    await expect(adminPage.locator(`text=${adminMessage}`)).toBeVisible({ timeout: 5000 });
    
    await adminPage.close();

    // Subscriber subscribes and checks
    const subPage = await browser.newPage();
    await registerUser(subPage, subscriber, password);
    await subPage.click('a:has-text("Channels")');
    
    await subPage.click(`button[data-subscribe-channel="${channelName}"]`);
    await subPage.waitForTimeout(2000);
    
    await subPage.click(`a:has-text("${channelName}")`);
    await expect(subPage.locator(`h1:has-text("${channelName}")`)).toBeVisible({ timeout: 10000 });
    
    // Subscriber should NOT see the message input
    await expect(subPage.locator('input[placeholder="Type a message..."]')).not.toBeVisible();
    
    await subPage.close();
  });
});
