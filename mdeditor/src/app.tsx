import { MantineProvider } from '@mantine/core'
import { Sidebar } from './components/Sidebar'
import { Editor } from './components/Editor'

export function App() {
  return (
    <MantineProvider>
      <div className="flex h-screen overflow-hidden">
        <div className="w-64 md:w-80 flex-shrink-0">
          <Sidebar />
        </div>
        <div className="flex-1 overflow-hidden">
          <Editor />
        </div>
      </div>
    </MantineProvider>
  )
}
