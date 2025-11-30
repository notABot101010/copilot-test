import * as Automerge from '@automerge/automerge';
import { signal } from '@preact/signals';

export class AutomergeDocumentManager<T> {
  private doc: Automerge.Doc<T>;
  private ws: WebSocket | null = null;
  public content = signal<T | null>(null);
  private documentId: string;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor(documentId: string, initialDoc?: Automerge.Doc<T>) {
    this.documentId = documentId;
    this.doc = initialDoc || Automerge.init<T>();
    this.content.value = this.doc as T;
  }

  async loadFromServer(): Promise<void> {
    try {
      const response = await fetch(`/api/documents/${this.documentId}`);
      if (response.ok) {
        const data = await response.arrayBuffer();
        if (data.byteLength > 0) {
          this.doc = Automerge.load(new Uint8Array(data));
          this.content.value = this.doc as T;
        }
      }
    } catch (err) {
      console.error('Failed to load document:', err);
    }
  }

  connectWebSocket(): void {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.hostname}:8080/api/documents/${this.documentId}/ws`;

    this.ws = new WebSocket(wsUrl);
    this.ws.binaryType = 'arraybuffer';

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const changes = new Uint8Array(event.data);
        const [newDoc] = Automerge.applyChanges(this.doc, [changes]);
        this.doc = newDoc;
        this.content.value = this.doc as T;
      } catch (err) {
        console.error('Failed to apply changes:', err);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket closed');
      this.ws = null;

      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        setTimeout(() => this.connectWebSocket(), delay);
      }
    };
  }

  change(fn: (doc: T) => void): void {
    const newDoc = Automerge.change(this.doc, fn);
    const changes = Automerge.getLastLocalChange(newDoc);

    if (changes) {
      this.doc = newDoc;
      this.content.value = this.doc as T;

      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(changes);
      } else {
        this.syncToServer(changes);
      }
    }
  }

  private async syncToServer(changes: Uint8Array): Promise<void> {
    try {
      await fetch(`/api/documents/${this.documentId}/sync`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          changes: Array.from(changes),
        }),
      });
    } catch (err) {
      console.error('Failed to sync to server:', err);
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  getDoc(): Automerge.Doc<T> {
    return this.doc;
  }
}
