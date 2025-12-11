import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('Document Persistence', () => {
  it('should preserve text content after save and load', () => {
    // Create a document and add content
    let doc = createBlockNoteDocument();

    const block = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello World!' }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'insert',
        block: block as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    doc = applyBlockNoteChanges(doc, changes);

    // Verify content is there
    expect(doc.blocks).toHaveLength(1);
    expect(doc.blocks[0].content[0].text).toBe('Hello World!');

    // Save to binary
    const saved = Automerge.save(doc);

    // Load from binary
    const loaded = Automerge.load(saved);

    // Verify content is still there
    expect(loaded.blocks).toHaveLength(1);
    expect(loaded.blocks[0].content[0].text).toBe('Hello World!');
  });

  it('should preserve text content after update and reload', () => {
    // Create a document with initial content
    let doc = createBlockNoteDocument();

    const originalBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: originalBlock as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ]);

    // Update the text
    const updatedBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello World' }],
      children: [],
    };

    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ]);

    // Verify updated content
    expect(doc.blocks[0].content[0].text).toBe('Hello World');

    // Save and reload
    const saved = Automerge.save(doc);
    const loaded = Automerge.load(saved);

    // Verify content persists
    expect(loaded.blocks).toHaveLength(1);
    expect(loaded.blocks[0].content[0].text).toBe('Hello World');
  });

  it('should preserve multiple text edits after reload', () => {
    let doc = createBlockNoteDocument();

    // Insert initial block
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'H' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    // Simulate typing character by character
    const texts = ['He', 'Hel', 'Hell', 'Hello'];
    let prevText = 'H';

    for (const text of texts) {
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: prevText }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);
      prevText = text;
    }

    // Verify final state
    expect(doc.blocks[0].content[0].text).toBe('Hello');

    // Save and reload
    const saved = Automerge.save(doc);
    const loaded = Automerge.load(saved);

    // Verify persistence
    expect(loaded.blocks).toHaveLength(1);
    expect(loaded.blocks[0].content[0].text).toBe('Hello');
  });
});
