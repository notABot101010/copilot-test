import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BlockNoteView } from "@blocknote/mantine";
import { useCreateBlockNote } from "@blocknote/react";
import * as Y from "yjs";
import * as Automerge from "@automerge/automerge";

import './index.css';
import '@mantine/core/styles.css';
import "@blocknote/mantine/blocknoteStyles.css";
import { WebrtcProvider } from "y-webrtc";
import { applyBlockNoteChanges, type BlockNoteDocument } from "@copilot-test/automerge-utils";

const ydoc = new Y.Doc();

// Initialize WebRTC provider for collaborative editing
new WebrtcProvider("my-document-id", ydoc);

const DOCUMENT_LOCAL_STORAGE_KEY = 'BLOCKNOTE_COLLAB_AUTOMERGE_DOC';

function loadOrCreateAutomergeDoc(): Automerge.Doc<BlockNoteDocument<any, any, any>> {
  let savedDoc = localStorage.getItem(DOCUMENT_LOCAL_STORAGE_KEY);

  if (savedDoc) {
    const t0 = performance.now();
    const doc = Automerge.load(base64ToUint8Array(savedDoc));
    const t1 = performance.now();
    console.log(`Automerge document loaded in ${t1 - t0} ms`)
    return doc as any;
  }
  return Automerge.from({
    blocks: [],
  }) as any;
}

let automergeDoc = loadOrCreateAutomergeDoc();


function App() {
  const editor = useCreateBlockNote({
    initialContent: automergeDoc.blocks.length === 0 ? undefined : automergeDoc.blocks,
    // collaboration: {
    //   provider: yProvider,
    //   fragment: ydoc.getXmlFragment("blocks"),
    //   user: {
    //     name: "My Username",
    //     color: "#ff0000",
    //   },
    // },
  });

  editor.onChange((_editor, { getChanges }) => {
      const changes = getChanges();
      automergeDoc = applyBlockNoteChanges(automergeDoc, changes);
      console.log(`Automerge Doc size: ${Automerge.save(automergeDoc).length}`);
      console.log(`Yjs Doc size: ${Y.encodeStateAsUpdateV2(ydoc).length}`)
      localStorage.setItem(DOCUMENT_LOCAL_STORAGE_KEY, uint8ArrayToBase64(Automerge.save(automergeDoc)));
  });

  return <BlockNoteView editor={editor} />;
}


createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {/* <MantineProvider> */}
      <App />
    {/* </MantineProvider> */}
  </StrictMode>,
)


  function base64ToUint8Array(base64: string): Uint8Array {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let idx = 0; idx < binary.length; idx += 1) {
      bytes[idx] = binary.charCodeAt(idx);
    }
    return bytes;
  }

  function uint8ArrayToBase64(bytes: Uint8Array): string {
    let binary = "";
    for (let idx = 0; idx < bytes.length; idx += 1) {
      binary += String.fromCharCode(bytes[idx]);
    }
    return btoa(binary);
  }
