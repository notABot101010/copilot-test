import { signal, computed } from '@preact/signals';
import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import Login from './components/Login';
import Register from './components/Register';
import Groups from './components/Groups';
import Chat from './components/Chat';
import Channels from './components/Channels';
import CreateGroup from './components/CreateGroup';
import CreateChannel from './components/CreateChannel';
import InviteMembers from './components/InviteMembers';

// Auth state
export interface UserState {
  username: string;
  userId: number;
}

const storedUser = localStorage.getItem('mls_chat_user');
export const currentUser = signal<UserState | null>(storedUser ? JSON.parse(storedUser) : null);
export const isLoggedIn = computed(() => currentUser.value !== null);

export function setCurrentUser(user: UserState | null) {
  currentUser.value = user;
  if (user) {
    localStorage.setItem('mls_chat_user', JSON.stringify(user));
  } else {
    localStorage.removeItem('mls_chat_user');
  }
}

const router = createRouter({
  routes: [
    { path: '/', component: Login },
    { path: '/login', component: Login },
    { path: '/register', component: Register },
    { path: '/groups', component: Groups },
    { path: '/groups/create', component: CreateGroup },
    { path: '/groups/:groupId', component: Chat },
    { path: '/groups/:groupId/invite', component: InviteMembers },
    { path: '/channels', component: Channels },
    { path: '/channels/create', component: CreateChannel },
  ],
});

export default function App() {
  return (
    <RouterProvider router={router}>
      <div class="min-h-screen bg-gray-100">
        <RouterView />
      </div>
    </RouterProvider>
  );
}
