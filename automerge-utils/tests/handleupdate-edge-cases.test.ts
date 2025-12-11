import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

describe('handleUpdate Edge Cases', () => {
  describe('Empty string handling', () => {
    it('should handle empty to non-empty text', () => {
      let doc = createBlockNoteDocument();

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: '' }],
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
            content: [{ type: 'text', text: 'Hello' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: '' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello');
    });

    it('should handle non-empty to empty text', () => {
      let doc = createBlockNoteDocument();

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
            content: [{ type: 'text', text: '' }],
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

      expect(doc.blocks[0].content[0].text).toBe('');
    });
  });

  describe('Unicode and special characters', () => {
    it('should handle emoji correctly', () => {
      let doc = createBlockNoteDocument();

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
            content: [{ type: 'text', text: 'Hello 👋🌍' }],
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

      expect(doc.blocks[0].content[0].text).toBe('Hello 👋🌍');
    });

    it('should handle multi-byte unicode characters', () => {
      let doc = createBlockNoteDocument();

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: '你好' }],
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
            content: [{ type: 'text', text: '你好世界' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: '你好' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('你好世界');
    });
  });

  describe('Multiple consecutive updates', () => {
    it('should handle rapid sequential typing', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: '' }],
          children: [],
        } as any,
      ]);

      const sequence = ['H', 'He', 'Hel', 'Hell', 'Hello'];
      let prevText = '';

      for (const text of sequence) {
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

    it('should handle typing and backspacing', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: '' }],
          children: [],
        } as any,
      ]);

      const sequence = ['T', 'Te', 'Tes', 'Test', 'Tes', 'Te', 'Tex', 'Text'];
      let prevText = '';

      for (const text of sequence) {
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

      expect(doc.blocks[0].content[0].text).toBe('Text');
    });
  });

  describe('Save/load cycles with updates', () => {
    it('should handle update after save/load', () => {
      let doc = createBlockNoteDocument();

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save and load
      const saved = Automerge.save(doc);
      doc = Automerge.load(saved);

      // Update after reload
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial text' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Initial' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Initial text');

      // Another save/load cycle
      const saved2 = Automerge.save(doc);
      doc = Automerge.load(saved2);

      expect(doc.blocks[0].content[0].text).toBe('Initial text');
    });

    it('should handle multiple updates across multiple save/load cycles', () => {
      let doc = createBlockNoteDocument();

      // Insert
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

      // Save/load 1
      doc = Automerge.load(Automerge.save(doc));
      
      // Update 1
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

      // Save/load 2
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('AB');

      // Update 2
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABC' }],
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

      // Save/load 3
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('ABC');

      // Update 3
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABCD' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABC' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Final save/load
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('ABCD');
    });
  });

  describe('Complex text edits', () => {
    it('should handle insertion in the middle', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Helo World' }],
          children: [],
        } as any,
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
            content: [{ type: 'text', text: 'Helo World' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello World');
    });

    it('should handle deletion from the middle', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Helllo World' }],
          children: [],
        } as any,
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
            content: [{ type: 'text', text: 'Helllo World' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello World');
    });

    it('should handle replacement in the middle', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello World' }],
          children: [],
        } as any,
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello Universe' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello World' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello Universe');
    });
  });

  describe('State mismatch scenarios', () => {
    it('should handle mismatch when current document has more text', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'ABC' }],
          children: [],
        } as any,
      ]);

      // prevBlock says "AB", but document actually has "ABC"
      // User wants to change to "ABD"
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

      // Should result in "ABD", not "ABDC"
      expect(doc.blocks[0].content[0].text).toBe('ABD');
    });

    it('should handle mismatch when current document has less text', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'A' }],
          children: [],
        } as any,
      ]);

      // prevBlock says "ABC", but document actually has "A"
      // User wants to change to "ABCD"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABCD' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABC' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Should result in "ABCD"
      expect(doc.blocks[0].content[0].text).toBe('ABCD');
    });

    it('should handle mismatch when current document has completely different text', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'XYZ' }],
          children: [],
        } as any,
      ]);

      // prevBlock says "ABC", but document actually has "XYZ"
      // User wants to change to "ABCD"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABCD' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'ABC' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Should result in "ABCD"
      expect(doc.blocks[0].content[0].text).toBe('ABCD');
    });
  });

  describe('Large text updates', () => {
    it('should handle large text insertion', () => {
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Start' }],
          children: [],
        } as any,
      ]);

      const largeText = 'Start' + ' '.repeat(100) + 'End with a lot of text in between';

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: largeText }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Start' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe(largeText);
    });

    it('should handle large text deletion', () => {
      const largeText = 'Start' + ' '.repeat(100) + 'End with a lot of text in between';
      
      let doc = createBlockNoteDocument([
        {
          id: '1',
          type: 'paragraph',
          content: [{ type: 'text', text: largeText }],
          children: [],
        } as any,
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Start' }],
            children: [],
          } as any,
          prevBlock: {
            id: '1',
            type: 'paragraph',
            content: [{ type: 'text', text: largeText }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Start');
    });
  });
});
