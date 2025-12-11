import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
  getDocumentSize,
  compareDocumentSizes,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

// Simple mock block structure for testing
interface MockBlock {
  id: string;
  type: string;
  content: Array<{ type: string; text: string }>;
  children: MockBlock[];
}

describe('createBlockNoteDocument', () => {
  it('should create an empty document', () => {
    const doc = createBlockNoteDocument();
    expect(doc.blocks).toEqual([]);
  });

  it('should create a document with initial blocks', () => {
    const blocks: MockBlock[] = [
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello' }],
        children: [],
      },
    ];
    const doc = createBlockNoteDocument(blocks as any);
    expect(doc.blocks).toHaveLength(1);
    expect(doc.blocks[0].id).toBe('1');
  });
});

describe('applyBlockNoteChanges', () => {
  it('should handle empty changes', () => {
    const doc = createBlockNoteDocument();
    const changes: BlockNoteChanges = [];
    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks).toEqual([]);
  });

  it('should handle block insertion', () => {
    const doc = createBlockNoteDocument();
    const newBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'insert',
        block: newBlock as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks).toHaveLength(1);
    expect(newDoc.blocks[0].id).toBe('1');
  });

  it('should handle block deletion', () => {
    const block: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    const doc = createBlockNoteDocument([block as any]);

    const changes: BlockNoteChanges = [
      {
        type: 'delete',
        block: block as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks).toHaveLength(0);
  });

  it('should handle block update', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello World' }],
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
    expect(newDoc.blocks).toHaveLength(1);
    expect(newDoc.blocks[0].content[0].text).toBe('Hello World');
  });

  it('should handle multiple block insertions', () => {
    const doc = createBlockNoteDocument();

    const block1: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'First' }],
      children: [],
    };

    const block2: MockBlock = {
      id: '2',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Second' }],
      children: [],
    };

    const changes: BlockNoteChanges = [
      {
        type: 'insert',
        block: block1 as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
      {
        type: 'insert',
        block: block2 as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks).toHaveLength(2);
  });
});

describe('optimization', () => {
  it('should filter out no-op updates', () => {
    const block: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    const doc = createBlockNoteDocument([block as any]);

    // Update with the same content (no-op)
    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: block as any,
        prevBlock: block as any,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes, { optimize: true });
    
    // Document should be unchanged
    const comparison = compareDocumentSizes(doc, newDoc);
    expect(comparison.growth).toBe(0);
  });
});

describe('getDocumentSize', () => {
  it('should return size in bytes', () => {
    const block: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello' }],
      children: [],
    };

    const doc = createBlockNoteDocument([block as any]);
    const size = getDocumentSize(doc);
    expect(size).toBeGreaterThan(0);
  });

  it('should show larger documents have larger size', () => {
    const smallDoc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hi' }],
        children: [],
      } as any,
    ]);

    const largeDoc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello World, this is a much longer text with more content' }],
        children: [],
      } as any,
      {
        id: '2',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Another paragraph with more text' }],
        children: [],
      } as any,
    ]);

    const smallSize = getDocumentSize(smallDoc);
    const largeSize = getDocumentSize(largeDoc);

    expect(largeSize).toBeGreaterThan(smallSize);
  });
});

describe('compareDocumentSizes', () => {
  it('should calculate growth correctly', () => {
    const oldDoc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello' }],
        children: [],
      } as any,
    ]);

    const newDoc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello' }],
        children: [],
      } as any,
      {
        id: '2',
        type: 'paragraph',
        content: [{ type: 'text', text: 'World' }],
        children: [],
      } as any,
    ]);

    const comparison = compareDocumentSizes(oldDoc, newDoc);

    expect(comparison.oldSize).toBeGreaterThan(0);
    expect(comparison.newSize).toBeGreaterThan(comparison.oldSize);
    expect(comparison.growth).toBe(comparison.newSize - comparison.oldSize);
    expect(comparison.growthPercent).toBeGreaterThan(0);
  });
});
