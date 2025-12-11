import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

// Mock block structure
interface MockBlock {
  id: string;
  type: string;
  content: Array<{ type: string; text: string; styles?: Record<string, any> }>;
  children: MockBlock[];
}

describe('String Splice Optimization', () => {
  it('should use minimal string operations for text insertion at end', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'helo wo', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);
    const beforeSize = Automerge.save(doc).byteLength;

    // Add characters at the end: "helo wo" -> "helo world"
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'helo world', styles: {} }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    const afterSize = Automerge.save(newDoc).byteLength;

    expect(newDoc.blocks[0].content[0].text).toBe('helo world');
    
    const growth = afterSize - beforeSize;
    console.log(`Appending "rld" growth: ${growth} bytes`);
    // Should be minimal since only "rld" was added
    expect(growth).toBeLessThan(100);
  });

  it('should use minimal string operations for text insertion in middle', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'helo world', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Insert 'l' in middle: "helo world" -> "hello world"
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'hello world', styles: {} }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);

    expect(newDoc.blocks[0].content[0].text).toBe('hello world');
  });

  it('should use minimal string operations for text deletion', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'hello world', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Delete " world": "hello world" -> "hello"
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'hello', styles: {} }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);

    expect(newDoc.blocks[0].content[0].text).toBe('hello');
  });

  it('should handle replacement in middle of string', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'hello world', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Replace "world" with "universe": "hello world" -> "hello universe"
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'hello universe', styles: {} }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);

    expect(newDoc.blocks[0].content[0].text).toBe('hello universe');
  });

  it('should efficiently handle incremental typing', () => {
    let doc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: '', styles: {} }],
        children: [],
      } as any,
    ]);

    const initialSize = Automerge.save(doc).byteLength;

    // Simulate typing "hello" character by character
    const typingSequence = ['h', 'he', 'hel', 'hell', 'hello'];
    let prevText = '';

    for (const currentText of typingSequence) {
      const changes: BlockNoteChanges = [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: currentText, styles: {} }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: prevText, styles: {} }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ];

      doc = applyBlockNoteChanges(doc, changes);
      prevText = currentText;
    }

    const finalSize = Automerge.save(doc).byteLength;
    const totalGrowth = finalSize - initialSize;

    expect(doc.blocks[0].content[0].text).toBe('hello');
    console.log(`Typing 5 characters incrementally: ${totalGrowth} bytes`);
    
    // Should be very efficient with splice operations
    expect(totalGrowth).toBeLessThan(300);
  });

  it('should handle complex edits with prefix and suffix preservation', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'The quick brown fox', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Change "brown" to "red": "The quick brown fox" -> "The quick red fox"
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'The quick red fox', styles: {} }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock as any,
        prevBlock: originalBlock as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);

    expect(newDoc.blocks[0].content[0].text).toBe('The quick red fox');
  });
});
