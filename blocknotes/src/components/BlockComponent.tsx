import { useRef } from 'preact/hooks';
import type { JSX } from 'preact';
import { useSignal } from '@preact/signals';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { ActionIcon, TextInput, Menu } from '@mantine/core';
import { RichTextEditor } from './RichTextEditor';
import { TableBlock } from './TableBlock';
import { DatabaseBlock } from './DatabaseBlock';
import { deleteBlock, updateBlock, pages, openCommandPalette, currentPage } from '../store';
import type { Block } from '../types';

interface BlockComponentProps {
  block: Block;
  pageId: string;
  autoFocus?: boolean;
}

export function BlockComponent({ block, pageId, autoFocus }: BlockComponentProps) {
  const isHovered = useSignal(false);
  const blockRef = useRef<HTMLDivElement>(null);

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: block.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const handleDelete = () => {
    deleteBlock(pageId, block.id);
  };

  const handleAddBlockBelow = () => {
    const rect = blockRef.current?.getBoundingClientRect();
    if (rect) {
      const page = currentPage.value;
      const blockIndex = page?.blocks.findIndex((b) => b.id === block.id) ?? -1;
      openCommandPalette(rect.left, rect.bottom + 4, blockIndex);
    }
  };

  const renderBlockContent = () => {
    switch (block.type) {
      case 'divider':
        return <hr className="my-4 border-zinc-600" />;

      case 'image':
        return (
          <ImageBlock
            block={block}
            pageId={pageId}
            onUpdate={(url: string) =>
              updateBlock(pageId, block.id, {
                properties: { ...block.properties, imageUrl: url },
              })
            }
          />
        );

      case 'table':
        return <TableBlock block={block} pageId={pageId} />;

      case 'database':
        return <DatabaseBlock block={block} pageId={pageId} />;

      case 'pageLink':
        return <PageLinkBlock block={block} pageId={pageId} />;

      default:
        return <RichTextEditor block={block} pageId={pageId} autoFocus={autoFocus} />;
    }
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="group relative"
      onMouseEnter={() => (isHovered.value = true)}
      onMouseLeave={() => (isHovered.value = false)}
    >
      <div
        ref={blockRef}
        className={`relative flex items-start gap-1 rounded px-1 py-0.5 transition-colors ${
          isDragging ? 'bg-zinc-700/50' : ''
        }`}
      >
        <div
          className={`flex flex-shrink-0 items-center gap-0.5 pt-1 transition-opacity ${
            isHovered.value ? 'opacity-100' : 'opacity-0'
          }`}
        >
          <ActionIcon
            {...attributes}
            {...listeners}
            variant="subtle"
            color="gray"
            size="sm"
            className="cursor-grab"
            aria-label="Drag block"
          >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="currentColor" viewBox="0 0 16 16">
              <path d="M2 4h2v2H2V4zm4 0h2v2H6V4zm4 0h2v2h-2V4zm-8 4h2v2H2V8zm4 0h2v2H6V8zm4 0h2v2h-2V8zm-8 4h2v2H2v-2zm4 0h2v2H6v-2zm4 0h2v2h-2v-2z" />
            </svg>
          </ActionIcon>

          <Menu shadow="md" width={160}>
            <Menu.Target>
              <ActionIcon variant="subtle" color="gray" size="sm" aria-label="Block options">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="currentColor" viewBox="0 0 16 16">
                  <path d="M8 4a1.5 1.5 0 100-3 1.5 1.5 0 000 3zm0 5.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3zm0 5.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
                </svg>
              </ActionIcon>
            </Menu.Target>

            <Menu.Dropdown>
              <Menu.Item
                leftSection={
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
                  </svg>
                }
                onClick={handleAddBlockBelow}
              >
                Add block below
              </Menu.Item>
              <Menu.Divider />
              <Menu.Item
                color="red"
                leftSection={
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                }
                onClick={handleDelete}
              >
                Delete
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
        </div>

        <div className="min-w-0 flex-1">{renderBlockContent()}</div>
      </div>
    </div>
  );
}

interface ImageBlockProps {
  block: Block;
  pageId: string;
  onUpdate: (url: string) => void;
}

function ImageBlock({ block, onUpdate }: ImageBlockProps) {
  const isEditing = useSignal(!block.properties?.imageUrl);
  const urlInput = useSignal(block.properties?.imageUrl || '');

  const handleSubmit = () => {
    if (urlInput.value.trim()) {
      onUpdate(urlInput.value.trim());
      isEditing.value = false;
    }
  };

  if (isEditing.value) {
    return (
      <div className="flex items-center gap-2 rounded bg-zinc-800 p-4">
        <TextInput
          placeholder="Paste image URL..."
          className="flex-1"
          value={urlInput.value}
          onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => (urlInput.value = e.currentTarget.value)}
          onKeyDown={(e: JSX.TargetedKeyboardEvent<HTMLInputElement>) => {
            if (e.key === 'Enter') {
              handleSubmit();
            }
          }}
        />
        <ActionIcon onClick={handleSubmit} variant="filled" color="blue">
          <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
          </svg>
        </ActionIcon>
      </div>
    );
  }

  return (
    <div className="group/image relative">
      <img
        src={block.properties?.imageUrl}
        alt={block.properties?.imageAlt || 'Image'}
        className="max-w-full rounded-lg"
      />
      <ActionIcon
        className="absolute right-2 top-2 opacity-0 transition-opacity group-hover/image:opacity-100"
        variant="filled"
        color="dark"
        onClick={() => (isEditing.value = true)}
      >
        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
        </svg>
      </ActionIcon>
    </div>
  );
}

interface PageLinkBlockProps {
  block: Block;
  pageId: string;
}

function PageLinkBlock({ block, pageId }: PageLinkBlockProps) {
  const linkedPageId = block.properties?.linkedPageId;
  const linkedPage = pages.value.find((p) => p.id === linkedPageId);
  const isEditing = useSignal(!linkedPageId);

  if (isEditing.value) {
    return (
      <div className="rounded bg-zinc-800 p-4">
        <p className="mb-2 text-sm text-zinc-400">Select a page to link:</p>
        <div className="space-y-1">
          {pages.value
            .filter((p) => p.id !== pageId)
            .map((page) => (
              <button
                key={page.id}
                className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm transition-colors hover:bg-zinc-700"
                onClick={() => {
                  updateBlock(pageId, block.id, {
                    properties: { ...block.properties, linkedPageId: page.id },
                  });
                  isEditing.value = false;
                }}
              >
                <span>{page.icon || '📄'}</span>
                <span>{page.title || 'Untitled'}</span>
              </button>
            ))}
        </div>
      </div>
    );
  }

  return (
    <a
      href={`#/page/${linkedPageId}`}
      className="flex items-center gap-2 rounded bg-zinc-800 px-3 py-2 text-sm transition-colors hover:bg-zinc-700"
    >
      <span>{linkedPage?.icon || '📄'}</span>
      <span className="text-white">{linkedPage?.title || 'Untitled'}</span>
    </a>
  );
}
