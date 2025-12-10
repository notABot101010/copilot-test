# @copilot-test/automerge-utils

A TypeScript package for transforming BlockNote editor changes into minimal Automerge document updates. This package helps keep Automerge document size small by applying precise, optimized block-level changes instead of replacing entire document arrays.

## Features

- 🎯 **Minimal Updates**: Transform BlockNote block changes into precise Automerge operations
- 📦 **Small Document Size**: Optimizations to prevent document bloat
- 🔄 **Block-Level Operations**: Handle insert, update, delete, and move operations efficiently
- ⚡ **Efficient**: Filters out no-op changes automatically
- 🧪 **Well Tested**: Comprehensive unit and integration tests
- 📊 **Size Tracking**: Built-in utilities to monitor document growth
- 🎨 **BlockNote Native**: Designed specifically for BlockNote's change API

## Installation

```bash
npm install @copilot-test/automerge-utils
```

## Usage

### Basic Example with BlockNote

```typescript
import { useCreateBlockNote } from '@blocknote/react';
import { applyBlockNoteChanges, createBlockNoteDocument } from '@copilot-test/automerge-utils';
import * as Automerge from '@automerge/automerge';

function MyEditor() {
  // Create Automerge document
  let doc = createBlockNoteDocument();

  // Create BlockNote editor
  const editor = useCreateBlockNote({
    // ... your BlockNote config
  });

  // Listen for changes
  editor.onChange((editor, { getChanges }) => {
    const changes = getChanges();
    
    // Apply changes to Automerge document
    doc = applyBlockNoteChanges(doc, changes);
    
    // Now sync doc to your backend, other clients, etc.
    syncDocument(doc);
  });

  return <BlockNoteView editor={editor} />;
}
```

### Creating a Document

```typescript
import { createBlockNoteDocument } from '@copilot-test/automerge-utils';

// Create empty document
const doc = createBlockNoteDocument();

// Or with initial blocks
const docWithBlocks = createBlockNoteDocument([
  {
    id: '1',
    type: 'paragraph',
    content: [{ type: 'text', text: 'Hello World' }],
    children: [],
  },
]);
```

### Applying Changes

```typescript
import { applyBlockNoteChanges } from '@copilot-test/automerge-utils';

// Inside your onChange callback
editor.onChange((editor, { getChanges }) => {
  const changes = getChanges(); // Returns BlocksChanged from BlockNote
  
  // Apply to Automerge document
  doc = applyBlockNoteChanges(doc, changes);
});
```

### Options

```typescript
// Disable optimization (keeps all changes, even no-ops)
doc = applyBlockNoteChanges(doc, changes, { 
  optimize: false 
});
```

### Document Size Monitoring

```typescript
import { compareDocumentSizes, getDocumentSize } from '@copilot-test/automerge-utils';

// Get document size in bytes
const size = getDocumentSize(doc);
console.log(`Document size: ${size} bytes`);

// Compare sizes after changes
const comparison = compareDocumentSizes(oldDoc, newDoc);
console.log(`Growth: ${comparison.growth} bytes (${comparison.growthPercent}%)`);
```

## API Reference

### `applyBlockNoteChanges(doc, changes, options?)`

Applies BlockNote editor changes to an Automerge document in a minimal way.

**Parameters:**
- `doc`: The current Automerge document
- `changes`: Array of block changes from BlockNote's `getChanges()`
- `options`: Optional configuration
  - `optimize` (default: `true`): Filter out no-op changes

**Returns:** Updated Automerge document

### `createBlockNoteDocument(initialBlocks?)`

Creates a new Automerge document for BlockNote blocks.

**Parameters:**
- `initialBlocks`: Optional array of initial blocks

**Returns:** New Automerge document

### `getDocumentSize(doc)`

Gets the size of an Automerge document in bytes.

**Parameters:**
- `doc`: The Automerge document

**Returns:** Size in bytes

### `compareDocumentSizes(oldDoc, newDoc)`

Compares sizes of two Automerge documents.

**Parameters:**
- `oldDoc`: The original document
- `newDoc`: The updated document

**Returns:** Object with size comparison data

## Types

### `BlockNoteChange`

```typescript
interface BlockNoteChange {
  type: 'insert' | 'delete' | 'update' | 'move';
  block: Block;  // BlockNote block
  prevBlock?: Block;  // Previous state for updates
  source: {
    type: 'local' | 'paste' | 'drop' | 'undo' | 'redo' | 'undo-redo' | 'yjs-remote';
  };
}
```

### `BlockNoteChanges`

```typescript
type BlockNoteChanges = Array<BlockNoteChange>;
```

### `BlockNoteDocument`

```typescript
interface BlockNoteDocument {
  blocks: Block[];  // Array of BlockNote blocks
}
```

## Running Tests

```bash
# Run tests
npm test

# Run tests in watch mode
npm run test:watch
```

## Running Demo

```bash
npm run demo
```

The demo shows:
- Creating a BlockNote document
- Inserting paragraph and heading blocks
- Updating block content
- Batch operations (paste simulation)
- Block deletion
- Comparison with naive full-array replacement approach

## Why Use This Package?

When building collaborative BlockNote editors with Automerge, a common pitfall is replacing the entire blocks array on every change. This causes the Automerge document to grow rapidly because it stores the full operation history.

This package solves that by:

1. **Applying only actual block-level changes** (insert, update, delete, move)
2. **Filtering out no-op changes** that don't actually modify content
3. **Using precise block operations** for minimal overhead
4. **Leveraging BlockNote's native change tracking**

## Performance

Using precise block changes instead of full array replacement can reduce document size growth, especially for documents with frequent edits. The savings depend on the editing patterns and document size.

## BlockNote Integration

This package is specifically designed to work with BlockNote's `onChange` callback:

```typescript
editor.onChange((editor, { getChanges }) => {
  const changes = getChanges(); // BlocksChanged array
  doc = applyBlockNoteChanges(doc, changes);
});
```

The `getChanges()` function returns an array of block-level changes that describe what happened in the editor. This package transforms these changes into efficient Automerge operations.

## License

MIT
