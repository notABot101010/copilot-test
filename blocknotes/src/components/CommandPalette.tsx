import { useEffect, useRef } from 'preact/hooks';
import { useSignal } from '@preact/signals';
import { Paper, Text, UnstyledButton } from '@mantine/core';
import type { Command } from '../types';
import {
  isCommandPaletteOpen,
  commandPalettePosition,
  commandPaletteBlockIndex,
  closeCommandPalette,
  createBlock,
  currentPageId,
} from '../store';

const commands: Command[] = [
  { id: 'text', name: 'Text', description: 'Plain text paragraph', icon: '📝', blockType: 'text' },
  { id: 'heading1', name: 'Heading 1', description: 'Large heading', icon: 'H1', blockType: 'heading1' },
  { id: 'heading2', name: 'Heading 2', description: 'Medium heading', icon: 'H2', blockType: 'heading2' },
  { id: 'heading3', name: 'Heading 3', description: 'Small heading', icon: 'H3', blockType: 'heading3' },
  { id: 'bulletList', name: 'Bullet List', description: 'Unordered list', icon: '•', blockType: 'bulletList' },
  { id: 'numberedList', name: 'Numbered List', description: 'Ordered list', icon: '1.', blockType: 'numberedList' },
  { id: 'todoList', name: 'To-do List', description: 'Task list with checkboxes', icon: '☑', blockType: 'todoList' },
  { id: 'image', name: 'Image', description: 'Upload or embed an image', icon: '🖼️', blockType: 'image' },
  { id: 'table', name: 'Table', description: 'Add a table', icon: '📊', blockType: 'table' },
  { id: 'pageLink', name: 'Link to Page', description: 'Link to another page', icon: '🔗', blockType: 'pageLink' },
  { id: 'divider', name: 'Divider', description: 'Visual separator', icon: '—', blockType: 'divider' },
  { id: 'quote', name: 'Quote', description: 'Block quote', icon: '"', blockType: 'quote' },
  { id: 'database', name: 'Database', description: 'Table or Kanban board', icon: '🗃️', blockType: 'database' },
];

interface CommandPaletteProps {
  onBlockCreated?: (blockId: string) => void;
}

export function CommandPalette({ onBlockCreated }: CommandPaletteProps) {
  const searchQuery = useSignal('');
  const selectedIndex = useSignal(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredCommands = commands.filter(
    (cmd) =>
      cmd.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      cmd.description.toLowerCase().includes(searchQuery.value.toLowerCase())
  );

  useEffect(() => {
    if (isCommandPaletteOpen.value && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isCommandPaletteOpen.value]);

  useEffect(() => {
    selectedIndex.value = 0;
  }, [searchQuery.value]);

  const handleSelectCommand = (command: Command) => {
    if (!currentPageId.value) return;
    const block = createBlock(currentPageId.value, command.blockType, commandPaletteBlockIndex.value ?? undefined);
    closeCommandPalette();
    searchQuery.value = '';
    onBlockCreated?.(block.id);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, filteredCommands.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    } else if (e.key === 'Enter' && filteredCommands[selectedIndex.value]) {
      e.preventDefault();
      handleSelectCommand(filteredCommands[selectedIndex.value]);
    } else if (e.key === 'Escape') {
      closeCommandPalette();
      searchQuery.value = '';
    }
  };

  if (!isCommandPaletteOpen.value) return null;

  return (
    <div 
      className="fixed inset-0 z-50" 
      onClick={() => {
        closeCommandPalette();
        searchQuery.value = '';
      }}
    >
      <Paper
        shadow="lg"
        className="absolute max-h-80 w-72 overflow-hidden bg-zinc-800 border border-zinc-700"
        style={{
          left: Math.min(commandPalettePosition.value.x, window.innerWidth - 300),
          top: Math.min(commandPalettePosition.value.y, window.innerHeight - 340),
        }}
        onClick={(e: MouseEvent) => e.stopPropagation()}
      >
        <div className="border-b border-zinc-700 p-2">
          <input
            ref={inputRef}
            type="text"
            placeholder="Filter blocks..."
            className="w-full bg-transparent px-2 py-1 text-sm text-white outline-none placeholder:text-zinc-500"
            value={searchQuery.value}
            onInput={(e) => (searchQuery.value = (e.target as HTMLInputElement).value)}
            onKeyDown={handleKeyDown}
          />
        </div>
        <div className="max-h-64 overflow-y-auto p-1">
          {filteredCommands.map((command, index) => (
            <UnstyledButton
              key={command.id}
              className={`flex w-full items-center gap-3 rounded px-2 py-1.5 transition-colors ${
                selectedIndex.value === index
                  ? 'bg-zinc-700 text-white'
                  : 'text-zinc-300 hover:bg-zinc-700/50'
              }`}
              onClick={() => handleSelectCommand(command)}
            >
              <span className="flex h-8 w-8 items-center justify-center rounded bg-zinc-700 text-lg">
                {command.icon}
              </span>
              <div className="text-left">
                <Text size="sm" fw={500}>
                  {command.name}
                </Text>
                <Text size="xs" c="dimmed">
                  {command.description}
                </Text>
              </div>
            </UnstyledButton>
          ))}
          {filteredCommands.length === 0 && (
            <Text size="sm" c="dimmed" className="p-4 text-center">
              No blocks found
            </Text>
          )}
        </div>
      </Paper>
    </div>
  );
}
