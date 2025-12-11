import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('Automerge Text Type Issues', () => {
  it('should check if text becomes Automerge.Text after splice', () => {
    let doc = createBlockNoteDocument();

    // Insert initial block
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

    console.log('After insert, text type:', typeof doc.blocks[0].content[0].text);
    console.log('After insert, text value:', doc.blocks[0].content[0].text);
    console.log('After insert, text is string?:', typeof doc.blocks[0].content[0].text === 'string');

    // Now update with surgical splice
    doc = applyBlockNoteChanges(doc, [
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

    console.log('After update, text type:', typeof doc.blocks[0].content[0].text);
    console.log('After update, text value:', doc.blocks[0].content[0].text);
    console.log('After update, text is string?:', typeof doc.blocks[0].content[0].text === 'string');
    console.log('After update, text constructor:', doc.blocks[0].content[0].text.constructor.name);

    // Check if it's an Automerge text object
    const textValue = doc.blocks[0].content[0].text;
    console.log('Is Array?:', Array.isArray(textValue));
    console.log('Has join method?:', typeof textValue.join === 'function');

    // Try to convert to string
    if (typeof textValue !== 'string' && typeof textValue.join === 'function') {
      const asString = textValue.join('');
      console.log('Converted to string:', asString);
      expect(asString).toBe('Hello World');
    } else {
      expect(textValue).toBe('Hello World');
    }
  });

  it('should check what happens after save/load cycle', () => {
    let doc = createBlockNoteDocument();

    // Insert and update
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

    doc = applyBlockNoteChanges(doc, [
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

    // Save and load
    const saved = Automerge.save(doc);
    const loaded = Automerge.load(saved);

    console.log('After load, text type:', typeof loaded.blocks[0].content[0].text);
    console.log('After load, text value:', loaded.blocks[0].content[0].text);
    console.log('After load, text constructor:', loaded.blocks[0].content[0].text.constructor.name);

    const textValue = loaded.blocks[0].content[0].text;
    console.log('After load, is Array?:', Array.isArray(textValue));
    console.log('After load, has join method?:', typeof textValue.join === 'function');

    // Check if we can use it as a string
    if (typeof textValue !== 'string') {
      console.log('Text is NOT a plain string after load!');
      if (typeof textValue.join === 'function') {
        console.log('Converting with join:', textValue.join(''));
      }
      if (typeof textValue.toString === 'function') {
        console.log('Converting with toString:', textValue.toString());
      }
    }
  });
});
