import { describe, it, expect } from 'vitest';
import * as Automerge from '@automerge/automerge';

describe('Automerge.splice behavior', () => {
  it('should work when splicing from nested object', () => {
    let doc = Automerge.from({
      items: [
        { id: '1', text: 'Hello' },
      ],
    });

    doc = Automerge.change(doc, (d) => {
      const item = d.items[0];
      // Try to splice the text field directly on the nested object
      Automerge.splice(item, ['text'], 5, 0, ' World');
    });

    console.log('After splice on nested object:', doc.items[0].text);
    expect(doc.items[0].text).toBe('Hello World');

    // Save and reload
    const saved = Automerge.save(doc);
    const loaded = Automerge.load(saved);

    console.log('After save/load:', loaded.items[0].text);
    expect(loaded.items[0].text).toBe('Hello World');
  });

  it('should work when using assignment instead', () => {
    let doc = Automerge.from({
      items: [
        { id: '1', text: 'Hello' },
      ],
    });

    doc = Automerge.change(doc, (d) => {
      // Just assign the new value
      d.items[0].text = 'Hello World';
    });

    console.log('After assignment:', doc.items[0].text);
    expect(doc.items[0].text).toBe('Hello World');

    // Save and reload
    const saved = Automerge.save(doc);
    const loaded = Automerge.load(saved);

    console.log('After save/load with assignment:', loaded.items[0].text);
    expect(loaded.items[0].text).toBe('Hello World');
  });

  it('should compare splice vs assignment for same content', () => {
    // Using splice
    let docWithSplice = Automerge.from({
      items: [{ text: 'Hello' }],
    });

    docWithSplice = Automerge.change(docWithSplice, (d) => {
      Automerge.splice(d.items[0], ['text'], 5, 0, ' World');
    });

    // Using assignment
    let docWithAssignment = Automerge.from({
      items: [{ text: 'Hello' }],
    });

    docWithAssignment = Automerge.change(docWithAssignment, (d) => {
      d.items[0].text = 'Hello World';
    });

    console.log('Splice result:', docWithSplice.items[0].text);
    console.log('Assignment result:', docWithAssignment.items[0].text);

    expect(docWithSplice.items[0].text).toBe('Hello World');
    expect(docWithAssignment.items[0].text).toBe('Hello World');

    // Both should work after save/load
    const savedSplice = Automerge.save(docWithSplice);
    const loadedSplice = Automerge.load(savedSplice);

    const savedAssignment = Automerge.save(docWithAssignment);
    const loadedAssignment = Automerge.load(savedAssignment);

    console.log('Loaded splice:', loadedSplice.items[0].text);
    console.log('Loaded assignment:', loadedAssignment.items[0].text);

    expect(loadedSplice.items[0].text).toBe('Hello World');
    expect(loadedAssignment.items[0].text).toBe('Hello World');
  });
});
