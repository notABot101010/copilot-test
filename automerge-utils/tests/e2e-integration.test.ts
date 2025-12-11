import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
} from '../src/transformer';
import type { BlockNoteChanges } from '../src/types';

/**
 * End-to-end integration tests that simulate real-world usage patterns
 * like those in the blocknote-collab app
 */
describe('End-to-End Integration Tests', () => {
  describe('User session with save/load', () => {
    it('should handle complete user workflow: type -> save -> reload -> type more', () => {
      // Session 1: User types initial text
      let doc = createBlockNoteDocument();

      // User types "The quick brown"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'The quick brown' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('The quick brown');

      // Simulate saving to localStorage (like in blocknote-collab)
      const savedData = Automerge.save(doc);

      // Session 2: Page reload
      doc = Automerge.load(savedData);
      expect(doc.blocks[0].content[0].text).toBe('The quick brown');

      // User continues typing: " fox"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'The quick brown fox' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'The quick brown' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('The quick brown fox');

      // Save again
      const savedData2 = Automerge.save(doc);

      // Session 3: Another reload
      doc = Automerge.load(savedData2);
      expect(doc.blocks[0].content[0].text).toBe('The quick brown fox');

      // User types more: " jumps over"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'The quick brown fox jumps over' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'The quick brown fox' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('The quick brown fox jumps over');

      // Final verification
      const savedData3 = Automerge.save(doc);
      doc = Automerge.load(savedData3);
      expect(doc.blocks[0].content[0].text).toBe('The quick brown fox jumps over');
    });

    it('should handle multiple blocks with save/load cycles', () => {
      let doc = createBlockNoteDocument();

      // User creates first block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First paragraph' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // User creates second block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second paragraph' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks.length).toBe(2);
      expect(doc.blocks[0].content[0].text).toBe('First paragraph');
      expect(doc.blocks[1].content[0].text).toBe('Second paragraph');

      // Save and reload
      doc = Automerge.load(Automerge.save(doc));

      expect(doc.blocks.length).toBe(2);
      expect(doc.blocks[0].content[0].text).toBe('First paragraph');
      expect(doc.blocks[1].content[0].text).toBe('Second paragraph');

      // User edits first block after reload
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First paragraph edited' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'First paragraph' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('First paragraph edited');
      expect(doc.blocks[1].content[0].text).toBe('Second paragraph');

      // User edits second block
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second paragraph also edited' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-2',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Second paragraph' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Final verification
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks.length).toBe(2);
      expect(doc.blocks[0].content[0].text).toBe('First paragraph edited');
      expect(doc.blocks[1].content[0].text).toBe('Second paragraph also edited');
    });
  });

  describe('Real-world typing patterns', () => {
    it('should handle user typing with pauses and corrections', () => {
      let doc = createBlockNoteDocument();

      // User types initial text
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: '' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Typing "I am writng"
      const typingSteps = [
        { text: 'I', prev: '' },
        { text: 'I ', prev: 'I' },
        { text: 'I a', prev: 'I ' },
        { text: 'I am', prev: 'I a' },
        { text: 'I am ', prev: 'I am' },
        { text: 'I am w', prev: 'I am ' },
        { text: 'I am wr', prev: 'I am w' },
        { text: 'I am wri', prev: 'I am wr' },
        { text: 'I am writ', prev: 'I am wri' },
        { text: 'I am writn', prev: 'I am writ' },
        { text: 'I am writng', prev: 'I am writn' },
      ];

      for (const step of typingSteps) {
        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: step.text }],
              children: [],
            } as any,
            prevBlock: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: step.prev }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
      }

      expect(doc.blocks[0].content[0].text).toBe('I am writng');

      // User notices typo, corrects it
      // Backspace twice: "writng" -> "writn" -> "writ"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writn' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writng' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writ' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writn' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Type correctly: "writing"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writi' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writ' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writin' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writi' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writing' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'I am writin' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('I am writing');

      // Save and verify
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('I am writing');
    });

    it('should handle middle insertion after reload', () => {
      let doc = createBlockNoteDocument();

      // Session 1: Type "Hello orld"
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello orld' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello orld');

      // Save and reload
      doc = Automerge.load(Automerge.save(doc));

      // Session 2: Notice missing 'W', insert it in the middle
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello World' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Hello orld' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      expect(doc.blocks[0].content[0].text).toBe('Hello World');

      // Save and reload again
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('Hello World');
    });
  });

  describe('Document size efficiency', () => {
    it('should keep document size reasonable with many edits', () => {
      let doc = createBlockNoteDocument([
        {
          id: 'block-1',
          type: 'paragraph',
          content: [{ type: 'text', text: '' }],
          children: [],
        } as any,
      ]);

      const initialSize = Automerge.save(doc).byteLength;

      // Simulate typing 20 characters one by one
      let text = '';
      for (let i = 0; i < 20; i++) {
        const prevText = text;
        text += String.fromCharCode(65 + i); // A, B, C, ...

        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text }],
              children: [],
            } as any,
            prevBlock: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: prevText }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
      }

      const finalSize = Automerge.save(doc).byteLength;
      const growth = finalSize - initialSize;

      expect(doc.blocks[0].content[0].text).toBe('ABCDEFGHIJKLMNOPQRST');
      console.log(`Document growth after 20 character insertions: ${growth} bytes`);

      // The growth should be reasonable (not exponential)
      // Each character should add roughly the same overhead
      expect(growth).toBeLessThan(1000); // Reasonable upper bound
    });

    it('should handle save/load cycles without size explosion', () => {
      let doc = createBlockNoteDocument([
        {
          id: 'block-1',
          type: 'paragraph',
          content: [{ type: 'text', text: 'Start' }],
          children: [],
        } as any,
      ]);

      const sizes: number[] = [];

      // Do 10 edit + save/load cycles
      for (let i = 0; i < 10; i++) {
        const prevText = doc.blocks[0].content[0].text;
        const newText = prevText + i;

        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: newText }],
              children: [],
            } as any,
            prevBlock: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: prevText }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);

        // Save and reload
        doc = Automerge.load(Automerge.save(doc));
        sizes.push(Automerge.save(doc).byteLength);
      }

      expect(doc.blocks[0].content[0].text).toBe('Start0123456789');

      console.log('Document sizes across cycles:', sizes);
      
      // Size should grow linearly, not exponentially
      // Check that later sizes don't grow disproportionately
      const firstHalfAvg = sizes.slice(0, 5).reduce((a, b) => a + b, 0) / 5;
      const secondHalfAvg = sizes.slice(5, 10).reduce((a, b) => a + b, 0) / 5;
      const growthRatio = secondHalfAvg / firstHalfAvg;

      // Growth ratio should be close to 1 (linear), not >> 1 (exponential)
      expect(growthRatio).toBeLessThan(1.5);
    });
  });

  describe('Complex real-world scenarios', () => {
    it('should handle the exact scenario from the problem statement', () => {
      // Step 1: Run blocknote-collab, enter some text
      let doc = createBlockNoteDocument();

      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Test document' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Save to localStorage (simulated)
      const saved = Automerge.save(doc);

      // Step 2: Reload the page
      doc = Automerge.load(saved);

      // Step 3: Text should be the same, not "complete garbage"
      expect(doc.blocks[0].content[0].text).toBe('Test document');

      // Step 4: Continue editing
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Test document with more text' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Test document' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Step 5: Save and reload again
      doc = Automerge.load(Automerge.save(doc));

      // Step 6: Text should STILL be correct, not garbage
      expect(doc.blocks[0].content[0].text).toBe('Test document with more text');
    });

    it('should handle rapid edits followed by reload', () => {
      let doc = createBlockNoteDocument();

      // Rapid typing
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'insert',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: '' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Type quickly: "Quick test"
      const steps = [
        ['', 'Q'],
        ['Q', 'Qu'],
        ['Qu', 'Qui'],
        ['Qui', 'Quic'],
        ['Quic', 'Quick'],
        ['Quick', 'Quick '],
        ['Quick ', 'Quick t'],
        ['Quick t', 'Quick te'],
        ['Quick te', 'Quick tes'],
        ['Quick tes', 'Quick test'],
      ];

      for (const [prev, curr] of steps) {
        doc = applyBlockNoteChanges(doc, [
          {
            type: 'update',
            block: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: curr }],
              children: [],
            } as any,
            prevBlock: {
              id: 'block-1',
              type: 'paragraph',
              content: [{ type: 'text', text: prev }],
              children: [],
            } as any,
            source: { type: 'local' },
          },
        ]);
      }

      expect(doc.blocks[0].content[0].text).toBe('Quick test');

      // Immediate save and reload
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('Quick test');

      // Continue editing after reload
      doc = applyBlockNoteChanges(doc, [
        {
          type: 'update',
          block: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Quick test completed' }],
            children: [],
          } as any,
          prevBlock: {
            id: 'block-1',
            type: 'paragraph',
            content: [{ type: 'text', text: 'Quick test' }],
            children: [],
          } as any,
          source: { type: 'local' },
        },
      ]);

      // Final check
      doc = Automerge.load(Automerge.save(doc));
      expect(doc.blocks[0].content[0].text).toBe('Quick test completed');
    });
  });
});
