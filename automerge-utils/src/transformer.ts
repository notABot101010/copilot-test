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
 * Handles block update
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
  if (index !== -1) {
    // Replace the block at the found index
    doc.blocks[index] = block;
  }
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
 * Simple deep equality check for blocks
 * For production, consider using a library like fast-deep-equal
 */
function blocksEqual<T>(a: T, b: T): boolean {
  // For simple comparison, use JSON.stringify
  // This is acceptable for the current use case but could be optimized
  return JSON.stringify(a) === JSON.stringify(b);
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
