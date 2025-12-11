import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('End-to-End Integration Tests', () => {
  describe('Complete user workflows', () => {
    it('should handle complete typing session with save/load', () => {
      // Simulate a complete user session
      let doc = createBlockNoteDocument();

      // User starts typing
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
            id: 'placeholder-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'H' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'placeholder-1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Continue typing
      const texts = ['He', 'Hel', 'Hell', 'Hello', 'Hello ', 'Hello W', 'Hello Wo', 'Hello Wor', 'Hello Worl', 'Hello World'];
      let prevText = 'H';

      for (const text of texts) {
        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'placeholder-1',
              type: 'paragraph',
              content: [{ type: 'text', text }],
              children: [],
            } as any,
            prevBlock: {
              id: 'placeholder-1',
              type: 'paragraph',
              content: [{ type: 'text', text: prevText }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
        prevText = text;
      }

      // Save document
      const saved = Automerge.save(doc);
      
      // Simulate page reload
      const loaded = Automerge.load(saved);

      // Verify text persists
      const blockWithText = loaded.blocks.find((b: any) => b.content.length > 0);
      expect(blockWithText).toBeDefined();
      expect((blockWithText as any).content[0].text).toBe('Hello World');
    });

    it('should handle multiple paragraphs with persistence', () => {
      let doc = createBlockNoteDocument();

      // Add first paragraph
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'new-1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'para-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First paragraph' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'para-1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Add second paragraph
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'new-2',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'para-2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second paragraph' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'para-2',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and reload
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Find both paragraphs
      const blocks = loaded.blocks.filter((b: any) => b.content.length > 0);
      expect(blocks.length).toBe(2);
      
      const texts = blocks.map((b: any) => b.content[0].text);
      expect(texts).toContain('First paragraph');
      expect(texts).toContain('Second paragraph');
    });

    it('should handle editing existing text after reload', () => {
      let doc = createBlockNoteDocument();

      // Initial text
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'para',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial text' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and reload
      const saved = Automerge.save(doc);
      let loaded = Automerge.load(saved);

      // Edit after reload
      loaded = applyBlockNoteChanges(loaded, [
        {
          type: 'update',
          block: {
            id: 'para',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial text edited' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'para',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial text' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save again and reload
      const saved2 = Automerge.save(loaded);
      const loaded2 = Automerge.load(saved2);

      // Verify edit persisted
      const block = loaded2.blocks.find((b: any) => b.id === 'para');
      expect(block).toBeDefined();
      expect((block as any).content[0].text).toBe('Initial text edited');
    });

    it('should handle mixed operations with persistence', () => {
      let doc = createBlockNoteDocument();

      // Add multiple blocks
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 1' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'insert',
          block: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 2' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'insert',
          block: {
            id: '3',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 3' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Update middle block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 2 Updated' }],
            children: [],
          } as any,
          prevBlock: {
            id: '2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 2' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Delete first block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'delete',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Block 1' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and reload
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Verify final state
      expect(loaded.blocks.length).toBe(2);
      expect(loaded.blocks.some((b: any) => b.id === '1')).toBe(false);
      
      const block2 = loaded.blocks.find((b: any) => b.id === '2');
      expect(block2).toBeDefined();
      expect((block2 as any).content[0].text).toBe('Block 2 Updated');

      const block3 = loaded.blocks.find((b: any) => b.id === '3');
      expect(block3).toBeDefined();
      expect((block3 as any).content[0].text).toBe('Block 3');
    });
  });

  describe('BlockNote real-world patterns', () => {
    it('should handle the exact pattern BlockNote uses when typing', () => {
      // This test replicates the exact sequence of changes BlockNote sends
      let doc = createBlockNoteDocument();

      // User clicks in editor and starts typing "Test"
      // BlockNote sends: INSERT empty block + UPDATE old placeholder with text
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'new-cursor-block',
            type: 'paragraph',
            content: [],
            children: [],
            props: {
              backgroundColor: 'default',
              textColor: 'default',
              textAlignment: 'left',
            },
          } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'old-placeholder-block',
            type: 'paragraph',
            content: [
              {
                type: 'text',
                text: 'Test',
                styles: {},
              },
            ],
            children: [],
            props: {
              backgroundColor: 'default',
              textColor: 'default',
              textAlignment: 'left',
            },
          } as any,
          prevBlock: {
            id: 'old-placeholder-block',
            type: 'paragraph',
            content: [],
            children: [],
            props: {
              backgroundColor: 'default',
              textColor: 'default',
              textAlignment: 'left',
            },
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Verify we have 2 blocks and text is saved
      expect(doc.blocks.length).toBe(2);
      const blockWithText = doc.blocks.find((b: any) => b.content.length > 0);
      expect(blockWithText).toBeDefined();
      expect(blockWithText!.id).toBe('old-placeholder-block');
      expect((blockWithText as any).content[0].text).toBe('Test');

      // Save and reload - this is where the bug was
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Text MUST persist after reload
      const loadedBlockWithText = loaded.blocks.find((b: any) => b.content.length > 0);
      expect(loadedBlockWithText).toBeDefined();
      expect((loadedBlockWithText as any).content[0].text).toBe('Test');
    });

    it('should handle multiple typing sessions with reloads', () => {
      let doc = createBlockNoteDocument();

      // Session 1: Type "Hello"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: { id: 'new-1', type: 'paragraph', content: [], children: [] } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'para-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'para-1',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and reload (simulate closing browser)
      let saved = Automerge.save(doc);
      doc = Automerge.load(saved);

      // Session 2: Type "World" in a new paragraph
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: { id: 'new-2', type: 'paragraph', content: [], children: [] } as any,
          source: { type: 'local' },
        },
        {
          type: 'update',
          block: {
            id: 'para-2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'World' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'para-2',
            type: 'paragraph',
            content: [],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and reload again
      saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      // Both texts should persist
      const textsFound = loaded.blocks
        .filter((b: any) => b.content.length > 0)
        .map((b: any) => b.content[0].text);

      expect(textsFound).toContain('Hello');
      expect(textsFound).toContain('World');
    });
  });

  describe('Document size optimization', () => {
    it('should maintain reasonable document size with many operations', () => {
      let doc = createBlockNoteDocument();
      const initialSize = Automerge.save(doc).byteLength;

      // Add 10 blocks with text
      for (let i = 0; i < 10; i++) {
        doc = applyBlockNoteChanges(doc, [
          {
            type: 'insert',
            block: {
              id: `new-${i}`,
              type: 'paragraph',
              content: [],
              children: [],
            } as any,
            source: { type: 'local' },
          },
          {
            type: 'update',
            block: {
              id: `para-${i}`,
              type: 'paragraph',
              content: [{ type: 'text', text: `Paragraph ${i}` }],
              children: [],
            } as any,
            prevBlock: {
              id: `para-${i}`,
              type: 'paragraph',
              content: [],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
      }

      const finalSize = Automerge.save(doc).byteLength;
      const growth = finalSize - initialSize;

      // Should be reasonable (not exponential)
      expect(growth).toBeLessThan(5000);

      // Verify content persists
      const saved = Automerge.save(doc);
      const loaded = Automerge.load(saved);

      const textsWithContent = loaded.blocks.filter((b: any) => b.content.length > 0);
      expect(textsWithContent.length).toBe(10);
    });
  });
});
