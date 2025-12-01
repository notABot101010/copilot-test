export interface Session {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  status: string;
}

export interface Message {
  id: string;
  session_id: string;
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
  metadata?: string;
}

export interface PromptTemplate {
  id: string;
  name: string;
  subagent: string;
  system_prompt: string;
  variables: string[];
}

export type SteerCommand = 'cancel' | 'pause' | 'resume' | 'modify' | 'prioritize' | 'focus';

export interface StreamEvent {
  event_type: string;
  data: Record<string, unknown>;
}
