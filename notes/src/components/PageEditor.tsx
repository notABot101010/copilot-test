import { useEffect } from 'preact/hooks';
import type { JSX } from 'preact';
import { useSignal } from '@preact/signals';
import type { DragEndEvent } from '@dnd-kit/core';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { TextInput, ScrollArea, ActionIcon } from '@mantine/core';
import { BlockComponent } from './BlockComponent';
import { CommandPalette } from './CommandPalette';
import {
  currentPage,
  updatePage,
  createBlock,
  reorderBlocks,
  openCommandPalette,
  isSidebarOpen,
  toggleSidebar,
  focusedBlockId,
  setFocusedBlock,
} from '../store';

export function PageEditor() {
  const titleInputRef = useSignal<HTMLInputElement | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  useEffect(() => {
    if (currentPage.value?.blocks.length === 0 && currentPage.value) {
      createBlock(currentPage.value.id, 'text');
    }
  }, [currentPage.value?.id]);

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id && currentPage.value) {
      const oldIndex = currentPage.value.blocks.findIndex(
        (block) => block.id === active.id
      );
      const newIndex = currentPage.value.blocks.findIndex(
        (block) => block.id === over.id
      );
      reorderBlocks(currentPage.value.id, oldIndex, newIndex);
    }
  };

  const handleBlockCreated = (blockId: string) => {
    setFocusedBlock(blockId);
  };

  const handleAddBlock = () => {
    if (!currentPage.value) return;
    const lastBlockIndex = currentPage.value.blocks.length - 1;
    const rect = document.querySelector('.page-content')?.getBoundingClientRect();
    if (rect) {
      openCommandPalette(rect.left + 60, rect.bottom - 100, lastBlockIndex);
    }
  };

  const handleTitleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const firstBlock = currentPage.value?.blocks[0];
      if (firstBlock) {
        setFocusedBlock(firstBlock.id);
      }
    } else if (e.key === '/' && currentPage.value) {
      e.preventDefault();
      const rect = (e.target as HTMLElement).getBoundingClientRect();
      openCommandPalette(rect.left, rect.bottom + 4, -1);
    }
  };

  // Handle clicking below the last block to create a new block
  const handleContentAreaClick = () => {
    if (!currentPage.value) return;
    
    const page = currentPage.value;
    const lastBlockIndex = page.blocks.length - 1;
    const newBlock = createBlock(page.id, 'text', lastBlockIndex);
    setFocusedBlock(newBlock.id);
  };

  if (!currentPage.value) {
    return (
      <div className="flex h-screen flex-1 items-center justify-center">
        <div className="text-center">
          <p className="mb-4 text-xl text-zinc-400">No page selected</p>
          <p className="text-sm text-zinc-500">Select a page from the sidebar or create a new one</p>
        </div>
      </div>
    );
  }

  const page = currentPage.value;

  return (
    <div className="flex h-screen flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b border-zinc-700 px-4 py-2">
        <ActionIcon
          variant="subtle"
          color="gray"
          onClick={toggleSidebar}
          aria-label={isSidebarOpen.value ? "Close sidebar" : "Open sidebar"}
        >
          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fillRule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clipRule="evenodd" />
          </svg>
        </ActionIcon>
        <div className="flex items-center gap-2">
          <span className="text-lg">{page.icon || '📄'}</span>
          <span className="text-sm text-zinc-300">{page.title || 'Untitled'}</span>
        </div>
      </div>

      {/* Page content */}
      <ScrollArea className="flex-1">
        <div className="page-content mx-auto max-w-3xl px-12 py-8">
          {/* Icon and Title */}
          <div className="mb-8">
            <button className="mb-4 text-5xl transition-transform hover:scale-110">
              {page.icon || '📄'}
            </button>
            <TextInput
              ref={(el: HTMLInputElement | null) => (titleInputRef.value = el)}
              variant="unstyled"
              placeholder="Untitled"
              value={page.title}
              onChange={(e: JSX.TargetedEvent<HTMLInputElement>) => updatePage(page.id, { title: e.currentTarget.value })}
              onKeyDown={handleTitleKeyDown}
              className="text-4xl font-bold"
              styles={{
                input: {
                  fontSize: '2.25rem',
                  fontWeight: 700,
                  padding: 0,
                  height: 'auto',
                },
              }}
            />
          </div>

          {/* Blocks */}
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={page.blocks.map((b) => b.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="space-y-1">
                {page.blocks.map((block) => (
                  <BlockComponent
                    key={block.id}
                    block={block}
                    pageId={page.id}
                    autoFocus={focusedBlockId.value === block.id}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>

          {/* Add block button when empty */}
          {page.blocks.length === 0 && (
            <button
              className="flex w-full items-center gap-2 rounded px-2 py-3 text-sm text-zinc-500 transition-colors hover:bg-zinc-800 hover:text-zinc-300"
              onClick={handleAddBlock}
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
              </svg>
              Click here or press '/' to add a block
            </button>
          )}

          {/* Clickable area below blocks to add new block */}
          {page.blocks.length > 0 && (
            <div 
              className="min-h-48 cursor-text" 
              onClick={handleContentAreaClick}
            />
          )}
        </div>
      </ScrollArea>

      <CommandPalette onBlockCreated={handleBlockCreated} />
    </div>
  );
}
