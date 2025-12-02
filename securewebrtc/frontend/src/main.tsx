import { render } from 'preact'
import { MantineProvider } from '@mantine/core'
import App from './app'
import './index.css'

render(
  <MantineProvider>
    <App />
  </MantineProvider>,
  document.getElementById('app')!
)
