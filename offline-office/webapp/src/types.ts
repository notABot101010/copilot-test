export interface Document {
  id: string;
  title: string;
  doc_type: 'document' | 'presentation';
  created_at: number;
  updated_at: number;
}

export interface MarkdownContent {
  text: string;
}

export interface Slide {
  id: string;
  title: string;
  content: string;
}

export interface PresentationContent {
  slides: Slide[];
}
