import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('HandleUpdate Bug - Wrong oldText assumption', () => {
  it('should fail when document state differs from prevBlock', () => {
    // This test demonstrates the bug where handleUpdate assumes
    // the current document state matches prevBlock, but it doesn't

    let doc = createBlockNoteDocument();

    // Insert initial block
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'A' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    console.log('After insert, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('A');

    // First update: A -> AB
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'AB' }],
          children: [],
        } as any,
        prevBlock: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'A' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    console.log('After first update, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('AB');

    // Save and reload
    const saved1 = Automerge.save(doc);
    const loaded1 = Automerge.load(saved1);
    console.log('After first save/load, text:', loaded1.blocks[0].content[0].text);
    expect(loaded1.blocks[0].content[0].text).toBe('AB');

    // Second update: AB -> ABC
    // BUT if the prevBlock is wrong or document was reloaded,
    // the current state might not match prevBlock
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'ABC' }],
          children: [],
        } as any,
        prevBlock: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'AB' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    console.log('After second update, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('ABC');

    // Save and reload
    const saved2 = Automerge.save(doc);
    const loaded2 = Automerge.load(saved2);
    console.log('After second save/load, text:', loaded2.blocks[0].content[0].text);
    expect(loaded2.blocks[0].content[0].text).toBe('ABC');
  });

  it('should demonstrate the actual bug scenario', () => {
    // Simulate what happens in the real app:
    // 1. User types "Hello"
    // 2. Document is saved
    // 3. Page reloads
    // 4. Document is loaded from storage
    // 5. User types more text
    // 6. The prevBlock passed to handleUpdate refers to BlockNote's view,
    //    not the Automerge document state

    let doc = createBlockNoteDocument();

    // Initial insert
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    // Save
    const saved = Automerge.save(doc);

    // Simulate reload: load from storage
    let loadedDoc = Automerge.load(saved);
    console.log('After load, text:', loadedDoc.blocks[0].content[0].text);

    // Now simulate user editing after reload
    // The editor's prevBlock would be the state before the edit in the editor,
    // but it might not perfectly match what's in loadedDoc

    // User types " World", so prevBlock has "Hello" and new block has "Hello World"
    loadedDoc = applyBlockNoteChanges(loadedDoc, [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello World' }],
          children: [],
        } as any,
        prevBlock: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    console.log('After update post-reload, text:', loadedDoc.blocks[0].content[0].text);
    expect(loadedDoc.blocks[0].content[0].text).toBe('Hello World');

    // Save and reload again
    const saved2 = Automerge.save(loadedDoc);
    const loaded2 = Automerge.load(saved2);
    console.log('After second reload, text:', loaded2.blocks[0].content[0].text);
    
    // THIS IS WHERE THE BUG WOULD MANIFEST
    // If the text is not "Hello World", we have a bug
    expect(loaded2.blocks[0].content[0].text).toBe('Hello World');
  });
});
