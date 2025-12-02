import { createRouter } from '@copilot-test/preact-router';
import type { RouteRecord } from '@copilot-test/preact-router';
import { ChatPage } from './views/ChatPage';
import { CreateChannelPage } from './views/CreateChannelPage';
import { ExplorePage } from './views/ExplorePage';
import { SettingsPage } from './views/SettingsPage';

export const routes: RouteRecord[] = [
  { path: '/', name: 'chats', component: ChatPage },
  { path: '/chats/:chatId', name: 'chat', component: ChatPage },
  { path: '/settings', name: 'settings', component: SettingsPage },
  { path: '/create-channel', name: 'createChannel', component: CreateChannelPage },
  { path: '/explore', name: 'explore', component: ExplorePage },
];

export const router = createRouter({
  mode: 'history',
  routes,
});
