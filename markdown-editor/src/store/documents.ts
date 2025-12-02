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
  let markdown = html;
  
  // Headers
  markdown = markdown.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
  markdown = markdown.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
  markdown = markdown.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
  
  // Bold and italic
  markdown = markdown.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
  markdown = markdown.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*');
  markdown = markdown.replace(/<s[^>]*>(.*?)<\/s>/gi, '~~$1~~');
  
  // Code
  markdown = markdown.replace(/<code[^>]*>(.*?)<\/code>/gi, '`$1`');
  markdown = markdown.replace(/<pre[^>]*><code[^>]*>(.*?)<\/code><\/pre>/gis, '```\n$1\n```\n\n');
  
  // Lists
  markdown = markdown.replace(/<ul[^>]*>/gi, '');
  markdown = markdown.replace(/<\/ul>/gi, '\n');
  markdown = markdown.replace(/<ol[^>]*>/gi, '');
  markdown = markdown.replace(/<\/ol>/gi, '\n');
  markdown = markdown.replace(/<li[^>]*>(.*?)<\/li>/gi, '- $1\n');
  
  // Blockquote
  markdown = markdown.replace(/<blockquote[^>]*>(.*?)<\/blockquote>/gis, (_, content) => {
    return content.split('\n').map((line: string) => `> ${line}`).join('\n') + '\n\n';
  });
  
  // Paragraphs and line breaks
  markdown = markdown.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n');
  markdown = markdown.replace(/<br\s*\/?>/gi, '\n');
  
  // Horizontal rule
  markdown = markdown.replace(/<hr[^>]*\/?>/gi, '---\n\n');
  
  // Remove remaining HTML tags
  markdown = markdown.replace(/<[^>]+>/g, '');
  
  // Clean up excessive newlines
  markdown = markdown.replace(/\n{3,}/g, '\n\n');
  
  // Decode HTML entities
  markdown = markdown.replace(/&amp;/g, '&');
  markdown = markdown.replace(/&lt;/g, '<');
  markdown = markdown.replace(/&gt;/g, '>');
  markdown = markdown.replace(/&quot;/g, '"');
  markdown = markdown.replace(/&#39;/g, "'");
  markdown = markdown.replace(/&nbsp;/g, ' ');
  
  return markdown.trim();
}
