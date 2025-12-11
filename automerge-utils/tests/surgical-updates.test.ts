import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
  compareDocumentSizes,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

// Mock block structure
interface MockBlock {
  id: string;
  type: string;
  content: Array<{ type: string; text: string; styles?: Record<string, any> }>;
  children: MockBlock[];
}

describe('Surgical Updates', () => {
  it('should surgically update only the text field when text changes', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'helo wo', styles: {} }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);
    const beforeSize = Automerge.save(doc).byteLength;

    // Update just the text content
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

    // Verify the text was updated
    expect(newDoc.blocks[0].content[0].text).toBe('helo world');

    // The growth should be minimal since we only updated the text field
    const growth = afterSize - beforeSize;
    console.log(`Text update growth: ${growth} bytes`);
    expect(growth).toBeLessThan(200); // Should be much smaller than full block replacement
  });

  it('should handle multiple text edits efficiently', () => {
    let doc = createBlockNoteDocument([
      {
        id: '1',
        type: 'paragraph',
        content: [{ type: 'text', text: 'Hello', styles: {} }],
        children: [],
      } as any,
    ]);

    const initialSize = Automerge.save(doc).byteLength;

    // Simulate typing character by character
    const texts = ['Hello ', 'Hello w', 'Hello wo', 'Hello wor', 'Hello worl', 'Hello world'];
    
    for (let i = 0; i < texts.length; i++) {
      const prevText = i === 0 ? 'Hello' : texts[i - 1];
      
      const changes: BlockNoteChanges = [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: texts[i], styles: {} }],
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
    }

    const finalSize = Automerge.save(doc).byteLength;
    const totalGrowth = finalSize - initialSize;

    expect(doc.blocks[0].content[0].text).toBe('Hello world');
    console.log(`Total growth for 6 text edits: ${totalGrowth} bytes`);
    
    // With surgical updates, growth should be reasonable
    expect(totalGrowth).toBeLessThan(800);
  });

  it('should update only changed fields, not the entire block', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Original', styles: { bold: true } }],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Change only the text, keeping styles the same
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Updated', styles: { bold: true } }],
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

    expect(newDoc.blocks[0].content[0].text).toBe('Updated');
    expect(newDoc.blocks[0].content[0].styles).toEqual({ bold: true });
  });

  it('should handle content array changes efficiently', () => {
    const originalBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [
        { type: 'text', text: 'Hello ', styles: {} },
        { type: 'text', text: 'world', styles: { bold: true } },
      ],
      children: [],
    };

    const doc = createBlockNoteDocument([originalBlock as any]);

    // Update just the first text item
    const updatedBlock: MockBlock = {
      id: '1',
      type: 'paragraph',
      content: [
        { type: 'text', text: 'Hi ', styles: {} },
        { type: 'text', text: 'world', styles: { bold: true } },
      ],
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

    expect(newDoc.blocks[0].content[0].text).toBe('Hi ');
    expect(newDoc.blocks[0].content[1].text).toBe('world');
    expect(newDoc.blocks[0].content[1].styles).toEqual({ bold: true });
  });

  it('should handle field additions and removals', () => {
    const originalBlock: any = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello', styles: {} }],
      children: [],
      props: { backgroundColor: 'blue' },
    };

    const doc = createBlockNoteDocument([originalBlock]);

    // Update with a new field and remove old field
    const updatedBlock: any = {
      id: '1',
      type: 'paragraph',
      content: [{ type: 'text', text: 'Hello', styles: {} }],
      children: [],
      props: { color: 'red' }, // Changed field
    };

    const changes: BlockNoteChanges = [
      {
        type: 'update',
        block: updatedBlock,
        prevBlock: originalBlock,
        source: { type: 'local' },
      },
    ];

    const newDoc = applyBlockNoteChanges(doc, changes);

    expect((newDoc.blocks[0] as any).props).toEqual({ color: 'red' });
  });
});
