import type { Block, BlockSchema, InlineContentSchema, StyleSchema } from '@blocknote/core';

/**
 * A change from BlockNote's onChange callback
 */
export interface BlockNoteChange<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
> {
  type: 'insert' | 'delete' | 'update' | 'move';
  block: Block<BSchema, ISchema, SSchema>;
  prevBlock?: Block<BSchema, ISchema, SSchema>;
  source: {
    type: 'local' | 'paste' | 'drop' | 'undo' | 'redo' | 'undo-redo' | 'yjs-remote';
  };
  prevParent?: Block<BSchema, ISchema, SSchema>;
  currentParent?: Block<BSchema, ISchema, SSchema>;
}

/**
 * Result from BlockNote editor's getChanges() function
 */
export type BlockNoteChanges<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
> = Array<BlockNoteChange<BSchema, ISchema, SSchema>>;

/**
 * Type for document content in Automerge document
 */
export interface BlockNoteDocument<
  BSchema extends BlockSchema = any,
  ISchema extends InlineContentSchema = any,
  SSchema extends StyleSchema = any
> extends Record<string, unknown> {
  blocks: Block<BSchema, ISchema, SSchema>[];
}

/**
 * Options for transforming changes
 */
export interface TransformOptions {
  /**
   * Whether to apply optimizations to reduce document size
   * @default true
   */
  optimize?: boolean;

  /**
   * Whether to include metadata about the change source
   * @default false
   */
  includeMetadata?: boolean;
}
