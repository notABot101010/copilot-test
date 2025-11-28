import { createRouter } from '@copilot-test/preact-router';
import { HomePage } from './pages/HomePage';
import { ServerHomePage } from './pages/ServerHomePage';
import { ChannelPage } from './pages/ChannelPage';
import { CreateServerPage } from './pages/CreateServerPage';

export const router = createRouter({
  mode: 'history',
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomePage,
    },
    {
      path: '/server/create',
      name: 'create-server',
      component: CreateServerPage,
    },
    {
      path: '/server/:serverId',
      name: 'server-home',
      component: ServerHomePage,
    },
    {
      path: '/server/:serverId/channels/:channelId',
      name: 'channel',
      component: ChannelPage,
    },
  ],
});
