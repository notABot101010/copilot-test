import { BlockNoteView } from "@blocknote/mantine";
import { useCreateBlockNote } from "@blocknote/react";
import { MantineProvider } from "@mantine/core";

import './index.css';
import '@mantine/core/styles.css';
import "@blocknote/mantine/blocknoteStyles.css";
import './blocknote_overrides.css';
import { createRouter, RouterProvider, RouterView } from "@copilot-test/signals-router";
// import { useSignal, useSignalEffect } from "@preact/signals-react";
// import "@blocknote/core/fonts/inter.css";


export default function App() {
  // Creates a new editor instance.
  // Renders the editor instance using a React component.
  return <MantineProvider>
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    </MantineProvider>
}


function NotFound() {
  return (
    <div className="flex items-center justify-center h-screen bg-gray-100">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-gray-900 mb-2">Page Not Found</h1>
        <a href="/" className="text-blue-600 hover:underline">
          Go to Home
        </a>
      </div>
    </div>
  );
}

function Editor() {
  // Creates a new editor instance.
  const editor = useCreateBlockNote();

  return (
      <BlockNoteView editor={editor} className="h-full"/>
  )
}

const router = createRouter({
  routes: [
    { path: '/', component: Editor },
    { path: '/*', component: NotFound },
  ],
});
