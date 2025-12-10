import * as Automerge from '@automerge/automerge';
import {
  applyEditorChanges,
  createTextDocument,
  compareDocumentSizes,
  type EditorChanges,
} from '../src/index';

console.log('='.repeat(60));
console.log('Automerge Utils Demo');
console.log('Demonstrating minimal document growth with editor changes');
console.log('='.repeat(60));

// Create initial document
let doc = createTextDocument('');
console.log('\n1. Creating empty document...');
console.log(`   Initial size: ${Automerge.save(doc).byteLength} bytes`);

// Simulate typing "Hello World"
console.log('\n2. Simulating typing "Hello World" character by character...');
const characters = 'Hello World';
for (let i = 0; i < characters.length; i++) {
  const oldDoc = doc;
  const changes: EditorChanges = {
    changes: [{ from: i, to: i, text: characters[i] }],
  };
  doc = applyEditorChanges(doc, changes);
  
  const comparison = compareDocumentSizes(oldDoc, doc);
  console.log(`   [${i + 1}/${characters.length}] Added '${characters[i]}' - Growth: ${comparison.growth} bytes`);
}

console.log(`\n   Final text: "${doc.text}"`);
console.log(`   Final size: ${Automerge.save(doc).byteLength} bytes`);

// Demonstrate batch insertion
console.log('\n3. Demonstrating batch insertion (paste operation)...');
const beforeBatch = doc;
const batchChanges: EditorChanges = {
  changes: [{ from: 11, to: 11, text: ' - Batch inserted!' }],
};
doc = applyEditorChanges(doc, batchChanges);
const batchComparison = compareDocumentSizes(beforeBatch, doc);
console.log(`   Text after batch: "${doc.text}"`);
console.log(`   Growth: ${batchComparison.growth} bytes (${batchComparison.growthPercent.toFixed(2)}%)`);

// Demonstrate deletion
console.log('\n4. Demonstrating deletion...');
const beforeDelete = doc;
const deleteChanges: EditorChanges = {
  changes: [{ from: 11, to: 30, text: '' }],
};
doc = applyEditorChanges(doc, deleteChanges);
const deleteComparison = compareDocumentSizes(beforeDelete, doc);
console.log(`   Text after deletion: "${doc.text}"`);
console.log(`   Growth: ${deleteComparison.growth} bytes (${deleteComparison.growthPercent.toFixed(2)}%)`);

// Demonstrate replacement
console.log('\n5. Demonstrating replacement...');
const beforeReplace = doc;
const replaceChanges: EditorChanges = {
  changes: [{ from: 6, to: 11, text: 'Universe' }],
};
doc = applyEditorChanges(doc, replaceChanges);
const replaceComparison = compareDocumentSizes(beforeReplace, doc);
console.log(`   Text after replacement: "${doc.text}"`);
console.log(`   Growth: ${replaceComparison.growth} bytes (${replaceComparison.growthPercent.toFixed(2)}%)`);

// Demonstrate multiple concurrent changes
console.log('\n6. Demonstrating concurrent edits and merge...');
const baseDoc = createTextDocument('The quick brown fox jumps over the lazy dog');
console.log(`   Base text: "${baseDoc.text}"`);

// User 1 edits
let user1Doc = Automerge.clone(baseDoc);
user1Doc = applyEditorChanges(user1Doc, {
  changes: [{ from: 10, to: 15, text: 'red' }], // Change "brown" to "red"
});
console.log(`   User 1 edit: "${user1Doc.text}"`);

// User 2 edits
let user2Doc = Automerge.clone(baseDoc);
user2Doc = applyEditorChanges(user2Doc, {
  changes: [{ from: 35, to: 39, text: 'sleepy' }], // Change "lazy" to "sleepy"
});
console.log(`   User 2 edit: "${user2Doc.text}"`);

// Merge both edits
const merged = Automerge.merge(user1Doc, user2Doc);
console.log(`   Merged result: "${merged.text}"`);
console.log(`   Merged size: ${Automerge.save(merged).byteLength} bytes`);

// Compare with naive approach
console.log('\n7. Comparing with naive approach (full document replacement)...');
const naiveBase = createTextDocument('');
const textSequence = ['H', 'He', 'Hel', 'Hell', 'Hello'];

console.log('   Naive approach (replace entire document each time):');
let naiveDoc = naiveBase;
for (const text of textSequence) {
  const oldSize = Automerge.save(naiveDoc).byteLength;
  naiveDoc = Automerge.change(naiveDoc, (d) => {
    d.text = text;
  });
  const newSize = Automerge.save(naiveDoc).byteLength;
  console.log(`     "${text}" - Size: ${newSize} bytes (growth: ${newSize - oldSize})`);
}
const naiveTotalSize = Automerge.save(naiveDoc).byteLength;

console.log('\n   Optimized approach (precise changes):');
let optimizedDoc = createTextDocument('');
for (let i = 0; i < textSequence.length; i++) {
  const text = textSequence[i];
  const oldSize = Automerge.save(optimizedDoc).byteLength;
  
  const prevText = i > 0 ? textSequence[i - 1] : '';
  const newChar = text[text.length - 1];
  
  optimizedDoc = applyEditorChanges(optimizedDoc, {
    changes: [{ from: prevText.length, to: prevText.length, text: newChar }],
  });
  
  const newSize = Automerge.save(optimizedDoc).byteLength;
  console.log(`     "${text}" - Size: ${newSize} bytes (growth: ${newSize - oldSize})`);
}
const optimizedTotalSize = Automerge.save(optimizedDoc).byteLength;

console.log(`\n   Naive total size: ${naiveTotalSize} bytes`);
console.log(`   Optimized total size: ${optimizedTotalSize} bytes`);
console.log(`   Difference: ${naiveTotalSize - optimizedTotalSize} bytes`);
console.log(`   Optimized is ${((naiveTotalSize - optimizedTotalSize) / naiveTotalSize * 100).toFixed(2)}% smaller`);

console.log('\n' + '='.repeat(60));
console.log('Demo completed!');
console.log('Key takeaway: Using precise editor changes keeps document growth minimal');
console.log('='.repeat(60));
