import { createRoot } from 'react-dom/client'
import './index.css'
import { App } from './app.tsx'

const root = createRoot(document.getElementById('app')!)
root.render(<App />)
