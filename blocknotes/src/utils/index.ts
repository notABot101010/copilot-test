export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

export function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

export function getBlockTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    text: 'Text',
    heading1: 'Heading 1',
    heading2: 'Heading 2',
    heading3: 'Heading 3',
    bulletList: 'Bullet List',
    numberedList: 'Numbered List',
    todoList: 'To-do List',
    image: 'Image',
    table: 'Table',
    pageLink: 'Link to Page',
    divider: 'Divider',
    quote: 'Quote',
    database: 'Database',
  };
  return labels[type] || type;
}
