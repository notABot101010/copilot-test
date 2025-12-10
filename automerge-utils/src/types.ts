/**
 * Represents a change in the editor
 */
export interface EditorChange {
  from: number;
  to: number;
  text: string;
}

/**
 * Result from editor's getChanges() function
 */
export interface EditorChanges {
  changes: EditorChange[];
}

/**
 * Type for text content in Automerge document
 */
export interface TextDocument extends Record<string, unknown> {
  text: string;
}

/**
 * Options for transforming changes
 */
export interface TransformOptions {
  /**
   * Whether to merge adjacent changes
   * @default true
   */
  mergeAdjacent?: boolean;
  
  /**
   * Whether to apply optimizations to reduce document size
   * @default true
   */
  optimize?: boolean;
}
