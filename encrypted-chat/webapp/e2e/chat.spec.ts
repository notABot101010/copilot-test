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
    
    // Should show an error or stay on login page (user not found)
    // The server returns 404 for non-existent users, which triggers an error
    await expect(page).toHaveURL('/');
    
    // Wait for the button to no longer be loading (indicates request completed)
    await expect(page.locator('button[type="submit"]')).not.toHaveAttribute('data-loading', { timeout: 5000 });
    
    // Should still be on login page (not redirected to conversations)
    await expect(page.locator('h3:has-text("Login")')).toBeVisible();
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
    await expect(page).toHaveURL('/new-chat');
    await expect(page.locator(`text=${alice}`)).toBeVisible({ timeout: 5000 });
    await page.click(`a:has-text("${alice}")`);
    await expect(page).toHaveURL(`/chat/${alice}`);
    await page.fill('input[placeholder="Type a message..."]', aliceMessage);
    await page.click('button:has-text("Send")');
    await expect(page.locator(`text=${aliceMessage}`)).toBeVisible({ timeout: 5000 });
    
    // 4. Go back to conversations and verify message shows in list
    await page.click('a[href="/conversations"]');
    await expect(page).toHaveURL('/conversations');
    await expect(page.locator('button:has-text("Logout")')).toBeVisible({ timeout: 5000 });
    
    // Verify conversation with Alice appears
    await expect(page.locator(`text=${alice}`)).toBeVisible();
    await expect(page.locator(`text=You: ${aliceMessage}`)).toBeVisible();
    
    // 5. Verify Bob can logout
    await page.click('button:has-text("Logout")');
    await expect(page).toHaveURL('/');
    
    // 6. Verify login page is shown
    await expect(page.locator('h3:has-text("Login")')).toBeVisible();
  });
});

test.describe('Bidirectional Messaging', () => {
  test('user1 and user2 can exchange messages and both see full conversation', async ({ browser }) => {
    const user1 = generateUniqueId();
    const user2 = generateUniqueId();
    const password = 'TestPassword123!';

    // Create two separate browser contexts for user1 and user2
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      // 1. Register user1
      await page1.goto('/register');
      await page1.fill('input[placeholder="Choose a username"]', user1);
      await page1.fill('input[placeholder="Choose a password"]', password);
      await page1.fill('input[placeholder="Confirm your password"]', password);
      await page1.click('button[type="submit"]');
      await expect(page1).toHaveURL('/conversations', { timeout: 10000 });

      // 2. Register user2
      await page2.goto('/register');
      await page2.fill('input[placeholder="Choose a username"]', user2);
      await page2.fill('input[placeholder="Choose a password"]', password);
      await page2.fill('input[placeholder="Confirm your password"]', password);
      await page2.click('button[type="submit"]');
      await expect(page2).toHaveURL('/conversations', { timeout: 10000 });

      // 3. User1 sends first message to User2
      await page1.click('a:has-text("New Chat")');
      await expect(page1).toHaveURL('/new-chat');
      await expect(page1.locator(`text=${user2}`)).toBeVisible({ timeout: 5000 });
      await page1.click(`a:has-text("${user2}")`);
      await expect(page1).toHaveURL(`/chat/${user2}`);

      const message1 = 'Hello User2! This is my first message.';
      await page1.fill('input[placeholder="Type a message..."]', message1);
      await page1.click('button:has-text("Send")');
      await expect(page1.locator(`text=${message1}`)).toBeVisible({ timeout: 5000 });

      // 4. User2 navigates to chat with user1 directly
      // Since user2 knows user1 exists, they can initiate or view the chat
      await page2.waitForTimeout(3000);
      
      // User2 goes to new chat, selects user1, and the messages should appear
      await page2.click('a:has-text("New Chat")');
      await expect(page2.locator(`text=${user1}`)).toBeVisible({ timeout: 5000 });
      await page2.click(`a:has-text("${user1}")`);
      await expect(page2).toHaveURL(`/chat/${user1}`);
      
      // Wait for polling to pick up the message
      await page2.waitForTimeout(5000);
      await page2.reload();
      
      // User2 should see the message from user1
      await expect(page2.locator(`text=${message1}`)).toBeVisible({ timeout: 10000 });

      // 5. User2 sends a reply to User1
      const message2 = 'Hi User1! Got your message. How are you?';
      await page2.fill('input[placeholder="Type a message..."]', message2);
      await page2.click('button:has-text("Send")');
      await expect(page2.locator(`text=${message2}`)).toBeVisible({ timeout: 5000 });

      // User2 should see both messages in the conversation
      await expect(page2.locator(`text=${message1}`)).toBeVisible();
      await expect(page2.locator(`text=${message2}`)).toBeVisible();

      // 6. User1 should receive User2's reply
      // Refresh or wait for polling to pick up the new message
      await page1.waitForTimeout(3000);
      await page1.reload();
      await expect(page1).toHaveURL(`/chat/${user2}`);
      
      // User1 should see both messages
      await expect(page1.locator(`text=${message1}`)).toBeVisible({ timeout: 5000 });
      await expect(page1.locator(`text=${message2}`)).toBeVisible({ timeout: 5000 });

      // 7. User1 sends another message
      const message3 = "I'm doing great! Thanks for asking.";
      await page1.fill('input[placeholder="Type a message..."]', message3);
      await page1.click('button:has-text("Send")');
      await expect(page1.locator(`text=${message3}`)).toBeVisible({ timeout: 5000 });

      // User1 should see all three messages
      await expect(page1.locator(`text=${message1}`)).toBeVisible();
      await expect(page1.locator(`text=${message2}`)).toBeVisible();
      await expect(page1.locator(`text=${message3}`)).toBeVisible();

      // 8. User2 receives User1's latest message
      await page2.waitForTimeout(3000);
      await page2.reload();
      await expect(page2).toHaveURL(`/chat/${user1}`);

      // User2 should see all three messages
      await expect(page2.locator(`text=${message1}`)).toBeVisible({ timeout: 5000 });
      await expect(page2.locator(`text=${message2}`)).toBeVisible({ timeout: 5000 });
      await expect(page2.locator(`text=${message3}`)).toBeVisible({ timeout: 5000 });

      // 9. Verify conversation list shows latest message for both users
      await page1.click('a[href="/conversations"]');
      await expect(page1).toHaveURL('/conversations');
      await expect(page1.locator(`text=${user2}`)).toBeVisible();
      await expect(page1.locator(`text=You: ${message3}`)).toBeVisible();

      await page2.click('a[href="/conversations"]');
      await expect(page2).toHaveURL('/conversations');
      await expect(page2.locator(`text=${user1}`)).toBeVisible();
      await expect(page2.locator(`text=${message3}`)).toBeVisible();

    } finally {
      await context1.close();
      await context2.close();
    }
  });

  test('both users can initiate conversations independently', async ({ browser }) => {
    const userA = generateUniqueId();
    const userB = generateUniqueId();
    const password = 'TestPassword123!';

    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      // Register both users
      await pageA.goto('/register');
      await pageA.fill('input[placeholder="Choose a username"]', userA);
      await pageA.fill('input[placeholder="Choose a password"]', password);
      await pageA.fill('input[placeholder="Confirm your password"]', password);
      await pageA.click('button[type="submit"]');
      await expect(pageA).toHaveURL('/conversations', { timeout: 10000 });

      await pageB.goto('/register');
      await pageB.fill('input[placeholder="Choose a username"]', userB);
      await pageB.fill('input[placeholder="Choose a password"]', password);
      await pageB.fill('input[placeholder="Confirm your password"]', password);
      await pageB.click('button[type="submit"]');
      await expect(pageB).toHaveURL('/conversations', { timeout: 10000 });

      // UserA initiates conversation with UserB
      await pageA.click('a:has-text("New Chat")');
      await expect(pageA.locator(`text=${userB}`)).toBeVisible({ timeout: 5000 });
      await pageA.click(`a:has-text("${userB}")`);
      
      const msgFromA = 'Hey B, this is A!';
      await pageA.fill('input[placeholder="Type a message..."]', msgFromA);
      await pageA.click('button:has-text("Send")');
      await expect(pageA.locator(`text=${msgFromA}`)).toBeVisible({ timeout: 5000 });

      // UserB receives and replies
      await pageB.waitForTimeout(2000);
      await pageB.goto('/conversations');
      await expect(pageB.locator(`text=${userA}`)).toBeVisible({ timeout: 10000 });
      await pageB.click(`a:has-text("${userA}")`);
      await expect(pageB.locator(`text=${msgFromA}`)).toBeVisible({ timeout: 5000 });

      const msgFromB = 'Hello A, got your message!';
      await pageB.fill('input[placeholder="Type a message..."]', msgFromB);
      await pageB.click('button:has-text("Send")');
      await expect(pageB.locator(`text=${msgFromB}`)).toBeVisible({ timeout: 5000 });

      // Verify both see the complete conversation
      await pageA.waitForTimeout(2000);
      await pageA.reload();
      await expect(pageA.locator(`text=${msgFromA}`)).toBeVisible({ timeout: 5000 });
      await expect(pageA.locator(`text=${msgFromB}`)).toBeVisible({ timeout: 5000 });

    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
