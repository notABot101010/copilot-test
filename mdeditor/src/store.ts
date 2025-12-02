import { signal, computed } from '@preact/signals'
import type { Document } from './types'

// Initial document
const initialDoc: Document = {
  id: '1',
  title: 'Welcome',
  content: '# Welcome to Markdown Editor\n\nStart writing your markdown here...',
  createdAt: new Date(),
  updatedAt: new Date(),
}

// Store documents in local storage
const STORAGE_KEY = 'markdown-editor-docs'

function loadDocuments(): Document[] {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored) {
    const docs = JSON.parse(stored)
    return docs.map((doc: Document) => ({
      ...doc,
      createdAt: new Date(doc.createdAt),
      updatedAt: new Date(doc.updatedAt),
    }))
  }
  return [initialDoc]
}

function saveDocuments(docs: Document[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(docs))
}

// Signals
export const documents = signal<Document[]>(loadDocuments())
export const currentDocId = signal<string>(documents.value[0]?.id || '1')

// Computed values
export const currentDocument = computed(() => {
  return documents.value.find(doc => doc.id === currentDocId.value) || documents.value[0]
})

// Actions
export function createDocument() {
  const newDoc: Document = {
    id: Date.now().toString(),
    title: 'Untitled Document',
    content: '',
    createdAt: new Date(),
    updatedAt: new Date(),
  }
  documents.value = [...documents.value, newDoc]
  currentDocId.value = newDoc.id
  saveDocuments(documents.value)
}

export function updateDocument(id: string, updates: Partial<Document>) {
  documents.value = documents.value.map(doc =>
    doc.id === id
      ? { ...doc, ...updates, updatedAt: new Date() }
      : doc
  )
  saveDocuments(documents.value)
}

export function deleteDocument(id: string) {
  if (documents.value.length === 1) return // Don't delete the last document

  documents.value = documents.value.filter(doc => doc.id !== id)

  if (currentDocId.value === id) {
    currentDocId.value = documents.value[0].id
  }

  saveDocuments(documents.value)
}

export function selectDocument(id: string) {
  currentDocId.value = id
}
