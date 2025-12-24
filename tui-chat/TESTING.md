# Testing Guide

## Manual Testing Steps

### 1. Starting the Application
```bash
cd tui-chat
cargo run
```

### 2. Testing Navigation
1. **View Initial State**:
   - You should see 5 conversations in the left panel
   - Right panel should show "No conversation selected" message
   - Bottom shows input box with "Press Enter to type a message"

2. **Navigate Conversations**:
   - Press `↓` or `j` to move down the list
   - Press `↑` or `k` to move up the list
   - The selected conversation should be highlighted with a gray background

3. **View Unread Counts**:
   - "Friends Group" should show [3] unread messages
   - "Carol" should show [1] unread message
   - Other conversations should show no unread indicator

### 3. Testing Conversation Selection
1. Navigate to "Alice" using arrow keys or j/k
2. Press `Enter` to select the conversation
3. You should see:
   - Messages from Alice in the right panel
   - Message timestamps displayed
   - Different colors for your messages (green) vs Alice's messages (yellow)
   - Input box now shows "Type your message..." with yellow border

### 4. Testing Message Input
1. While in a conversation (after pressing Enter):
   - Type some text: "Hello there!"
   - Press `Enter` to send
   - Your message should appear at the bottom of the conversation
   - The message should be marked as "You" in green

2. Test multi-line messages:
   - Type "This is line 1"
   - Press `Shift+Enter` for new line
   - Type "This is line 2"
   - Press `Enter` to send
   - Both lines should appear in your message

3. Test canceling:
   - Type some text
   - Press `Esc`
   - Text should be cleared and input box deactivated

### 5. Testing Deselection
1. While viewing a conversation, press `Esc`
2. The right panel should clear and show "No conversation selected"
3. You should return to navigation mode

### 6. Testing Different Conversations
1. Navigate to "Dev Team" and press Enter
2. You should see group chat messages from multiple people
3. Navigate to "Friends Group" and press Enter
4. The unread count [3] should disappear once selected

### 7. Exit the Application
- Press `q` or `Q` from navigation mode
- Application should exit cleanly

## Expected Visual Layout

```
┌─ Conversations ─────────┐┌─ 👩 Alice ──────────────────────────────┐
│👩 Alice          06:05  ││Alice [04:05]                            │
│  Doing great! Want...   ││  Hey! How are you doing?                │
│👨 Bob            11:05  ││                                         │
│  Yes! Just submitted... ││You [04:10]                              │
│💻 Dev Team       02:08  ││  I'm good! How about you?               │
│  On my way              ││                                         │
│👩‍💼 Carol          06:35  ││Alice [06:05]                            │
│  Can you review...  [1] ││  Doing great! Want to grab lunch...    │
│🎉 Friends Group   06:20 ││                                         │
│  How about 7 PM?    [3] ││                                         │
└─────────────────────────┘└─────────────────────────────────────────┘
                           ┌─ Type your message... ──────────────────┐
                           │                                         │
                           │                                         │
                           └─────────────────────────────────────────┘
```

## Features Verified

✅ Conversation list displays on the left (30% width)
✅ Message view displays on the right (70% width)
✅ Multi-line text input at bottom
✅ Keyboard navigation (↑/↓, j/k)
✅ Conversation selection with Enter
✅ Conversation deselection with Escape
✅ Message sending with Enter
✅ Multi-line message input with Shift+Enter
✅ Unread count display
✅ Timestamp formatting
✅ Different colors for own vs other messages
✅ Emoji avatars display
✅ Mock data with 5 conversations
✅ Smooth keyboard navigation
✅ Clean exit with 'q' or 'Q'
