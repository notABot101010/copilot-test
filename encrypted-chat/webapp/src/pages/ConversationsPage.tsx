import { Button, Paper, Container, Title, Badge } from '@mantine/core';
import { currentUser, logout, getConversationList } from '../services/chatService';
import { router } from '../router';

export function ConversationsPage() {
  const user = currentUser.value;
  
  // Redirect if not logged in
  if (!user) {
    router.push('/');
    return null;
  }

  // Subscribe to conversations signal
  const conversationList = getConversationList();

  function handleLogout() {
    logout();
    router.push('/');
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const isToday = date.toDateString() === now.toDateString();
    
    if (isToday) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }

  return (
    <Container size="md" className="min-h-screen py-4">
      <Paper shadow="md" p="md" radius="md" className="mb-4">
        <div className="flex justify-between items-center">
          <div>
            <Title order={2}>Messages</Title>
            <p className="text-gray-400 text-sm">Logged in as {user.username}</p>
          </div>
          <div className="flex gap-2">
            <a href="/new-chat">
              <Button variant="filled">New Chat</Button>
            </a>
            <Button variant="subtle" color="red" onClick={handleLogout}>
              Logout
            </Button>
          </div>
        </div>
      </Paper>

      {conversationList.length === 0 ? (
        <Paper shadow="md" p="xl" radius="md" className="text-center">
          <p className="text-gray-400 mb-4">No conversations yet</p>
          <a href="/new-chat">
            <Button>Start a new chat</Button>
          </a>
        </Paper>
      ) : (
        <div className="space-y-2">
          {conversationList.map((conv) => {
            const lastMessage = conv.messages[conv.messages.length - 1];
            return (
              <a 
                key={conv.peerUsername} 
                href={`/chat/${conv.peerUsername}`}
                className="block no-underline"
              >
                <Paper 
                  shadow="sm" 
                  p="md" 
                  radius="md" 
                  className="hover:bg-gray-800 transition-colors cursor-pointer"
                >
                  <div className="flex justify-between items-start">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold text-white m-0">{conv.peerUsername}</h3>
                        {conv.unread > 0 && (
                          <Badge color="blue" size="sm">{conv.unread}</Badge>
                        )}
                      </div>
                      {lastMessage && (
                        <p className="text-gray-400 text-sm mt-1 truncate m-0">
                          {lastMessage.isOutgoing ? 'You: ' : ''}
                          {lastMessage.content}
                        </p>
                      )}
                    </div>
                    {lastMessage && (
                      <span className="text-gray-500 text-xs ml-2 flex-shrink-0">
                        {formatTime(lastMessage.timestamp)}
                      </span>
                    )}
                  </div>
                </Paper>
              </a>
            );
          })}
        </div>
      )}
    </Container>
  );
}
