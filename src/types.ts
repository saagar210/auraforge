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
  session_id: string;
}

export interface GenerateComplete {
  session_id: string;
  count: number;
}

export interface GenerateDocumentsRequest {
  session_id: string;
  target?: ForgeTarget;
  force?: boolean;
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
  provider: 'ollama';
  model: string;
  base_url: string;
  api_key?: string | null;
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
  default_target: ForgeTarget;
}

export type ForgeTarget = 'claude' | 'codex' | 'cursor' | 'gemini' | 'generic';

export interface QualityReport {
  score: number;
  missing_must_haves: string[];
  missing_should_haves: string[];
  summary: string;
}

export type CoverageStatus = 'missing' | 'partial' | 'covered';

export interface CoverageTopic {
  topic: string;
  status: CoverageStatus;
  evidence_message_ids: string[];
}

export interface CoverageReport {
  must_have: CoverageTopic[];
  should_have: CoverageTopic[];
  missing_must_haves: number;
  missing_should_haves: number;
  summary: string;
}

export interface ConfidenceFactor {
  name: string;
  max_points: number;
  points: number;
  detail: string;
}

export interface ConfidenceReport {
  score: number;
  factors: ConfidenceFactor[];
  blocking_gaps: string[];
  summary: string;
}

export interface GenerationMetadata {
  session_id: string;
  target: ForgeTarget | string;
  provider: string;
  model: string;
  quality_json: string | null;
  confidence_json: string | null;
  created_at: string;
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
