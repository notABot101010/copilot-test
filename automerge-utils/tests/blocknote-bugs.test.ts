import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';

describe('BlockNote Integration Bugs', () => {
  it('should produce blocks that can be serialized to JSON', () => {
    let doc = createBlockNoteDocument();

    // Insert a block
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

    // Update it
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

    // Try to use blocks as BlockNote would
    const blocks = doc.blocks;
    console.log('Blocks array:', blocks);
    console.log('Blocks length:', blocks.length);
    console.log('Block 0:', blocks[0]);
    console.log('Block 0 type:', blocks[0].type);
    console.log('Block 0 content:', blocks[0].content);
    console.log('Block 0 content[0]:', blocks[0].content[0]);
    console.log('Block 0 content[0].text:', blocks[0].content[0].text);

    // Try to JSON serialize
    try {
      const json = JSON.stringify(blocks);
      console.log('JSON serialized successfully, length:', json.length);
      const parsed = JSON.parse(json);
      console.log('Parsed blocks:', parsed);
      console.log('Parsed text:', parsed[0].content[0].text);
      expect(parsed[0].content[0].text).toBe('Hello World');
    } catch (err) {
      console.error('JSON serialization failed:', err);
      throw err;
    }
  });

  it('should work when blocks are passed through spread operator', () => {
    let doc = createBlockNoteDocument();

    doc = applyBlockNoteChanges(doc, [
      {
        type: 'insert',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Test' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    // Update with surgical splice
    doc = applyBlockNoteChanges(doc, [
      {
        type: 'update',
        block: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Test 123' }],
          children: [],
        } as any,
        prevBlock: {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Test' }],
          children: [],
        } as any,
        source: { type: 'local' },
      },
    ]);

    // Simulate how BlockNote might use it: spread into array or pass directly
    const blocksArray = [...doc.blocks];
    console.log('Spread blocks:', blocksArray);
    console.log('Spread text:', blocksArray[0].content[0].text);
    expect(blocksArray[0].content[0].text).toBe('Test 123');

    // Try mapping
    const mapped = doc.blocks.map((b: any) => b);
    console.log('Mapped blocks:', mapped);
    console.log('Mapped text:', mapped[0].content[0].text);
    expect(mapped[0].content[0].text).toBe('Test 123');
  });
});
