import * as Automerge from '@automerge/automerge';
import {
  applyBlockNoteChanges,
  createBlockNoteDocument,
  compareDocumentSizes,
  type BlockNoteChanges,
} from '../src/index';

console.log('='.repeat(70));
console.log('BlockNote + Automerge Utils Demo');
console.log('Demonstrating minimal document growth with BlockNote editor changes');
console.log('='.repeat(70));

// Create initial empty document
let doc = createBlockNoteDocument();
console.log('\n1. Creating empty BlockNote document...');
console.log(`   Initial size: ${Automerge.save(doc).byteLength} bytes`);
console.log(`   Blocks count: ${doc.blocks.length}`);

// Simulate inserting a paragraph block
console.log('\n2. Inserting first paragraph block...');
const firstBlock = {
  id: '1',
  type: 'paragraph',
  content: [{ type: 'text', text: 'Hello World!' }],
  children: [],
};

let changes: BlockNoteChanges = [
  {
    type: 'insert',
    block: firstBlock as any,
    source: { type: 'local' },
    prevBlock: undefined,
  },
];

const beforeFirstInsert = doc;
doc = applyBlockNoteChanges(doc, changes);
let comparison = compareDocumentSizes(beforeFirstInsert, doc);
console.log(`   Text: "${(doc.blocks[0] as any).content[0].text}"`);
console.log(`   Size: ${Automerge.save(doc).byteLength} bytes (growth: ${comparison.growth} bytes)`);

// Simulate inserting a heading block
console.log('\n3. Inserting heading block...');
const headingBlock = {
  id: '2',
  type: 'heading',
  content: [{ type: 'text', text: 'My Document' }],
  children: [],
};

changes = [
  {
    type: 'insert',
    block: headingBlock as any,
    source: { type: 'local' },
    prevBlock: undefined,
  },
];

const beforeHeading = doc;
doc = applyBlockNoteChanges(doc, changes);
comparison = compareDocumentSizes(beforeHeading, doc);
console.log(`   Blocks count: ${doc.blocks.length}`);
console.log(`   Size: ${Automerge.save(doc).byteLength} bytes (growth: ${comparison.growth} bytes)`);

// Simulate updating the first block
console.log('\n4. Updating first paragraph block...');
const updatedBlock = {
  ...firstBlock,
  content: [{ type: 'text', text: 'Hello World! This text was updated.' }],
};

changes = [
  {
    type: 'update',
    block: updatedBlock as any,
    prevBlock: firstBlock as any,
    source: { type: 'local' },
  },
];

const beforeUpdate = doc;
doc = applyBlockNoteChanges(doc, changes);
comparison = compareDocumentSizes(beforeUpdate, doc);
console.log(`   Updated text: "${(doc.blocks[0] as any).content[0].text}"`);
console.log(`   Size: ${Automerge.save(doc).byteLength} bytes (growth: ${comparison.growth} bytes)`);

// Demonstrate surgical text updates with string splice optimization
console.log('\n4.5. Demonstrating surgical string splice optimization...');
console.log('   (Only changed parts of the string are updated, not the entire field)');

// Start with a block
const textBlock = doc.blocks[0];
let currentText = (textBlock as any).content[0].text;

// Step 1: Append characters
console.log(`\n   Step 1: Appending characters "${currentText}" -> "${currentText} text"`);
const step1Before = doc;
changes = [
  {
    type: 'update',
    block: {
      ...textBlock,
      content: [{ type: 'text', text: currentText + ' text' }],
    } as any,
    prevBlock: textBlock as any,
    source: { type: 'local' },
  },
];
doc = applyBlockNoteChanges(doc, changes);
const step1Growth = compareDocumentSizes(step1Before, doc).growth;
console.log(`   Growth: ${step1Growth} bytes (only " text" was added)`);
currentText = (doc.blocks[0] as any).content[0].text;

// Step 2: Insert in middle
console.log(`\n   Step 2: Fixing typo in middle "${currentText}" -> "Hello World! This text"`);
const step2Before = doc;
const fixedText = 'Hello World! This text';
changes = [
  {
    type: 'update',
    block: {
      ...doc.blocks[0],
      content: [{ type: 'text', text: fixedText }],
    } as any,
    prevBlock: doc.blocks[0] as any,
    source: { type: 'local' },
  },
];
doc = applyBlockNoteChanges(doc, changes);
const step2Growth = compareDocumentSizes(step2Before, doc).growth;
console.log(`   Growth: ${step2Growth} bytes (minimal change in middle)`);

// Step 3: Simulate character-by-character typing
console.log(`\n   Step 3: Typing " was" character by character...`);
const typingSteps = [' was', ' was ', ' was u', ' was up', ' was upd', ' was upda', ' was updat', ' was update', ' was updated'];
let typingGrowth = 0;
let prevTypingText = fixedText;

for (const suffix of typingSteps) {
  const beforeTyping = doc;
  const newText = fixedText + suffix;
  
  changes = [
    {
      type: 'update',
      block: {
        ...doc.blocks[0],
        content: [{ type: 'text', text: newText }],
      } as any,
      prevBlock: {
        ...doc.blocks[0],
        content: [{ type: 'text', text: prevTypingText }],
      } as any,
      source: { type: 'local' },
    },
  ];
  
  doc = applyBlockNoteChanges(doc, changes);
  const charGrowth = compareDocumentSizes(beforeTyping, doc).growth;
  typingGrowth += charGrowth;
  prevTypingText = newText;
}

console.log(`   Total growth for ${typingSteps.length} incremental edits: ${typingGrowth} bytes`);
console.log(`   Average: ${(typingGrowth / typingSteps.length).toFixed(1)} bytes per edit`);
console.log(`   Final text: "${(doc.blocks[0] as any).content[0].text}"`);

// Simulate batch paste operation
console.log('\n5. Simulating paste operation (batch insert)...');
const pasteBlocks = [
  {
    id: '3',
    type: 'paragraph',
    content: [{ type: 'text', text: 'Pasted paragraph 1' }],
    children: [],
  },
  {
    id: '4',
    type: 'paragraph',
    content: [{ type: 'text', text: 'Pasted paragraph 2' }],
    children: [],
  },
  {
    id: '5',
    type: 'bulletListItem',
    content: [{ type: 'text', text: 'Pasted list item' }],
    children: [],
  },
];

changes = pasteBlocks.map((block) => ({
  type: 'insert' as const,
  block: block as any,
  source: { type: 'paste' as const },
  prevBlock: undefined,
}));

const beforePaste = doc;
doc = applyBlockNoteChanges(doc, changes);
comparison = compareDocumentSizes(beforePaste, doc);
console.log(`   Pasted ${pasteBlocks.length} blocks`);
console.log(`   Total blocks: ${doc.blocks.length}`);
console.log(`   Size: ${Automerge.save(doc).byteLength} bytes (growth: ${comparison.growth} bytes)`);

// Simulate deleting a block
console.log('\n6. Deleting a block...');
const blockToDelete = doc.blocks[2]; // Delete the third block

changes = [
  {
    type: 'delete',
    block: blockToDelete as any,
    source: { type: 'local' },
    prevBlock: undefined,
  },
];

const beforeDelete = doc;
doc = applyBlockNoteChanges(doc, changes);
comparison = compareDocumentSizes(beforeDelete, doc);
console.log(`   Deleted block with id: ${(blockToDelete as any).id}`);
console.log(`   Remaining blocks: ${doc.blocks.length}`);
console.log(`   Size: ${Automerge.save(doc).byteLength} bytes (growth: ${comparison.growth} bytes)`);

// Show final state
console.log('\n7. Final document state:');
console.log(`   Total blocks: ${doc.blocks.length}`);
console.log(`   Final size: ${Automerge.save(doc).byteLength} bytes`);
console.log('\n   Block contents:');
doc.blocks.forEach((block: any, index) => {
  const text = block.content?.[0]?.text || '(no text)';
  console.log(`   ${index + 1}. [${block.type}] ${text}`);
});

// Compare with naive approach
console.log('\n8. Comparing with naive full-replacement approach...');
let naiveDoc = createBlockNoteDocument();
const naiveInitialSize = Automerge.save(naiveDoc).byteLength;

// Naive: replace entire blocks array each time
for (let i = 0; i < 5; i++) {
  naiveDoc = Automerge.change(naiveDoc, (d) => {
    d.blocks = [
      {
        id: `${i + 1}`,
        type: 'paragraph',
        content: [{ type: 'text', text: `Block ${i + 1}` }],
        children: [],
      } as any,
    ];
  });
}
const naiveFinalSize = Automerge.save(naiveDoc).byteLength;
const naiveGrowth = naiveFinalSize - naiveInitialSize;

// Optimized: use applyBlockNoteChanges
let optimizedDoc = createBlockNoteDocument();
const optimizedInitialSize = Automerge.save(optimizedDoc).byteLength;

for (let i = 0; i < 5; i++) {
  optimizedDoc = applyBlockNoteChanges(optimizedDoc, [
    {
      type: 'insert',
      block: {
        id: `${i + 1}`,
        type: 'paragraph',
        content: [{ type: 'text', text: `Block ${i + 1}` }],
        children: [],
      } as any,
      source: { type: 'local' },
      prevBlock: undefined,
    },
  ]);
}
const optimizedFinalSize = Automerge.save(optimizedDoc).byteLength;
const optimizedGrowth = optimizedFinalSize - optimizedInitialSize;

console.log(`   Naive approach: ${naiveFinalSize} bytes (growth: ${naiveGrowth} bytes)`);
console.log(`   Optimized approach: ${optimizedFinalSize} bytes (growth: ${optimizedGrowth} bytes)`);
console.log(`   Savings: ${naiveGrowth - optimizedGrowth} bytes (${((naiveGrowth - optimizedGrowth) / naiveGrowth * 100).toFixed(1)}%)`);

console.log('\n' + '='.repeat(70));
console.log('Demo completed!');
console.log('Key takeaway: Using precise BlockNote changes keeps document growth minimal');
console.log('compared to replacing the entire blocks array on each change.');
console.log('='.repeat(70));
