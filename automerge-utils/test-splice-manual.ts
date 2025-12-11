import * as Automerge from '@automerge/automerge';

// Create a test document with a string
let doc = Automerge.from({ text: "hello world" });
console.log("Initial:", doc.text);

// Test 1: Delete characters using splice
doc = Automerge.change(doc, d => {
  // Delete 6 characters starting at position 5 (" world")
  Automerge.splice(d, 'text', 5, 6);
});
console.log("After delete:", doc.text);

// Test 2: Insert characters using splice
doc = Automerge.change(doc, d => {
  // Insert " there" at position 5
  Automerge.splice(d, 'text', 5, 0, " there");
});
console.log("After insert:", doc.text);

// Test 3: Replace characters using splice
doc = Automerge.change(doc, d => {
  // Replace 5 characters starting at position 6 ("there") with "universe"
  Automerge.splice(d, 'text', 6, 5, "universe");
});
console.log("After replace:", doc.text);

// Test 4: Test on nested object
let doc2 = Automerge.from({ 
  blocks: [{ id: '1', text: 'hello' }] 
});
console.log("\nNested initial:", doc2.blocks[0].text);

doc2 = Automerge.change(doc2, d => {
  // Append to nested text
  Automerge.splice(d.blocks[0], 'text', 5, 0, " world");
});
console.log("Nested after append:", doc2.blocks[0].text);
