import { signal } from '@preact/signals';
import { createRouter, RouterProvider, RouterView } from '@copilot-test/preact-router';
import Register from './components/Register';
import Login from './components/Login';
import Chat from './components/Chat';
import { getCurrentUser } from './crypto/storage';

export const currentUser = signal<string | null>(getCurrentUser());

const router = createRouter({
  routes: [
    { path: '/', component: Login },
    { path: '/register', component: Register },
    { path: '/login', component: Login },
    { path: '/chat', component: Chat },
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
