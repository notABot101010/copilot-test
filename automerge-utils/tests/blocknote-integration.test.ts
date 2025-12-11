import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
  compareDocumentSizes,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

// Simple mock block structure
interface MockBlock {
  id: string;
  type: string;
  content: Array<{ type: string; text: string }>;
  children: MockBlock[];
}

describe('Integration: Minimal Document Growth', () => {
  it('should keep document size minimal with sequential block additions', () => {
    let doc = createBlockNoteDocument();

    const initialSize = Automerge.save(doc).byteLength;

    // Simulate adding blocks one by one
    const blockIds = ['1', '2', '3', '4', '5'];
    for (const id of blockIds) {
      const changes: BlockNoteChanges = [
        {
          type: 'insert',
          block: {
            id,
            type: 'paragraph',
            content: [{ type: 'text', text: `Block ${id}` }],
            children: [],
          } as any,
          source: { type: 'local' },
          prevBlock: undefined,
        },
      ];

      doc = applyBlockNoteChanges(doc, changes);
    }

    expect(doc.blocks).toHaveLength(5);

    const finalSize = Automerge.save(doc).byteLength;
    const growth = finalSize - initialSize;

    // Document should grow, but not excessively
    expect(growth).toBeGreaterThan(0);
    expect(growth).toBeLessThan(2000); // Reasonable threshold for 5 blocks
  });

  it('should handle batch block insertions efficiently', () => {
    const emptyDoc = createBlockNoteDocument();
    const emptySize = Automerge.save(emptyDoc).byteLength;

    // Apply multiple blocks at once (like pasting)
    const changes: BlockNoteChanges = [
      {
        type: 'insert',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'First paragraph' }],
          children: [],
        } as any,
        source: { type: 'paste' },
        prevBlock: undefined,
      },
      {
        type: 'insert',
        block: {
          id: '2',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Second paragraph' }],
          children: [],
        } as any,
        source: { type: 'paste' },
        prevBlock: undefined,
      },
      {
        type: 'insert',
        block: {
          id: '3',
          type: 'heading',
          content: [{ type: 'text', text: 'A heading' }],
          children: [],
        } as any,
        source: { type: 'paste' },
        prevBlock: undefined,
      },
    ];

    const newDoc = applyBlockNoteChanges(emptyDoc, changes);
    expect(newDoc.blocks).toHaveLength(3);

    const newSize = Automerge.save(newDoc).byteLength;
    const growth = newSize - emptySize;

    // Batch changes should be efficient
    expect(growth).toBeGreaterThan(0);
    expect(growth).toBeLessThan(1500);
  });

  it('should handle block updates efficiently', () => {
    const originalBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Original text' }],
      children: [],
    } as any;

    const doc = createBlockNoteDocument([originalBlock]);
    const initialSize = Automerge.save(doc).byteLength;

    // Update the block content
    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Updated text with more content' }],
          children: [],
        } as any,
        prevBlock: originalBlock,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks[0].content[0].text).toBe('Updated text with more content');

    const finalSize = Automerge.save(newDoc).byteLength;

    // Size might grow slightly due to operation metadata
    expect(finalSize - initialSize).toBeLessThan(500);
  });

  it('should handle block deletions efficiently', () => {
    const blocks = [
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'First' }],
        children: [],
      } as any,
      {
        id: '2',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Second' }],
        children: [],
      } as any,
      {
        id: '3',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Third' }],
        children: [],
      } as any,
    ];

    const doc = createBlockNoteDocument(blocks);
    const initialSize = Automerge.save(doc).byteLength;

    // Delete middle block
    const changes: BlockNoteChanges = [
      {
        type: 'delete',
        block: blocks[1],
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);
    expect(newDoc.blocks).toHaveLength(2);
    expect(newDoc.blocks.some((b) => b.id === '2')).toBe(false);

    const finalSize = Automerge.save(newDoc).byteLength;

    // Deletion adds metadata but removes content
    expect(finalSize - initialSize).toBeLessThan(300);
  });

  it('should compare with naive approach', () => {
    // Naive approach: replace entire blocks array
    let naiveDoc = createBlockNoteDocument();

    const blocks = [
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello' }],
        children: [],
      } as any,
    ];

    naiveDoc = Automerge.change(naiveDoc, (d) => {
      d.blocks = blocks;
    });

    const naiveSize = Automerge.save(naiveDoc).byteLength;

    // Optimized approach: use applyBlockNoteChanges
    let optimizedDoc = createBlockNoteDocument();

    const changes: BlockNoteChanges = [
      {
        type: 'insert',
        block: blocks[0],
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ];

    optimizedDoc = applyBlockNoteChanges(optimizedDoc, changes);

    const optimizedSize = Automerge.save(optimizedDoc).byteLength;

    // Both should produce same result
    expect(optimizedDoc.blocks).toHaveLength(1);
    expect(naiveDoc.blocks).toHaveLength(1);

    // Sizes should be similar for simple case
    expect(Math.abs(optimizedSize - naiveSize)).toBeLessThan(100);
  });

  it('should demonstrate efficiency with mixed operations', () => {
    let doc = createBlockNoteDocument();

    // Insert initial blocks
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: { id: '1', type: 'paragraph', content: [{ type: 'text', text: 'First' }], children: [] } as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
      {
        type: 'insert',
        block: { id: '2', type: 'paragraph', content: [{ type: 'text', text: 'Second' }], children: [] } as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ]);

    const afterInsertSize = Automerge.save(doc).byteLength;

    // Update first block
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: { id: '1', type: 'paragraph', content: [{ type: 'text', text: 'First (updated)' }], children: [] } as any,
        prevBlock: { id: '1', type: 'paragraph', content: [{ type: 'text', text: 'First' }], children: [] } as any,
        source: { type: 'local' },
      },
    ]);

    // Delete second block
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'delete',
        block: { id: '2', type: 'paragraph', content: [{ type: 'text', text: 'Second' }], children: [] } as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ]);

    // Insert new block
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: { id: '3', type: 'heading', content: [{ type: 'text', text: 'New heading' }], children: [] } as any,
        source: { type: 'local' },
        prevBlock: undefined,
      },
    ]);

    expect(doc.blocks).toHaveLength(2);

    const finalSize = Automerge.save(doc).byteLength;
    const totalGrowth = finalSize - afterInsertSize;

    // Mixed operations should be reasonably efficient
    expect(totalGrowth).toBeLessThan(800);
  });
});
