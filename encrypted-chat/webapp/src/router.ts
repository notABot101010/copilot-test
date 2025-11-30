import { createRouter } from '@copilot-test/preact-router';
import { LoginPage } from './pages/LoginPage';
import { RegisterPage } from './pages/RegisterPage';
import { ConversationsPage } from './pages/ConversationsPage';
import { ChatPage } from './pages/ChatPage';
import { NewChatPage } from './pages/NewChatPage';

export const router = createRouter({
  mode: 'history',
  routes: [
    {
      path: '/',
      name: 'login',
      component: LoginPage,
    },
    {
      path: '/register',
      name: 'register',
      component: RegisterPage,
    },
    {
      path: '/conversations',
      name: 'conversations',
      component: ConversationsPage,
    },
    {
      path: '/chat/:username',
      name: 'chat',
      component: ChatPage,
    },
    {
      path: '/new-chat',
      name: 'new-chat',
      component: NewChatPage,
    },
  ],
});
