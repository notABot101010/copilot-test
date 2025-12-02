import { createRouter, RouterProvider, RouterView, useRouter } from '@copilot-test/preact-router';
import Home from './components/Home';
import Call from './components/Call';

const router = createRouter({
  routes: [
    { path: '/', component: HomePage },
    { path: '/call/:roomId', component: CallPage },
  ],
});

function HomePage() {
  const routerInstance = useRouter();
  
  function handleStartCall(roomId: string) {
    // Navigate to call page with the room ID
    routerInstance.push(`/call/${roomId}`);
  }

  return <Home onStartCall={handleStartCall} />;
}

function CallPage() {
  const routerInstance = useRouter();
  const params = routerInstance.currentRoute.value?.params;
  const roomId = params?.roomId as string;

  function handleEnd() {
    routerInstance.push('/');
  }

  if (!roomId) {
    return (
      <div class="min-h-screen bg-gray-100 flex items-center justify-center">
        <p class="text-gray-600">Invalid room ID</p>
      </div>
    );
  }

  return <Call roomId={roomId} onEnd={handleEnd} />;
}

export default function App() {
  return (
    <RouterProvider router={router}>
      <RouterView />
    </RouterProvider>
  );
}
