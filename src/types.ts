// Session types
export interface Session {
  id: string;
  name: string;
  description: string | null;
  status: 'active' | 'completed' | 'archived';
  created_at: string;
  updated_at: string;
}

export interface CreateSessionRequest {
  name?: string;
}

// Message types
export interface Message {
  id: string;
  session_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata: MessageMetadata | null;
  created_at: string;
}

export interface MessageMetadata {
  search_query?: string;
  search_results?: SearchResult[];
  model_used?: string;
  tokens_used?: number;
}

export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
  score: number;
}

// Chat types
export interface SendMessageRequest {
  session_id: string;
  content: string;
  retry?: boolean;
}

export interface StreamChunk {
  type: 'content' | 'search_start' | 'search_result' | 'error' | 'done';
  content?: string;
  search_query?: string;
  search_results?: SearchResult[];
  error?: string;
  session_id?: string;
}

// Document types
export interface GeneratedDocument {
  id: string;
  session_id: string;
  filename: string;
  content: string;
  created_at: string;
}

export interface GenerateProgress {
  current: number;
  total: number;
  filename: string;
}

export interface GenerateDocumentsRequest {
  session_id: string;
}

export interface SaveToFolderRequest {
  session_id: string;
  folder_path: string;
}

// Config types
export interface AppConfig {
  llm: LLMConfig;
  search: SearchConfig;
  ui: UIConfig;
  output: OutputConfig;
}

export interface LLMConfig {
  provider: 'ollama' | 'anthropic' | 'openai';
  model: string;
  base_url: string;
  temperature: number;
  max_tokens: number;
}

export interface SearchConfig {
  enabled: boolean;
  provider: 'tavily' | 'duckduckgo' | 'searxng' | 'none';
  tavily_api_key: string;
  searxng_url: string;
  proactive: boolean;
}

export interface UIConfig {
  theme: 'dark' | 'light';
}

export interface OutputConfig {
  include_conversation: boolean;
  default_save_path: string;
}

// Health check
export interface HealthStatus {
  ollama_connected: boolean;
  ollama_model_available: boolean;
  database_ok: boolean;
  config_valid: boolean;
  errors: string[];
}

export interface ErrorResponse {
  code: string;
  message: string;
  recoverable: boolean;
  action?: string;
}

// Model management
export interface ModelPullProgress {
  status: string;
  total?: number;
  completed?: number;
}

export interface DiskSpace {
  available_gb: number;
  sufficient: boolean;
}

// Onboarding
export type OnboardingStep =
  | "welcome"
  | "install-ollama"
  | "download-model"
  | "search"
  | "ready";
