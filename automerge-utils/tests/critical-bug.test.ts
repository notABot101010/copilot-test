import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('Critical Bug - updateTextFieldSurgically uses wrong text', () => {
  it('should fail when current document text differs from prevBlock.text', () => {
    // This demonstrates the critical bug:
    // updateTextFieldSurgically compares oldText (from prevBlock) with newText,
    // but then applies Automerge.splice to obj[fieldName] which might have different content!
    
    let doc = createBlockNoteDocument();

    // Insert initial block with text "A"
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

    expect(doc.blocks[0].content[0].text).toBe('A');

    // Update: A -> AB
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

    console.log('After update, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('AB');

    // Save and reload (simulating page reload)
    const saved = Automerge.save(doc);
    doc = Automerge.load(saved);
    
    console.log('After save/load, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('AB');

    // Now here's the bug scenario:
    // User types more text. prevBlock has "AB" but let's say due to some
    // concurrent update or state mismatch, the document actually has "ABC"
    // (This could happen in real collaborative scenarios)
    
    // Manually modify the document to simulate a state mismatch
    doc = Automerge.change(doc, d => {
      (d.blocks[0].content[0] as any).text = 'ABC';
    });
    
    console.log('After manual change, text:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('ABC');

    // Now apply an update where prevBlock says "AB" and newBlock says "ABD"
    // The function will calculate: prefix="AB", deleteCount=0, insertText="D"
    // But it applies this to the current document which has "ABC", not "AB"!
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'ABD' }],
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

    console.log('After buggy update, text:', doc.blocks[0].content[0].text);
    // This will likely NOT be "ABD" as expected, but something like "ABCD"
    // because the splice was calculated based on "AB"->"ABD" but applied to "ABC"
    expect(doc.blocks[0].content[0].text).toBe('ABD');
  });

  it('should demonstrate the real-world bug scenario', () => {
    // Real scenario from the problem statement:
    // 1. Type some text
    // 2. Reload page
    // 3. Type more text
    // 4. Text is wrong after reload
    
    let doc = createBlockNoteDocument();

    // User types "Hello"
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

    console.log('Step 1 - Typed "Hello":', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('Hello');

    // Save to localStorage (simulation)
    const saved1 = Automerge.save(doc);
    
    // Page reload - load from storage
    doc = Automerge.load(saved1);
    console.log('Step 2 - After reload:', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('Hello');

    // User types " World", but the prevBlock in the update might be slightly off
    // due to how BlockNote tracks changes
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

    console.log('Step 3 - After typing " World":', doc.blocks[0].content[0].text);
    expect(doc.blocks[0].content[0].text).toBe('Hello World');

    // Save and reload again
    const saved2 = Automerge.save(doc);
    doc = Automerge.load(saved2);
    
    console.log('Step 4 - After second reload:', doc.blocks[0].content[0].text);
    
    // BUG: Text might be corrupted here
    expect(doc.blocks[0].content[0].text).toBe('Hello World');
  });
});
