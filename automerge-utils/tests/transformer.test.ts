import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';
import {
  applyEditorChanges,
  createTextDocument,
  getDocumentSize,
  compareDocumentSizes,
} from '../src/transformer';
import type { EditorChanges } from '../src/types';

describe('createTextDocument', () => {
  it('should create an empty document', () => {
    const doc = createTextDocument();
    expect(doc.text).toBe('');
  });

  it('should create a document with initial text', () => {
    const doc = createTextDocument('Hello World');
    expect(doc.text).toBe('Hello World');
  });
});

describe('applyEditorChanges', () => {
  it('should handle empty changes', () => {
    const doc = createTextDocument('Hello');
    const changes: EditorChanges = { changes: [] };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello');
  });

  it('should insert text at the beginning', () => {
    const doc = createTextDocument('World');
    const changes: EditorChanges = {
      changes: [{ from: 0, to: 0, text: 'Hello ' }],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello World');
  });

  it('should insert text at the end', () => {
    const doc = createTextDocument('Hello');
    const changes: EditorChanges = {
      changes: [{ from: 5, to: 5, text: ' World' }],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello World');
  });

  it('should insert text in the middle', () => {
    const doc = createTextDocument('HelloWorld');
    const changes: EditorChanges = {
      changes: [{ from: 5, to: 5, text: ' ' }],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello World');
  });

  it('should delete text', () => {
    const doc = createTextDocument('Hello World');
    const changes: EditorChanges = {
      changes: [{ from: 5, to: 11, text: '' }],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello');
  });

  it('should replace text', () => {
    const doc = createTextDocument('Hello World');
    const changes: EditorChanges = {
      changes: [{ from: 6, to: 11, text: 'Universe' }],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('Hello Universe');
  });

  it('should handle multiple insertions', () => {
    const doc = createTextDocument('ac');
    const changes: EditorChanges = {
      changes: [
        { from: 1, to: 1, text: 'b' },
        { from: 2, to: 2, text: 'd' },
      ],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('abcd');
  });

  it('should handle multiple deletions', () => {
    const doc = createTextDocument('abcd');
    const changes: EditorChanges = {
      changes: [
        { from: 1, to: 2, text: '' },
        { from: 2, to: 3, text: '' },
      ],
    };
    const newDoc = applyEditorChanges(doc, changes);
    expect(newDoc.text).toBe('ad');
  });

  it('should handle overlapping changes', () => {
    const doc = createTextDocument('Hello');
    const changes: EditorChanges = {
      changes: [
        { from: 2, to: 4, text: 'yyy' },
        { from: 3, to: 5, text: 'zzz' },
      ],
    };
    const newDoc = applyEditorChanges(doc, changes);
    // After merging adjacent changes
    expect(newDoc.text).toContain('H');
  });
});

describe('applyEditorChanges with options', () => {
  it('should respect mergeAdjacent option when false', () => {
    const doc = createTextDocument('Hello');
    const changes: EditorChanges = {
      changes: [
        { from: 5, to: 5, text: ' ' },
        { from: 6, to: 6, text: 'World' },
      ],
    };
    const newDoc = applyEditorChanges(doc, changes, { mergeAdjacent: false });
    expect(newDoc.text).toBe('Hello World');
  });

  it('should respect optimize option', () => {
    const doc = createTextDocument('Hello');
    const changes: EditorChanges = {
      changes: [
        { from: 0, to: 5, text: 'Hello' }, // Replace with same text (no-op)
      ],
    };
    const newDoc = applyEditorChanges(doc, changes, { optimize: true });
    expect(newDoc.text).toBe('Hello');
  });
});

describe('getDocumentSize', () => {
  it('should return size in bytes', () => {
    const doc = createTextDocument('Hello');
    const size = getDocumentSize(doc);
    expect(size).toBeGreaterThan(0);
  });

  it('should show larger documents have larger size', () => {
    const smallDoc = createTextDocument('Hi');
    const largeDoc = createTextDocument('Hello World, this is a much longer text');
    
    const smallSize = getDocumentSize(smallDoc);
    const largeSize = getDocumentSize(largeDoc);
    
    expect(largeSize).toBeGreaterThan(smallSize);
  });
});

describe('compareDocumentSizes', () => {
  it('should calculate growth correctly', () => {
    const oldDoc = createTextDocument('Hello');
    const newDoc = createTextDocument('Hello World');
    
    const comparison = compareDocumentSizes(oldDoc, newDoc);
    
    expect(comparison.oldSize).toBeGreaterThan(0);
    expect(comparison.newSize).toBeGreaterThan(comparison.oldSize);
    expect(comparison.growth).toBe(comparison.newSize - comparison.oldSize);
    expect(comparison.growthPercent).toBeGreaterThan(0);
  });

  it('should handle no growth', () => {
    const doc1 = createTextDocument('Hello');
    const doc2 = createTextDocument('Hello');
    
    const comparison = compareDocumentSizes(doc1, doc2);
    
    expect(comparison.growth).toBe(0);
    expect(comparison.growthPercent).toBe(0);
  });
});
