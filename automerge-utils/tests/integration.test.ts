import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyEditorChanges,
  createTextDocument,
  compareDocumentSizes,
} from '../src/transformer';
import type { EditorChanges } from '../src/types';

describe('Integration: Minimal Document Growth', () => {
  it('should keep document size minimal with sequential edits', () => {
    let doc = createTextDocument('');
    
    const initialSize = Automerge.save(doc).byteLength;
    
    // Simulate typing "Hello World" character by character
    const edits = [
      { from: 0, to: 0, text: 'H' },
      { from: 1, to: 1, text: 'e' },
      { from: 2, to: 2, text: 'l' },
      { from: 3, to: 3, text: 'l' },
      { from: 4, to: 4, text: 'o' },
      { from: 5, to: 5, text: ' ' },
      { from: 6, to: 6, text: 'W' },
      { from: 7, to: 7, text: 'o' },
      { from: 8, to: 8, text: 'r' },
      { from: 9, to: 9, text: 'l' },
      { from: 10, to: 10, text: 'd' },
    ];

    for (const edit of edits) {
      const changes: EditorChanges = { changes: [edit] };
      doc = applyEditorChanges(doc, changes);
    }

    expect(doc.text).toBe('Hello World');
    
    const finalSize = Automerge.save(doc).byteLength;
    const growth = finalSize - initialSize;
    
    // Document should grow, but not excessively
    expect(growth).toBeLessThan(1000); // Reasonable threshold
  });

  it('should optimize batch changes', () => {
    const doc = createTextDocument('');
    
    // Apply multiple changes at once (like pasting)
    // Both changes are relative to the original empty document
    const changes: EditorChanges = {
      changes: [
        { from: 0, to: 0, text: 'Hello ' },
        { from: 0, to: 0, text: 'World' },
      ],
    };

    const newDoc = applyEditorChanges(doc, changes);
    // Since both are at position 0, they merge and one comes after the other
    expect(newDoc.text).toContain('Hello');
    expect(newDoc.text).toContain('World');
    
    const comparison = compareDocumentSizes(doc, newDoc);
    
    // Batch changes should be efficient
    expect(comparison.growth).toBeGreaterThan(0);
    expect(comparison.growth).toBeLessThan(500);
  });

  it('should handle deletion efficiently', () => {
    const doc = createTextDocument('Hello World');
    const initialSize = Automerge.save(doc).byteLength;
    
    // Delete "World"
    const changes: EditorChanges = {
      changes: [{ from: 6, to: 11, text: '' }],
    };

    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello ');
    
    const finalSize = Automerge.save(newDoc).byteLength;
    
    // Size might grow slightly due to operation metadata, but should be minimal
    expect(finalSize - initialSize).toBeLessThan(200);
  });

  it('should handle replacement efficiently', () => {
    const doc = createTextDocument('Hello World');
    const initialSize = Automerge.save(doc).byteLength;
    
    // Replace "World" with "Universe"
    const changes: EditorChanges = {
      changes: [{ from: 6, to: 11, text: 'Universe' }],
    };

    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello Universe');
    
    const finalSize = Automerge.save(newDoc).byteLength;
    const growth = finalSize - initialSize;
    
    // Growth should be proportional to the change
    expect(growth).toBeLessThan(500);
  });

  it('should compare with naive approach', () => {
    // Naive approach: replace entire document each time
    let naiveDoc = createTextDocument('');
    
    const text = 'Hello World';
    naiveDoc = Automerge.change(naiveDoc, (d) => {
      d.text = text;
    });
    
    const naiveSize = Automerge.save(naiveDoc).byteLength;
    
    // Optimized approach: apply precise changes
    let optimizedDoc = createTextDocument('');
    
    const changes: EditorChanges = {
      changes: [{ from: 0, to: 0, text: 'Hello World' }],
    };
    
    optimizedDoc = applyEditorChanges(optimizedDoc, changes);
    
    const optimizedSize = Automerge.save(optimizedDoc).byteLength;
    
    // Both should produce same result
    expect(optimizedDoc.text).toBe(naiveDoc.text);
    
    // Sizes should be similar for simple case
    expect(Math.abs(optimizedSize - naiveSize)).toBeLessThan(100);
  });

  it('should demonstrate efficiency with multiple sequential edits', () => {
    // Create a document and make multiple edits sequentially
    let doc = createTextDocument('The quick brown fox');
    const initialSize = Automerge.save(doc).byteLength;
    
    // Make several edits, one at a time (sequential, not batched)
    doc = applyEditorChanges(doc, {
      changes: [{ from: 10, to: 15, text: 'red' }] // Replace "brown" with "red"
    });
    
    doc = applyEditorChanges(doc, {
      changes: [{ from: 14, to: 14, text: ' fluffy' }] // Add " fluffy" after "red"
    });
    
    doc = applyEditorChanges(doc, {
      changes: [{ from: 0, to: 3, text: 'A' }] // Replace "The" with "A"
    });

    expect(doc.text).toBe('A quick red fluffy fox');
    
    const finalSize = Automerge.save(doc).byteLength;
    const growth = finalSize - initialSize;
    
    // Growth should be reasonable for 3 operations
    expect(growth).toBeLessThan(1000);
  });

  it('should handle sequential insertions efficiently', () => {
    // Start with base text
    let doc = createTextDocument('Hello');
    
    // Simulate typing " World" one character at a time (sequential, not batched)
    doc = applyEditorChanges(doc, { changes: [{ from: 5, to: 5, text: ' ' }] });
    doc = applyEditorChanges(doc, { changes: [{ from: 6, to: 6, text: 'W' }] });
    doc = applyEditorChanges(doc, { changes: [{ from: 7, to: 7, text: 'o' }] });
    doc = applyEditorChanges(doc, { changes: [{ from: 8, to: 8, text: 'r' }] });
    doc = applyEditorChanges(doc, { changes: [{ from: 9, to: 9, text: 'l' }] });
    doc = applyEditorChanges(doc, { changes: [{ from: 10, to: 10, text: 'd' }] });

    expect(doc.text).toBe('Hello World');
    
    // Check that document grew minimally
    const baseDoc = createTextDocument('Hello');
    const comparison = compareDocumentSizes(baseDoc, doc);
    expect(comparison.growth).toBeLessThan(500);
  });
});

describe('Integration: Automerge Synchronization', () => {
  it('should apply changes to cloned documents', () => {
    // Create two documents from same base
    const base = createTextDocument('Hello');
    let doc1 = Automerge.clone(base);
    let doc2 = Automerge.clone(base);
    
    // Apply changes to doc1
    doc1 = applyEditorChanges(doc1, {
      changes: [{ from: 5, to: 5, text: ' World' }],
    });
    expect(doc1.text).toBe('Hello World');
    
    // Apply different changes to doc2
    doc2 = applyEditorChanges(doc2, {
      changes: [{ from: 0, to: 0, text: 'Greetings: ' }],
    });
    expect(doc2.text).toBe('Greetings: Hello');
    
    // Note: Merging plain string properties with concurrent edits
    // will result in last-write-wins behavior, not CRDT text merging
    // For true CRDT text support, use Automerge.Text instead
  });

  it('should handle sequential edits on cloned documents', () => {
    const initial = createTextDocument('abcdef');
    
    // Clone and edit
    let doc1 = Automerge.clone(initial);
    doc1 = applyEditorChanges(doc1, {
      changes: [{ from: 3, to: 3, text: 'X' }],
    });
    expect(doc1.text).toBe('abcXdef');
    
    // Clone the original and edit differently
    let doc2 = Automerge.clone(initial);
    doc2 = applyEditorChanges(doc2, {
      changes: [{ from: 4, to: 4, text: 'Y' }],
    });
    expect(doc2.text).toBe('abcdYef');
    
    // Each document has its own changes applied correctly
    expect(doc1.text).not.toBe(doc2.text);
  });
  });
});
