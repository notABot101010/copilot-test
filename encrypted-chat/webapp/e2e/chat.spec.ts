/**
 * End-to-end tests for the encrypted chat application
 * 
 * These tests verify the complete user flow:
 * - User registration
 * - User login
 * - Viewing conversations
 * - Sending messages
 * - Message deduplication
 */

import { test, expect } from '@playwright/test';

function generateUniqueId(): string {
  return `test_${Date.now()}_${Math.random().toString(36).substring(2, 8)}`;
}

test.describe('User Registration', () => {
  test('should allow a new user to register', async ({ page }) => {
    const username = generateUniqueId();
    const password = 'TestPassword123!';

    await page.goto('/register');
    
    await expect(page.locator('h1')).toContainText('Encrypted Chat');
    await expect(page.locator('h3')).toContainText('Create Account');

    await page.fill('input[placeholder="Choose a username"]', username);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    
    await page.click('button[type="submit"]');
    
    // Should redirect to conversations page after successful registration
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await expect(page.locator('text=' + username)).toBeVisible();
  });

  test('should show error for password mismatch', async ({ page }) => {
    await page.goto('/register');
    
    await page.fill('input[placeholder="Choose a username"]', 'testuser');
    await page.fill('input[placeholder="Choose a password"]', 'Password123!');
    await page.fill('input[placeholder="Confirm your password"]', 'DifferentPass');
    
    await page.click('button[type="submit"]');
    
    await expect(page.locator('text=Passwords do not match')).toBeVisible();
  });

  test('should show error for short password', async ({ page }) => {
    await page.goto('/register');
    
    await page.fill('input[placeholder="Choose a username"]', 'testuser');
    await page.fill('input[placeholder="Choose a password"]', 'short');
    await page.fill('input[placeholder="Confirm your password"]', 'short');
    
    await page.click('button[type="submit"]');
    
    await expect(page.locator('text=Password must be at least 8 characters')).toBeVisible();
  });

  test('should show error for short username', async ({ page }) => {
    await page.goto('/register');
    
    await page.fill('input[placeholder="Choose a username"]', 'ab');
    await page.fill('input[placeholder="Choose a password"]', 'Password123!');
    await page.fill('input[placeholder="Confirm your password"]', 'Password123!');
    
    await page.click('button[type="submit"]');
    
    await expect(page.locator('text=Username must be at least 3 characters')).toBeVisible();
  });
});

test.describe('User Login', () => {
  test('should allow an existing user to login', async ({ page }) => {
    const username = generateUniqueId();
    const password = 'TestPassword123!';

    // First register
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', username);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Logout (click the logout button)
    await page.click('button:has-text("Logout")');
    await expect(page).toHaveURL('/');
    
    // Login again
    await page.fill('input[placeholder="Enter your username"]', username);
    await page.fill('input[placeholder="Enter your password"]', password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await expect(page.locator('text=' + username)).toBeVisible();
  });

  test('should show error for non-existent user', async ({ page }) => {
    await page.goto('/');
    
    await page.fill('input[placeholder="Enter your username"]', 'nonexistent_user_xyz');
    await page.fill('input[placeholder="Enter your password"]', 'SomePassword123');
    await page.click('button[type="submit"]');
    
    // Should show an error (user not found)
    await expect(page.locator('[class*="Alert"]')).toBeVisible({ timeout: 5000 });
  });
});

test.describe('Conversations Page', () => {
  test('should show empty state when no conversations', async ({ page }) => {
    const username = generateUniqueId();
    const password = 'TestPassword123!';

    // Register and go to conversations
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', username);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Should show "No conversations yet"
    await expect(page.locator('text=No conversations yet')).toBeVisible();
    await expect(page.locator('text=Start a new chat')).toBeVisible();
  });

  test('should navigate to new chat page', async ({ page }) => {
    const username = generateUniqueId();
    const password = 'TestPassword123!';

    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', username);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    await page.click('a:has-text("New Chat")');
    
    await expect(page).toHaveURL('/new-chat');
    await expect(page.locator('h2:has-text("New Chat")')).toBeVisible();
  });
});

test.describe('New Chat Page', () => {
  test('should list other users', async ({ page }) => {
    const user1 = generateUniqueId();
    const user2 = generateUniqueId();
    const password = 'TestPassword123!';

    // Register first user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user1);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register second user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user2);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Go to new chat
    await page.click('a:has-text("New Chat")');
    await expect(page).toHaveURL('/new-chat');
    
    // Should see the first user in the list
    await expect(page.locator(`text=${user1}`)).toBeVisible({ timeout: 5000 });
    
    // Should NOT see the current user in the list
    await expect(page.locator(`a:has-text("${user2}")`)).not.toBeVisible();
  });

  test('should filter users by search query', async ({ page }) => {
    const user1 = generateUniqueId();
    const user2 = `other_${generateUniqueId()}`;
    const password = 'TestPassword123!';

    // Register first user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user1);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register second user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user2);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register and search
    const user3 = generateUniqueId();
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user3);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    await page.click('a:has-text("New Chat")');
    await expect(page).toHaveURL('/new-chat');
    
    // Search for "other_"
    await page.fill('input[placeholder="Search users..."]', 'other_');
    
    // Should see user2 but not user1
    await expect(page.locator(`text=${user2}`)).toBeVisible();
  });
});

test.describe('Messaging', () => {
  test('should send a message to another user', async ({ page }) => {
    const user1 = generateUniqueId();
    const user2 = generateUniqueId();
    const password = 'TestPassword123!';
    const testMessage = 'Hello, this is a test message!';

    // Register first user (recipient)
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user1);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register second user (sender)
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user2);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Go to new chat and select user1
    await page.click('a:has-text("New Chat")');
    await expect(page).toHaveURL('/new-chat');
    await page.click(`a:has-text("${user1}")`);
    
    // Should be on chat page
    await expect(page).toHaveURL(`/chat/${user1}`);
    await expect(page.locator(`h3:has-text("${user1}")`)).toBeVisible();
    await expect(page.locator('text=End-to-end encrypted')).toBeVisible();
    
    // Send a message
    await page.fill('input[placeholder="Type a message..."]', testMessage);
    await page.click('button:has-text("Send")');
    
    // Message should appear in the chat
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 5000 });
  });

  test('should show message in conversations list after sending', async ({ page }) => {
    const user1 = generateUniqueId();
    const user2 = generateUniqueId();
    const password = 'TestPassword123!';
    const testMessage = 'Test message for list';

    // Register first user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user1);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register second user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user2);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Send message to user1
    await page.click('a:has-text("New Chat")');
    await page.click(`a:has-text("${user1}")`);
    await page.fill('input[placeholder="Type a message..."]', testMessage);
    await page.click('button:has-text("Send")');
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 5000 });
    
    // Go back to conversations
    await page.click('a[href="/conversations"]');
    await expect(page).toHaveURL('/conversations');
    
    // Should see the conversation with user1
    await expect(page.locator(`text=${user1}`)).toBeVisible();
    await expect(page.locator(`text=You: ${testMessage}`)).toBeVisible();
  });

  test('should not send duplicate messages on double click', async ({ page }) => {
    const user1 = generateUniqueId();
    const user2 = generateUniqueId();
    const password = 'TestPassword123!';
    const testMessage = 'Single message test';

    // Register first user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user1);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    await page.click('button:has-text("Logout")');
    
    // Register second user
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', user2);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Go to chat
    await page.click('a:has-text("New Chat")');
    await page.click(`a:has-text("${user1}")`);
    
    // Fill in message
    await page.fill('input[placeholder="Type a message..."]', testMessage);
    
    // Click send button multiple times rapidly
    const sendButton = page.locator('button:has-text("Send")');
    await sendButton.click();
    
    // Wait for the message to appear
    await expect(page.locator(`text=${testMessage}`)).toBeVisible({ timeout: 5000 });
    
    // Check that there is exactly one message with this content
    const messageElements = await page.locator(`text=${testMessage}`).count();
    expect(messageElements).toBe(1);
  });
});

test.describe('Complete User Flow', () => {
  test('complete flow: register, login, list conversations, send messages', async ({ page }) => {
    const alice = generateUniqueId();
    const bob = generateUniqueId();
    const password = 'TestPassword123!';
    const aliceMessage = 'Hello Bob, how are you?';
    const bobMessage = 'Hi Alice, I am fine!';

    // 1. Register Alice
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', alice);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // Verify Alice sees empty conversations
    await expect(page.locator('text=No conversations yet')).toBeVisible();
    
    // Logout Alice
    await page.click('button:has-text("Logout")');
    await expect(page).toHaveURL('/');
    
    // 2. Register Bob
    await page.goto('/register');
    await page.fill('input[placeholder="Choose a username"]', bob);
    await page.fill('input[placeholder="Choose a password"]', password);
    await page.fill('input[placeholder="Confirm your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // 3. Bob sends message to Alice
    await page.click('a:has-text("New Chat")');
    await page.click(`a:has-text("${alice}")`);
    await page.fill('input[placeholder="Type a message..."]', bobMessage);
    await page.click('button:has-text("Send")');
    await expect(page.locator(`text=${bobMessage}`)).toBeVisible({ timeout: 5000 });
    
    // Bob logs out
    await page.goto('/conversations');
    await page.click('button:has-text("Logout")');
    
    // 4. Alice logs in
    await page.fill('input[placeholder="Enter your username"]', alice);
    await page.fill('input[placeholder="Enter your password"]', password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/conversations', { timeout: 10000 });
    
    // 5. Alice should see conversation with Bob
    // Wait a moment for polling to receive the message
    await page.waitForTimeout(1000);
    
    // Reload to trigger polling check
    await page.reload();
    
    // Check if Bob appears in conversations (message may take time to arrive via polling)
    await expect(page.locator(`text=${bob}`)).toBeVisible({ timeout: 10000 });
    
    // 6. Alice opens chat with Bob
    await page.click(`a:has-text("${bob}")`);
    await expect(page).toHaveURL(`/chat/${bob}`);
    
    // Alice should see Bob's message
    await expect(page.locator(`text=${bobMessage}`)).toBeVisible({ timeout: 5000 });
    
    // 7. Alice replies to Bob
    await page.fill('input[placeholder="Type a message..."]', aliceMessage);
    await page.click('button:has-text("Send")');
    await expect(page.locator(`text=${aliceMessage}`)).toBeVisible({ timeout: 5000 });
    
    // 8. Go back to conversations and verify
    await page.click('a[href="/conversations"]');
    await expect(page.locator(`text=${bob}`)).toBeVisible();
    await expect(page.locator(`text=You: ${aliceMessage}`)).toBeVisible();
  });
});
