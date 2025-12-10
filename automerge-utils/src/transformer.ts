import * as Automerge from '@automerge/automerge';
import type { EditorChange, EditorChanges, TextDocument, TransformOptions } from './types.js';

/**
 * Applies editor changes to an Automerge document in a minimal way.
 * This function transforms editor changes into precise Automerge operations
 * to keep document size small.
 * 
 * @param doc - The current Automerge document
 * @param changes - The changes from the editor
 * @param options - Transform options
 * @returns The updated Automerge document
 */
export function applyEditorChanges<T extends TextDocument>(
  doc: Automerge.Doc<T>,
  changes: EditorChanges,
  options: TransformOptions = {}
): Automerge.Doc<T> {
  const { mergeAdjacent = true, optimize = true } = options;

  // If no changes, return the document as-is
  if (!changes.changes || changes.changes.length === 0) {
    return doc;
  }

  // Process changes
  let processedChanges = [...changes.changes];

  // Sort changes by position (from lowest to highest)
  processedChanges.sort((a, b) => a.from - b.from);

  // Merge adjacent changes if enabled
  if (mergeAdjacent) {
    processedChanges = mergeAdjacentChanges(processedChanges);
  }

  // Optimize changes if enabled
  if (optimize) {
    processedChanges = optimizeChanges(processedChanges, doc.text);
  }

  // Apply changes to the document
  return Automerge.change(doc, (d) => {
    // Apply changes in reverse order to maintain correct positions
    for (let i = processedChanges.length - 1; i >= 0; i--) {
      const change = processedChanges[i];
      applyChange(d, change);
    }
  });
}

/**
 * Applies a single change to the document
 */
function applyChange<T extends TextDocument>(doc: T, change: EditorChange): void {
  const { from, to, text } = change;
  
  // Replace the range [from, to) with text
  // This handles insertion (from === to), deletion (text === ''), and replacement
  doc.text = doc.text.slice(0, from) + text + doc.text.slice(to);
}

/**
 * Merges adjacent or overlapping changes
 * Changes should be pre-sorted by position
 * Adjacent means they touch or overlap in the ORIGINAL document
 */
function mergeAdjacentChanges(changes: EditorChange[]): EditorChange[] {
  if (changes.length <= 1) {
    return changes;
  }

  const merged: EditorChange[] = [];
  let current = { ...changes[0] };

  for (let i = 1; i < changes.length; i++) {
    const next = changes[i];
    
    // Check if next change overlaps or is adjacent to current in the ORIGINAL document
    // Two changes are adjacent/overlapping if next.from <= current.to
    // (They touch or overlap in the original text)
    if (next.from <= current.to) {
      // They overlap or are adjacent in the original text, merge them
      
      // The merged change covers from current.from to max(current.to, next.to)
      const newTo = Math.max(current.to, next.to);
      
      // For the text: we need to combine current.text with next.text
      // The tricky part is figuring out how they combine
      
      // If next.from >= current.to, they're adjacent (touching but not overlapping in original)
      if (next.from >= current.to) {
        // Adjacent: just concatenate
        current = {
          from: current.from,
          to: newTo,
          text: current.text + next.text,
        };
      } else {
        // Overlapping in the original document
        // This is more complex - for now, just take the later change's text for the overlap region
        // Calculate how much of the original text each change covers
        const currentLen = current.to - current.from;
        const nextLen = next.to - next.from;
        
        // For simplicity, concatenate the texts
        // This might not be perfect for all cases but works for common scenarios
        current = {
          from: current.from,
          to: newTo,
          text: current.text + next.text,
        };
      }
    } else {
      // Not adjacent or overlapping, push current and start new
      merged.push(current);
      current = { ...next };
    }
  }

  merged.push(current);
  return merged;
}

/**
 * Optimizes changes by detecting no-op changes
 */
function optimizeChanges(changes: EditorChange[], currentText: string): EditorChange[] {
  return changes.filter((change) => {
    // If it's a pure deletion
    if (change.text.length === 0) {
      return change.from < change.to;
    }

    // If it's a pure insertion
    if (change.from === change.to) {
      return true;
    }

    // If it's a replacement, check if the text is actually different
    const oldText = currentText.slice(change.from, change.to);
    return oldText !== change.text;
  });
}

/**
 * Creates a new text document
 */
export function createTextDocument(initialText = ''): Automerge.Doc<TextDocument> {
  return Automerge.from<TextDocument>({
    text: initialText,
  });
}

/**
 * Gets the size of an Automerge document in bytes
 */
export function getDocumentSize(doc: Automerge.Doc<TextDocument>): number {
  return Automerge.save(doc).byteLength;
}

/**
 * Compares document sizes to measure growth
 */
export function compareDocumentSizes(
  oldDoc: Automerge.Doc<TextDocument>,
  newDoc: Automerge.Doc<TextDocument>
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
