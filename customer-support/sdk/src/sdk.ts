// Customer Support Chat Widget SDK
interface CustomerSupportConfig {
  workspaceId: string;
  apiUrl?: string;
  position?: 'bottom-right' | 'bottom-left';
  primaryColor?: string;
  welcomeMessage?: string;
}

interface Message {
  id: string;
  conversation_id: string;
  sender_type: 'agent' | 'visitor';
  sender_id: string;
  content: string;
  created_at: number;
}

interface VisitorInitResponse {
  contact_id: string;
  conversation_id: string | null;
  messages: Message[];
}

interface VisitorSendMessageResponse {
  conversation_id: string;
  message: Message;
}

class CustomerSupportWidget {
  private config: CustomerSupportConfig;
  private container: HTMLDivElement | null = null;
  private chatWindow: HTMLDivElement | null = null;
  private isOpen = false;
  private visitorId: string;
  private contactId: string | null = null;
  private conversationId: string | null = null;
  private messages: Message[] = [];
  private ws: WebSocket | null = null;

  constructor(config: CustomerSupportConfig) {
    this.config = {
      apiUrl: '',
      position: 'bottom-right',
      primaryColor: '#2563eb',
      welcomeMessage: 'Hi! How can we help you today?',
      ...config,
    };

    // Get or create visitor ID from cookie
    this.visitorId = this.getVisitorId();

    // Initialize widget when DOM is ready
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => this.init());
    } else {
      this.init();
    }
  }

  private getVisitorId(): string {
    const cookieName = 'cs_visitor_id';
    const cookies = document.cookie.split(';');
    for (const cookie of cookies) {
      const [name, value] = cookie.trim().split('=');
      if (name === cookieName) {
        return value;
      }
    }

    // Generate new visitor ID
    const newId = 'v_' + Math.random().toString(36).substring(2) + Date.now().toString(36);
    // Set cookie for 1 year
    const expires = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000);
    document.cookie = `${cookieName}=${newId}; expires=${expires.toUTCString()}; path=/; SameSite=Lax`;
    return newId;
  }

  private async init() {
    this.createStyles();
    this.createWidget();
    await this.initVisitor();
    this.connectWebSocket();
    this.trackPageView();
  }

  private createStyles() {
    const style = document.createElement('style');
    style.textContent = `
      .cs-widget-container {
        position: fixed;
        ${this.config.position === 'bottom-right' ? 'right: 20px;' : 'left: 20px;'}
        bottom: 20px;
        z-index: 999999;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
      }

      .cs-widget-button {
        width: 60px;
        height: 60px;
        border-radius: 50%;
        background-color: ${this.config.primaryColor};
        border: none;
        cursor: pointer;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.2s, box-shadow 0.2s;
      }

      .cs-widget-button:hover {
        transform: scale(1.05);
        box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
      }

      .cs-widget-button svg {
        width: 28px;
        height: 28px;
        fill: white;
      }

      .cs-chat-window {
        position: absolute;
        ${this.config.position === 'bottom-right' ? 'right: 0;' : 'left: 0;'}
        bottom: 80px;
        width: 380px;
        height: 520px;
        background: white;
        border-radius: 16px;
        box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
        display: none;
        flex-direction: column;
        overflow: hidden;
      }

      .cs-chat-window.open {
        display: flex;
      }

      .cs-chat-header {
        background-color: ${this.config.primaryColor};
        color: white;
        padding: 20px;
        display: flex;
        justify-content: space-between;
        align-items: center;
      }

      .cs-chat-header h3 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
      }

      .cs-chat-close {
        background: none;
        border: none;
        color: white;
        cursor: pointer;
        padding: 4px;
        opacity: 0.8;
        transition: opacity 0.2s;
      }

      .cs-chat-close:hover {
        opacity: 1;
      }

      .cs-chat-messages {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
        background: #f9fafb;
      }

      .cs-message {
        max-width: 80%;
        margin-bottom: 12px;
        padding: 12px 16px;
        border-radius: 16px;
        font-size: 14px;
        line-height: 1.4;
      }

      .cs-message.visitor {
        margin-left: auto;
        background-color: ${this.config.primaryColor};
        color: white;
        border-bottom-right-radius: 4px;
      }

      .cs-message.agent {
        background-color: white;
        color: #1f2937;
        border-bottom-left-radius: 4px;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
      }

      .cs-message-time {
        font-size: 11px;
        opacity: 0.7;
        margin-top: 4px;
      }

      .cs-chat-input {
        padding: 16px;
        background: white;
        border-top: 1px solid #e5e7eb;
        display: flex;
        gap: 12px;
      }

      .cs-chat-input input {
        flex: 1;
        padding: 12px 16px;
        border: 1px solid #e5e7eb;
        border-radius: 24px;
        font-size: 14px;
        outline: none;
        transition: border-color 0.2s;
      }

      .cs-chat-input input:focus {
        border-color: ${this.config.primaryColor};
      }

      .cs-chat-input button {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: ${this.config.primaryColor};
        border: none;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
      }

      .cs-chat-input button:hover {
        background-color: ${this.config.primaryColor}dd;
      }

      .cs-chat-input button:disabled {
        background-color: #d1d5db;
        cursor: not-allowed;
      }

      .cs-chat-input button svg {
        width: 18px;
        height: 18px;
        fill: white;
      }

      .cs-welcome-message {
        text-align: center;
        padding: 40px 20px;
        color: #6b7280;
      }

      .cs-welcome-message p {
        margin: 0;
        font-size: 15px;
      }

      @media (max-width: 420px) {
        .cs-chat-window {
          width: 100vw;
          height: 100vh;
          bottom: 0;
          right: 0;
          left: 0;
          border-radius: 0;
        }
      }
    `;
    document.head.appendChild(style);
  }

  private createWidget() {
    // Container
    this.container = document.createElement('div');
    this.container.className = 'cs-widget-container';

    // Chat window
    this.chatWindow = document.createElement('div');
    this.chatWindow.className = 'cs-chat-window';
    this.chatWindow.innerHTML = `
      <div class="cs-chat-header">
        <h3>Support</h3>
        <button class="cs-chat-close">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
      <div class="cs-chat-messages">
        <div class="cs-welcome-message">
          <p>${this.config.welcomeMessage}</p>
        </div>
      </div>
      <div class="cs-chat-input">
        <input type="text" placeholder="Type a message..." />
        <button disabled>
          <svg viewBox="0 0 24 24" fill="currentColor">
            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
          </svg>
        </button>
      </div>
    `;

    // Toggle button
    const toggleButton = document.createElement('button');
    toggleButton.className = 'cs-widget-button';
    toggleButton.innerHTML = `
      <svg viewBox="0 0 24 24" fill="currentColor">
        <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H6l-2 2V4h16v12z"/>
      </svg>
    `;

    // Event listeners
    toggleButton.addEventListener('click', () => this.toggle());
    this.chatWindow.querySelector('.cs-chat-close')?.addEventListener('click', () => this.close());

    const input = this.chatWindow.querySelector('input') as HTMLInputElement;
    const sendButton = this.chatWindow.querySelector('.cs-chat-input button') as HTMLButtonElement;

    input.addEventListener('input', () => {
      sendButton.disabled = !input.value.trim();
    });

    input.addEventListener('keypress', (e) => {
      if (e.key === 'Enter' && input.value.trim()) {
        this.sendMessage(input.value.trim());
        input.value = '';
        sendButton.disabled = true;
      }
    });

    sendButton.addEventListener('click', () => {
      if (input.value.trim()) {
        this.sendMessage(input.value.trim());
        input.value = '';
        sendButton.disabled = true;
      }
    });

    this.container.appendChild(this.chatWindow);
    this.container.appendChild(toggleButton);
    document.body.appendChild(this.container);
  }

  private async initVisitor() {
    try {
      const response = await fetch(
        `${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/visitor/init`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ visitor_id: this.visitorId }),
        }
      );

      if (!response.ok) {
        console.error('Failed to initialize visitor');
        return;
      }

      const data: VisitorInitResponse = await response.json();
      this.contactId = data.contact_id;
      this.conversationId = data.conversation_id;
      this.messages = data.messages;

      this.renderMessages();
    } catch (err) {
      console.error('Failed to initialize visitor:', err);
    }
  }

  private connectWebSocket() {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = this.config.apiUrl
      ? new URL(this.config.apiUrl).host
      : window.location.host;

    this.ws = new WebSocket(
      `${wsProtocol}//${wsHost}/ws/workspaces/${this.config.workspaceId}`
    );

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (
          data.type === 'new_message' &&
          data.conversation_id === this.conversationId
        ) {
          this.messages.push(data.message);
          this.renderMessages();
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    this.ws.onclose = () => {
      // Reconnect after 5 seconds
      setTimeout(() => this.connectWebSocket(), 5000);
    };
  }

  private renderMessages() {
    const messagesContainer = this.chatWindow?.querySelector('.cs-chat-messages');
    if (!messagesContainer) return;

    if (this.messages.length === 0) {
      messagesContainer.innerHTML = `
        <div class="cs-welcome-message">
          <p>${this.config.welcomeMessage}</p>
        </div>
      `;
      return;
    }

    messagesContainer.innerHTML = this.messages
      .map((msg) => {
        const time = new Date(msg.created_at).toLocaleTimeString([], {
          hour: '2-digit',
          minute: '2-digit',
        });
        return `
          <div class="cs-message ${msg.sender_type}">
            ${msg.content}
            <div class="cs-message-time">${time}</div>
          </div>
        `;
      })
      .join('');

    messagesContainer.scrollTop = messagesContainer.scrollHeight;
  }

  private async sendMessage(content: string) {
    try {
      const response = await fetch(
        `${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/visitor/message`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            visitor_id: this.visitorId,
            content,
            conversation_id: this.conversationId,
          }),
        }
      );

      if (!response.ok) {
        console.error('Failed to send message');
        return;
      }

      const data: VisitorSendMessageResponse = await response.json();
      this.conversationId = data.conversation_id;
      this.messages.push(data.message);
      this.renderMessages();
    } catch (err) {
      console.error('Failed to send message:', err);
    }
  }

  private toggle() {
    if (this.isOpen) {
      this.close();
    } else {
      this.open();
    }
  }

  open() {
    this.isOpen = true;
    this.chatWindow?.classList.add('open');
    (this.chatWindow?.querySelector('input') as HTMLInputElement)?.focus();
  }

  close() {
    this.isOpen = false;
    this.chatWindow?.classList.remove('open');
  }

  // Track page view (cookieless analytics)
  async trackPageView() {
    try {
      await fetch(
        `${this.config.apiUrl}/api/workspaces/${this.config.workspaceId}/track`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            page_url: window.location.pathname,
            page_title: document.title,
            referrer: document.referrer || null,
          }),
        }
      );
    } catch (err) {
      console.error('Failed to track page view:', err);
    }
  }
}

// Export for global usage
(window as unknown as { CustomerSupport: typeof CustomerSupportWidget }).CustomerSupport =
  CustomerSupportWidget;

export { CustomerSupportWidget, CustomerSupportConfig };
