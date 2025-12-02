import { signal, computed } from '@preact/signals';

export interface Document {
  id: string;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
}

const STORAGE_KEY = 'markdown-editor-documents';

function loadDocuments(): Document[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored);
    }
  } catch (err) {
    console.error('Failed to load documents:', err);
  }
  return [];
}

function saveDocuments(docs: Document[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(docs));
  } catch (err) {
    console.error('Failed to save documents:', err);
  }
}

export const documents = signal<Document[]>(loadDocuments());
export const activeDocumentId = signal<string | null>(null);

export const activeDocument = computed(() => {
  const id = activeDocumentId.value;
  if (!id) return null;
  return documents.value.find((doc) => doc.id === id) || null;
});

export const sortedDocuments = computed(() => {
  return [...documents.value].sort((docA, docB) => docB.updatedAt - docA.updatedAt);
});

export function generateId(): string {
  return `doc_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

export function createDocument(title: string = 'Untitled'): Document {
  const now = Date.now();
  const newDoc: Document = {
    id: generateId(),
    title,
    content: '',
    createdAt: now,
    updatedAt: now,
  };
  
  documents.value = [...documents.value, newDoc];
  saveDocuments(documents.value);
  activeDocumentId.value = newDoc.id;
  
  return newDoc;
}

export function updateDocument(id: string, updates: Partial<Pick<Document, 'title' | 'content'>>) {
  const now = Date.now();
  documents.value = documents.value.map((doc) =>
    doc.id === id
      ? { ...doc, ...updates, updatedAt: now }
      : doc
  );
  saveDocuments(documents.value);
}

export function deleteDocument(id: string) {
  documents.value = documents.value.filter((doc) => doc.id !== id);
  saveDocuments(documents.value);
  
  if (activeDocumentId.value === id) {
    const remaining = documents.value;
    activeDocumentId.value = remaining.length > 0 ? remaining[0].id : null;
  }
}

export function setActiveDocument(id: string | null) {
  activeDocumentId.value = id;
}

export function convertToMarkdown(html: string): string {
  // Use a temporary DOM element to properly parse and extract text
  // This is safer than regex-based HTML parsing
  const tempDiv = document.createElement('div');
  tempDiv.innerHTML = html;
  
  // Process the DOM tree to convert to markdown
  function processNode(node: Node): string {
    if (node.nodeType === Node.TEXT_NODE) {
      return node.textContent || '';
    }
    
    if (node.nodeType !== Node.ELEMENT_NODE) {
      return '';
    }
    
    const element = node as HTMLElement;
    const tagName = element.tagName.toLowerCase();
    const children = Array.from(element.childNodes).map(processNode).join('');
    
    switch (tagName) {
      case 'h1':
        return `# ${children}\n\n`;
      case 'h2':
        return `## ${children}\n\n`;
      case 'h3':
        return `### ${children}\n\n`;
      case 'strong':
      case 'b':
        return `**${children}**`;
      case 'em':
      case 'i':
        return `*${children}*`;
      case 's':
      case 'strike':
      case 'del':
        return `~~${children}~~`;
      case 'code':
        return `\`${children}\``;
      case 'pre':
        return `\`\`\`\n${children}\n\`\`\`\n\n`;
      case 'ul':
        return children + '\n';
      case 'ol':
        let counter = 0;
        return Array.from(element.children)
          .map(child => {
            if (child.tagName.toLowerCase() === 'li') {
              counter++;
              return `${counter}. ${processNode(child).replace(/^- /, '')}\n`;
            }
            return processNode(child);
          })
          .join('') + '\n';
      case 'li':
        return `- ${children}\n`;
      case 'blockquote':
        return children.split('\n').map(line => `> ${line}`).join('\n') + '\n\n';
      case 'p':
        return `${children}\n\n`;
      case 'br':
        return '\n';
      case 'hr':
        return '---\n\n';
      default:
        return children;
    }
  }
  
  let markdown = processNode(tempDiv);
  
  // Clean up excessive newlines
  markdown = markdown.replace(/\n{3,}/g, '\n\n');
  
  return markdown.trim();
}
