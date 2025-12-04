import { useMemo, useRef } from 'react';
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/mantine";
import type { PartialBlock } from "@blocknote/core";
import "@blocknote/core/fonts/inter.css";
import "@blocknote/mantine/style.css";

import { updatePage } from '../store';
import type { Page, Block as AppBlock, BlockType } from '../types';

interface BlockNoteEditorProps {
  page: Page;
}

export function BlockNoteEditor({ page }: BlockNoteEditorProps) {
  // Keep track of existing block timestamps
  const blockTimestampsRef = useRef<Map<string, { createdAt: number; updatedAt: number }>>(new Map());

  // Parse content from page blocks to BlockNote blocks format
  // Only recalculate when page.id changes (initial load or page switch)
  const initialContent = useMemo(() => {
    // Store timestamps from existing blocks
    page.blocks.forEach((block) => {
      blockTimestampsRef.current.set(block.id, {
        createdAt: block.createdAt,
        updatedAt: block.updatedAt,
      });
    });

    if (page.blocks.length === 0) {
      return undefined;
    }

    // Convert our blocks to BlockNote format
    const blocks: PartialBlock[] = page.blocks.map((block) => {
      switch (block.type) {
        case 'heading1':
          return {
            id: block.id,
            type: 'heading' as const,
            props: { level: 1 as const },
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'heading2':
          return {
            id: block.id,
            type: 'heading' as const,
            props: { level: 2 as const },
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'heading3':
          return {
            id: block.id,
            type: 'heading' as const,
            props: { level: 3 as const },
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'bulletList':
          return {
            id: block.id,
            type: 'bulletListItem' as const,
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'numberedList':
          return {
            id: block.id,
            type: 'numberedListItem' as const,
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'todoList':
          return {
            id: block.id,
            type: 'checkListItem' as const,
            props: { checked: block.properties?.checked ?? false },
            content: block.content ? stripHtml(block.content) : '',
          };
        case 'image':
          if (block.properties?.imageUrl) {
            return {
              id: block.id,
              type: 'image' as const,
              props: {
                url: block.properties.imageUrl,
                caption: block.properties.imageAlt || '',
              },
            };
          }
          return {
            id: block.id,
            type: 'paragraph' as const,
            content: '',
          };
        default:
          return {
            id: block.id,
            type: 'paragraph' as const,
            content: block.content ? stripHtml(block.content) : '',
          };
      }
    });

    return blocks;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page.id]);

  // Create BlockNote editor - useCreateBlockNote already handles memoization internally
  const editor = useCreateBlockNote({
    initialContent,
  });

  // Handle content changes
  const handleChange = () => {
    // Convert BlockNote blocks back to our format
    const blocks = editor.document;
    const now = Date.now();
    
    const newBlocks: AppBlock[] = blocks.map((block) => {
      const content = getBlockContent(block);
      const blockType = mapBlockNoteTypeToOurs(block.type, block.props);
      
      // Preserve existing timestamps or create new ones
      const existingTimestamps = blockTimestampsRef.current.get(block.id);
      const createdAt = existingTimestamps?.createdAt ?? now;
      
      // Update the timestamp cache
      blockTimestampsRef.current.set(block.id, {
        createdAt,
        updatedAt: now,
      });
      
      return {
        id: block.id,
        type: blockType,
        content: content,
        properties: getBlockProperties(block),
        createdAt,
        updatedAt: now,
      };
    });

    updatePage(page.id, { blocks: newBlocks });
  };

  return (
    <div className="blocknote-editor-wrapper min-h-[200px]">
      <BlockNoteView
        editor={editor}
        onChange={handleChange}
        theme="dark"
      />
    </div>
  );
}

// Helper function to strip HTML tags
function stripHtml(html: string): string {
  const div = document.createElement('div');
  div.innerHTML = html;
  return div.textContent || div.innerText || '';
}

// Helper to get text content from a BlockNote block
function getBlockContent(block: { content?: unknown }): string {
  if (!block.content) return '';
  
  if (Array.isArray(block.content)) {
    return block.content
      .map((item: unknown) => {
        if (typeof item === 'object' && item !== null && 'text' in item) {
          return (item as { text: string }).text;
        }
        return '';
      })
      .join('');
  }
  
  if (typeof block.content === 'string') {
    return block.content;
  }
  
  return '';
}

// Map BlockNote block types to our block types
function mapBlockNoteTypeToOurs(type: string, props?: Record<string, unknown>): BlockType {
  switch (type) {
    case 'heading': {
      const level = (props?.level as number) || 1;
      if (level === 1) return 'heading1';
      if (level === 2) return 'heading2';
      return 'heading3';
    }
    case 'bulletListItem':
      return 'bulletList';
    case 'numberedListItem':
      return 'numberedList';
    case 'checkListItem':
      return 'todoList';
    case 'image':
      return 'image';
    case 'table':
      return 'table';
    default:
      return 'text';
  }
}

// Get properties from BlockNote block
function getBlockProperties(block: { type: string; props?: Record<string, unknown>; content?: unknown }): Record<string, unknown> | undefined {
  const props: Record<string, unknown> = {};
  
  if (block.type === 'checkListItem' && block.props) {
    props.checked = block.props.checked || false;
  }
  
  if (block.type === 'image' && block.props) {
    props.imageUrl = block.props.url;
    props.imageAlt = block.props.caption;
  }
  
  return Object.keys(props).length > 0 ? props : undefined;
}
