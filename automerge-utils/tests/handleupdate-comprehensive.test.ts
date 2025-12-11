import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('handleUpdate comprehensive tests', () => {
  describe('Missing block scenarios', () => {
    it('should treat update of non-existent block as insert', () => {
      let doc = createBlockNoteDocument();

      // Try to update a block that doesn't exist
      const changes: BlockNoteChanges = [
        {
          type: 'update',
          block: {
            id: 'non-existent-id',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello World' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'non-existent-id',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ];

      doc = applyBlockNoteChanges(doc, changes);

      // The block should be inserted
      expect(doc.blocks).toHaveLength(1);
      expect(doc.blocks[0].id).toBe('non-existent-id');
      expect(doc.blocks[0].content[0].text).toBe('Hello World');
    });

    it('should handle BlockNote pattern: insert empty + update different ID', () => {
      // This mimics what BlockNote actually does when you start typing
      let doc = createBlockNoteDocument();

      const changes: BlockNoteChanges = [
        // BlockNote inserts a new empty block
        {
          type: 'insert',
          block: {
            id: 'new-block-id',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        // BlockNote updates the old placeholder block (different ID)
        {
          type: 'update',
          block: {
            id: 'old-placeholder-id',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Typed text' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'old-placeholder-id',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ];

      doc = applyBlockNoteChanges(doc, changes);

      // Should have 2 blocks: the new empty one and the updated placeholder
      expect(doc.blocks).toHaveLength(2);
      
      // Find the block with text
      const blockWithText = doc.blocks.find((b) => (b as any).content.length > 0);
      expect(blockWithText).toBeDefined();
      expect(blockWithText!.id).toBe('old-placeholder-id');
      expect((blockWithText as any).content[0].text).toBe('Typed text');
    });

    it('should persist text after save/load with BlockNote pattern', () => {
      let doc = createBlockNoteDocument();

      // Simulate BlockNote's insert + update pattern
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'block-0',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-0',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and load
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Text should persist
      const blockWithText = loaded.blocks.find((b: any) => b.content.length > 0);
      expect(blockWithText).toBeDefined();
      expect((blockWithText as any).content[0].text).toBe('Hello');
    });
  });

  describe('Normal update scenarios', () => {
    it('should handle normal update when block exists', () => {
      let doc = createBlockNoteDocument();

      // Insert a block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Original' }],
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
            content: [{ type: 'text', text: 'Updated' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Original' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks).toHaveLength(1);
      expect(doc.blocks[0].id).toBe('1');
      expect(doc.blocks[0].content[0].text).toBe('Updated');
    });

    it('should handle surgical text updates', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello' }],
          children: [],
        } as any,
      ]);

      // Append text
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

      expect(doc.blocks[0].content[0].text).toBe('Hello World');
    });

    it('should handle multiple sequential updates', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: '' }],
          children: [],
        } as any,
      ]);

      const texts = ['H', 'He', 'Hel', 'Hell', 'Hello'];
      let prevText = '';

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

      expect(doc.blocks[0].content[0].text).toBe('Hello');
    });
  });

  describe('Edge cases', () => {
    it('should handle update without prevBlock', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Old' }],
          children: [],
        } as any,
      ]);

      // Update without prevBlock - should replace entire block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'New' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('New');
    });

    it('should handle update that adds new fields', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [],
          children: [],
        } as any,
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [],
            children: [],
            props: { color: 'red' },
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect((doc.blocks[0] as any).props).toEqual({ color: 'red' });
    });

    it('should handle update that removes fields', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [],
          children: [],
          props: { color: 'red' },
        } as any,
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [],
            children: [],
            props: { color: 'red' },
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect((doc.blocks[0] as any).props).toBeUndefined();
    });

    it('should handle empty content array updates', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Text' }],
          children: [],
        } as any,
      ]);

      // Clear content
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Text' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content).toHaveLength(0);
    });

    it('should handle content array with multiple items', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [
            { type: 'text', text: 'Hello ', styles: {} },
            { type: 'text', text: 'World', styles: { bold: true } },
          ],
          children: [],
        } as any,
      ]);

      // Update first item only
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [
              { type: 'text', text: 'Hi ', styles: {} },
              { type: 'text', text: 'World', styles: { bold: true } },
            ],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [
              { type: 'text', text: 'Hello ', styles: {} },
              { type: 'text', text: 'World', styles: { bold: true } },
            ],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hi ');
      expect(doc.blocks[0].content[1].text).toBe('World');
      expect((doc.blocks[0].content[1] as any).styles).toEqual({ bold: true });
    });
  });

  describe('Real-world scenarios', () => {
    it('should handle complete typing session with persistence', () => {
      let doc = createBlockNoteDocument();

      // Initial state from BlockNote
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'new-block',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'placeholder',
            type: 'paragraph',
            content: [{ type: 'text', text: 'H' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'placeholder',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Continue typing
      let prevText = 'H';
      const typing = ['He', 'Hel', 'Hell', 'Hello'];
      
      for (const text of typing) {
        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'placeholder',
              type: 'paragraph',
              content: [{ type: 'text', text }],
              children: [],
            } as any,
            prevBlock: {
              id: 'placeholder',
              type: 'paragraph',
              content: [{ type: 'text', text: prevText }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
        prevText = text;
      }

      // Save and reload
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Find block with text
      const blockWithText = loaded.blocks.find((b: any) => b.content.length > 0);
      expect(blockWithText).toBeDefined();
      expect((blockWithText as any).content[0].text).toBe('Hello');
    });

    it('should handle multiple blocks with mixed operations', () => {
      let doc = createBlockNoteDocument();

      // Add multiple blocks
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'insert',
          block: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Update both
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First Updated' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second Updated' }],
            children: [],
          } as any,
          prevBlock: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks).toHaveLength(2);
      expect(doc.blocks[0].content[0].text).toBe('First Updated');
      expect(doc.blocks[1].content[0].text).toBe('Second Updated');

      // Persist
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      expect(loaded.blocks).toHaveLength(2);
      expect((loaded.blocks[0] as any).content[0].text).toBe('First Updated');
      expect((loaded.blocks[1] as any).content[0].text).toBe('Second Updated');
    });
  });
});
