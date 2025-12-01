import { render } from 'preact'
import { MantineProvider } from '@mantine/core'
import App from './app'
import './index.css'
import '@mantine/core/styles.css'

render(
  <MantineProvider>
    <App />
  </MantineProvider>,
  document.getElementById('app')!
)
