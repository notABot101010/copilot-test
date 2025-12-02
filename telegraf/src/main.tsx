import { render } from 'preact'
import './index.css'
import { App } from './app'
import { RouterProvider } from '@copilot-test/preact-router'
import { router } from './router'

render(
  <RouterProvider router={router}>
    <App />
  </RouterProvider>,
  document.getElementById('app')!,
)
