export interface Workspace {
  id: string;
  name: string;
  created_at: number;
}

export interface Contact {
  id: string;
  workspace_id: string;
  visitor_id: string;
  name: string | null;
  email: string | null;
  created_at: number;
  last_seen_at: number;
}

export interface Conversation {
  id: string;
  workspace_id: string;
  contact_id: string;
  contact_name: string | null;
  status: string;
  last_message: string | null;
  created_at: number;
  updated_at: number;
}

export interface Message {
  id: string;
  conversation_id: string;
  sender_type: 'agent' | 'visitor';
  sender_id: string;
  content: string;
  created_at: number;
}

export interface PageStats {
  page_url: string;
  visitors: number;
  page_views: number;
}

export interface CountryStats {
  country: string;
  visitors: number;
}

export interface BrowserStats {
  browser: string;
  visitors: number;
}

export interface Analytics {
  top_pages: PageStats[];
  top_countries: CountryStats[];
  top_browsers: BrowserStats[];
  total_visitors: number;
  total_page_views: number;
}
