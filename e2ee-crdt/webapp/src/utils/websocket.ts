export interface WsOperationMessage {
  type: 'operation';
  document_id: string;
  encrypted_operation: string;
  timestamp: number;
}

export interface WsSubscribeMessage {
  type: 'subscribe';
  document_id: string;
}

export interface WsDocumentCreatedMessage {
  type: 'document_created';
  id: string;
  encrypted_data: string;
  created_at: number;
}

export type WsMessage = WsOperationMessage | WsSubscribeMessage | WsDocumentCreatedMessage;

export type OperationHandler = (message: WsOperationMessage) => void;
export type DocumentCreatedHandler = (message: WsDocumentCreatedMessage) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private handlers: Set<OperationHandler> = new Set();
  private documentCreatedHandlers: Set<DocumentCreatedHandler> = new Set();
  private reconnectTimer: number | null = null;
  private reconnectDelay = 1000;
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectDelay = 1000;
    };

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsMessage;
        if (message.type === 'operation') {
          this.handlers.forEach((handler) => handler(message));
        } else if (message.type === 'document_created') {
          this.documentCreatedHandlers.forEach((handler) => handler(message));
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.scheduleReconnect();
    };
  }

  disconnect(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer !== null) {
      return;
    }

    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
      this.connect();
    }, this.reconnectDelay);
  }

  send(message: WsMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  subscribe(documentId: string): void {
    this.send({
      type: 'subscribe',
      document_id: documentId,
    });
  }

  sendOperation(
    documentId: string,
    encryptedOperation: string,
    timestamp: number
  ): void {
    this.send({
      type: 'operation',
      document_id: documentId,
      encrypted_operation: encryptedOperation,
      timestamp,
    });
  }

  onOperation(handler: OperationHandler): () => void {
    this.handlers.add(handler);
    return () => {
      this.handlers.delete(handler);
    };
  }

  onDocumentCreated(handler: DocumentCreatedHandler): () => void {
    this.documentCreatedHandlers.add(handler);
    return () => {
      this.documentCreatedHandlers.delete(handler);
    };
  }
}

export function createWebSocketClient(): WebSocketClient {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${protocol}//${window.location.host}/ws`;
  return new WebSocketClient(url);
}
