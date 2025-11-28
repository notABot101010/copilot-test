import { useState } from 'preact/hooks'
import viteLogo from '/vite.svg'
import './app.css'
import { createRouter, RouterProvider, RouterView, useRoute, useRouter } from '@copilot-test/preact-router';
import { useSignal } from "@preact/signals";

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/test', name: 'test', component: TestPage },
    { path: '/test2', name: 'test2', component: TestPage },
    { path: '/nothing', name: 'nothing', redirect: '/test2' },
  ],
  mode: 'history' // or 'hash'
});

router.beforeEach((to) => {
  if (to.fullPath === '/nothing') {
    return '/';
  }
})

export function App() {
  return (
    <RouterProvider router={router}>
  {/* //     <nav>
  //       <a href="/">Home</a>
  //       <a href="/test">Test</a>
  //     </nav>
  //   <RouterView /> */}
  <RouterView />
 </RouterProvider>)

}

export function Home() {
  const count = useSignal(0);

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} class="logo" alt="Vite logo" />
        </a>
        <a href="/test">
          Test
        </a>
      </div>
      <h1>Vite + Preact</h1>
      <div class="card">
        <button onClick={() => count.value += 1}>
          count is {count}
        </button>
        <p>
          Edit <code>src/app.tsx</code> and save to test HMR
        </p>
      </div>
      <p>
        Check out{' '}
        <a
          href="https://preactjs.com/guide/v10/getting-started#create-a-vite-powered-preact-app"
          target="_blank"
        >
          create-preact
        </a>
        , the official Preact + Vite starter
      </p>
      <p class="read-the-docs">
        Click on the Vite and Preact logos to learn more
      </p>
    </>
  )
}

function TestPage() {
  const route = useRoute();
  const router = useRouter();

  return (<div>
    <h1>{route.value.fullPath}</h1>

    <a href="/nothing">Nothing</a>
    <a href="/test2">Test 2</a>

    <button onClick={() => router.back()}>Back</button>

    </div>
  )
}
