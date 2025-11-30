import { render } from 'preact'
import '@mantine/core/styles.css'
import './index.css'
import { App } from './app.tsx'

render(<App />, document.getElementById('app')!)
