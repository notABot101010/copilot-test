import * as Automerge from '@automerge/automerge';

// Test 1: Basic string with splice
let doc = Automerge.from({ text: "hello world" });
console.log("Initial:", doc.text);

// Try splice with array path
doc = Automerge.change(doc, d => {
  // Automerge.splice(obj, path, index, deletions, ...insertions)
  Automerge.splice(d, ['text'], 5, 6);  // Try with array path
});
console.log("After delete:", doc.text);
