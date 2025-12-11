import * as Automerge from '@automerge/automerge';
import type { Block, BlockSchema, InlineContentSchema, StyleSchema } from '@blocknote/core';
import type { BlockNoteChanges, BlockNoteDocument, TransformOptions } from './types.js';

/**
 * Applies BlockNote editor changes to an Automerge document in a minimal way.
 * This function transforms BlockNote block changes into precise Automerge operations
 * to keep document size small.
 * 
 * @param doc - The current Automerge document
 * @param changes - The changes from BlockNote's getChanges()
 * @param options - Transform options
 * @returns The updated Automerge document
 */
export function applyBlockNoteChanges<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: Automerge.Doc<BlockNoteDocument<BSchema, ISchema, SSchema>>,
  changes: BlockNoteChanges<BSchema, ISchema, SSchema>,
  options: TransformOptions = {}
): Automerge.Doc<BlockNoteDocument<BSchema, ISchema, SSchema>> {
  const { optimize = true } = options;

  // If no changes, return the document as-is
  if (!changes || changes.length === 0) {
    return doc;
  }

  // Filter out no-op changes if optimization is enabled
  let processedChanges = changes;
  if (optimize) {
    processedChanges = optimizeChanges(changes);
  }

  // Apply changes to the document
  return Automerge.change(doc, (d) => {
    for (const change of processedChanges) {
      applyChange(d, change);
    }
  });
}

/**
 * Applies a single BlockNote change to the document
 */
function applyChange<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: BlockNoteDocument<BSchema, ISchema, SSchema>,
  change: BlockNoteChanges<BSchema, ISchema, SSchema>[0]
): void {
  const { type, block } = change;

  switch (type) {
    case 'insert':
      handleInsert(doc, block);
      break;
    case 'delete':
      handleDelete(doc, block);
      break;
    case 'update':
      handleUpdate(doc, block, change.prevBlock);
      break;
    case 'move':
      handleMove(doc, block, change.prevBlock);
      break;
  }
}

/**
 * Handles block insertion
 * Note: Currently appends to the end. For production use with collaborative editing,
 * you would need to determine the correct insertion position based on block relationships
 * or use BlockNote's internal ordering system.
 */
function handleInsert<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: BlockNoteDocument<BSchema, ISchema, SSchema>,
  block: Block<BSchema, ISchema, SSchema>
): void {
  doc.blocks.push(block);
}

/**
 * Handles block deletion
 */
function handleDelete<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: BlockNoteDocument<BSchema, ISchema, SSchema>,
  block: Block<BSchema, ISchema, SSchema>
): void {
  const index = doc.blocks.findIndex((b) => b.id === block.id);
  if (index !== -1) {
    doc.blocks.splice(index, 1);
  }
}

/**
 * Handles block update with surgical precision
 * Only updates fields that have actually changed to minimize document growth
 */
function handleUpdate<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: BlockNoteDocument<BSchema, ISchema, SSchema>,
  block: Block<BSchema, ISchema, SSchema>,
  prevBlock?: Block<BSchema, ISchema, SSchema>
): void {
  const index = doc.blocks.findIndex((b) => b.id === block.id);
  if (index === -1) {
    return;
  }

  const currentBlock = doc.blocks[index] as any;
  const newBlock = block as any;
  const oldBlock = prevBlock as any;

  // If no previous block for comparison, replace entire block
  if (!oldBlock) {
    doc.blocks[index] = block;
    return;
  }

  // Surgically update only changed fields
  for (const key in newBlock) {
    if (key === 'id') continue; // Never update ID

    // Check if this field has changed
    if (!deepEqual(newBlock[key], oldBlock[key])) {
      // Special handling for content array (most common case for text edits)
      if (key === 'content' && Array.isArray(newBlock[key]) && Array.isArray(oldBlock[key])) {
        updateContentArray(currentBlock[key], newBlock[key], oldBlock[key]);
      } else {
        // Update the field
        currentBlock[key] = newBlock[key];
      }
    }
  }

  // Handle removed fields
  for (const key in oldBlock) {
    if (!(key in newBlock) && key !== 'id') {
      delete currentBlock[key];
    }
  }
}

/**
 * Surgically updates a content array, only changing items that differ
 */
function updateContentArray(current: any[], newContent: any[], oldContent: any[]): void {
  // If lengths differ significantly, just replace the whole array
  if (Math.abs(newContent.length - oldContent.length) > 3) {
    current.length = 0;
    current.push(...newContent);
    return;
  }

  // Update existing items and add new ones
  for (let i = 0; i < newContent.length; i++) {
    if (i < current.length) {
      // Update existing item only if changed
      const oldItem = i < oldContent.length ? oldContent[i] : null;
      if (!oldItem || !deepEqual(newContent[i], oldItem)) {
        // For text content items, update only the text field if that's what changed
        if (
          newContent[i]?.type === 'text' &&
          oldItem?.type === 'text' &&
          newContent[i].type === oldItem.type &&
          deepEqual(newContent[i].styles, oldItem.styles)
        ) {
          // Only text changed, use surgical string update
          updateTextFieldSurgically(current[i], 'text', oldItem.text, newContent[i].text);
        } else {
          // Replace entire item
          current[i] = newContent[i];
        }
      }
    } else {
      // Add new item
      current.push(newContent[i]);
    }
  }

  // Remove extra items
  if (current.length > newContent.length) {
    current.splice(newContent.length);
  }
}

/**
 * Updates a text field using Automerge.splice operations
 * Only modifies the parts of the string that have changed using minimal splice operations
 */
function updateTextFieldSurgically(obj: any, fieldName: string, oldText: string, newText: string): void {
  if (oldText === newText) {
    return; // No change needed
  }

  // If the field doesn't exist or is not a string, just set it
  if (typeof obj[fieldName] !== 'string') {
    obj[fieldName] = newText;
    return;
  }

  // Find common prefix
  let prefixLen = 0;
  const minLen = Math.min(oldText.length, newText.length);
  while (prefixLen < minLen && oldText[prefixLen] === newText[prefixLen]) {
    prefixLen++;
  }

  // Find common suffix (after the prefix)
  let suffixLen = 0;
  while (
    suffixLen < minLen - prefixLen &&
    oldText[oldText.length - 1 - suffixLen] === newText[newText.length - 1 - suffixLen]
  ) {
    suffixLen++;
  }

  // Calculate the changed region
  const deleteStart = prefixLen;
  const deleteCount = oldText.length - prefixLen - suffixLen;
  const insertText = newText.substring(prefixLen, newText.length - suffixLen);

  // Apply the minimal change using Automerge.splice
  if (deleteCount > 0 && insertText.length > 0) {
    // Replace: delete and insert
    Automerge.splice(obj, [fieldName], deleteStart, deleteCount, insertText);
  } else if (deleteCount > 0) {
    // Only deletion
    Automerge.splice(obj, [fieldName], deleteStart, deleteCount);
  } else if (insertText.length > 0) {
    // Only insertion
    Automerge.splice(obj, [fieldName], deleteStart, 0, insertText);
  }
  // else: no change needed (shouldn't happen due to early return)
}

/**
 * Handles block move
 * Note: Currently removes and appends. For production use with collaborative editing,
 * you would need to determine the correct new position based on block relationships.
 */
function handleMove<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  doc: BlockNoteDocument<BSchema, ISchema, SSchema>,
  block: Block<BSchema, ISchema, SSchema>,
  prevBlock?: Block<BSchema, ISchema, SSchema>
): void {
  // Remove from old position
  const oldIndex = doc.blocks.findIndex((b) => b.id === block.id);
  if (oldIndex !== -1) {
    doc.blocks.splice(oldIndex, 1);
  }

  // Insert at new position (append for now)
  doc.blocks.push(block);
}

/**
 * Deep equality check without JSON.stringify
 * Handles objects, arrays, and primitive values
 */
function deepEqual(a: any, b: any): boolean {
  // Same reference or both null/undefined
  if (a === b) return true;

  // One is null/undefined, the other isn't
  if (a == null || b == null) return false;

  // Different types
  if (typeof a !== typeof b) return false;

  // Primitive types (already checked equality above)
  if (typeof a !== 'object') return false;

  // Arrays
  if (Array.isArray(a)) {
    if (!Array.isArray(b)) return false;
    if (a.length !== b.length) return false;
    
    for (let i = 0; i < a.length; i++) {
      if (!deepEqual(a[i], b[i])) return false;
    }
    return true;
  }

  // One is array, other is not
  if (Array.isArray(b)) return false;

  // Objects
  const keysA = Object.keys(a);
  const keysB = Object.keys(b);

  if (keysA.length !== keysB.length) return false;

  for (const key of keysA) {
    if (!keysB.includes(key)) return false;
    if (!deepEqual(a[key], b[key])) return false;
  }

  return true;
}

/**
 * Checks if two blocks are equal (alias for deepEqual for backward compatibility)
 */
function blocksEqual<T>(a: T, b: T): boolean {
  return deepEqual(a, b);
}

/**
 * Optimizes changes by filtering out redundant operations
 */
function optimizeChanges<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  changes: BlockNoteChanges<BSchema, ISchema, SSchema>
): BlockNoteChanges<BSchema, ISchema, SSchema> {
  // Filter out no-op updates where block hasn't actually changed
  return changes.filter((change) => {
    if (change.type === 'update' && change.prevBlock) {
      // Check if the block actually changed
      return !blocksEqual(change.block, change.prevBlock);
    }
    return true;
  });
}

/**
 * Creates a new BlockNote document
 */
export function createBlockNoteDocument<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
>(
  initialBlocks: Block<BSchema, ISchema, SSchema>[] = []
): Automerge.Doc<BlockNoteDocument<BSchema, ISchema, SSchema>> {
  return Automerge.from<BlockNoteDocument<BSchema, ISchema, SSchema>>({
    blocks: initialBlocks,
  });
}

/**
 * Gets the size of an Automerge document in bytes
 */
export function getDocumentSize<T>(doc: Automerge.Doc<T>): number {
  return Automerge.save(doc).byteLength;
}

/**
 * Compares document sizes to measure growth
 */
export function compareDocumentSizes<T>(
  oldDoc: Automerge.Doc<T>,
  newDoc: Automerge.Doc<T>
): { oldSize: number; newSize: number; growth: number; growthPercent: number } {
  const oldSize = getDocumentSize(oldDoc);
  const newSize = getDocumentSize(newDoc);
  const growth = newSize - oldSize;
  const growthPercent = oldSize > 0 ? (growth / oldSize) * 100 : 0;

  return {
    oldSize,
    newSize,
    growth,
    growthPercent,
  };
}
