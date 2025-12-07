import { BlockNoteView } from "@blocknote/mantine";
import { useCreateBlockNote } from "@blocknote/react";
import { MantineProvider } from "@mantine/core";

import './index.css';
import '@mantine/core/styles.css';
import "@blocknote/mantine/blocknoteStyles.css";
import './blocknote_overrides.css';
// import { useSignal, useSignalEffect } from "@preact/signals-react";
// import "@blocknote/core/fonts/inter.css";


export default function App() {
  // Creates a new editor instance.
  const editor = useCreateBlockNote();

  // Renders the editor instance using a React component.
  return <MantineProvider>
      <BlockNoteView editor={editor} className="h-full"/>
    </MantineProvider>
}
