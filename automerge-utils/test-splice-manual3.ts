import * as Automerge from '@automerge/automerge';

// Test insert
let doc = Automerge.from({ text: "hello" });
console.log("Initial:", doc.text);

doc = Automerge.change(doc, d => {
  Automerge.splice(d, ['text'], 5, 0, " world");  // Insert at end
});
console.log("After insert:", doc.text);

// Test replace
doc = Automerge.change(doc, d => {
  Automerge.splice(d, ['text'], 6, 5, "universe");  // Replace "world" with "universe"
});
console.log("After replace:", doc.text);

// Test nested object
let doc2 = Automerge.from({ 
  blocks: [{ id: '1', text: 'hello' }] 
});
console.log("\nNested initial:", doc2.blocks[0].text);

doc2 = Automerge.change(doc2, d => {
  Automerge.splice(d.blocks[0], ['text'], 5, 0, " world");
});
console.log("Nested after append:", doc2.blocks[0].text);

// Test character-by-character
let doc3 = Automerge.from({ text: "" });
doc3 = Automerge.change(doc3, d => {
  Automerge.splice(d, ['text'], 0, 0, "h");
});
doc3 = Automerge.change(doc3, d => {
  Automerge.splice(d, ['text'], 1, 0, "e");
});
doc3 = Automerge.change(doc3, d => {
  Automerge.splice(d, ['text'], 2, 0, "l");
});
console.log("\nIncremental:", doc3.text);
