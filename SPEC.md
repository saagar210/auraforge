# AuraForge - Functional Specification

**Version**: 2.0  
**Date**: January 31, 2026  
**Status**: Ready for Implementation

---

## Executive Summary

AuraForge is a Tauri-based desktop application that serves as a Claude Code project planning assistant. Through conversational AI powered by local LLMs, it transforms project ideas into comprehensive planning documentation—specifications, Claude Code configurations, and phased implementation prompts—that users can immediately apply in their development workflow.

**Core Value Proposition**: Planning is the hardest part of vibe-coding. AuraForge eliminates the "where do I start?" problem by providing senior-engineer-level project planning through guided conversation, outputting everything needed to confidently execute in Claude Code.

**Target Users**: 
- Developers who use Claude Code for implementation
- Technical and non-technical individuals exploring AI-assisted development
- Anyone who struggles with project planning and wants structured guidance

---

## Product Vision

### What AuraForge Does (Plain Language)

> "AuraForge helps you plan software projects. You describe what you want to build, and it gives you a complete plan: what to build, how to build it, and step-by-step prompts you can use with AI coding tools like Claude Code. It's like having a senior engineer help you think through everything before you start coding."

### How You Use It

1. Open the app
2. Start a new project conversation
3. Describe your idea in natural language
4. AuraForge asks clarifying questions, challenges weak assumptions, and helps refine the vision
5. When the plan is solid, generate the output documents
6. Save to a folder and start building with Claude Code

### Where You Get the Output

A folder containing:
- `README.md` - Start here, explains everything
- `SPEC.md` - Complete specification
- `CLAUDE.md` - Drop into your project for Claude Code
- `PROMPTS.md` - Step-by-step implementation guide
- `CONVERSATION.md` - Full planning transcript
- Supporting files as needed

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    AuraForge Desktop Application                 │
│                         (Tauri 2.0 + React)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────────────┐  ┌─────────────────┐  ┌───────────────┐  │
│   │   Chat View     │  │  Preview View   │  │   History     │  │
│   │                 │  │                 │  │   Sidebar     │  │
│   │  Conversation   │  │   Generated     │  │               │  │
│   │  Interface      │  │   Documents     │  │   Past        │  │
│   │                 │  │   (tabbed)      │  │   Sessions    │  │
│   └────────┬────────┘  └────────┬────────┘  └───────┬───────┘  │
│            │                    │                    │          │
│   ┌────────┴────────────────────┴────────────────────┴────────┐ │
│   │                   React State (Zustand)                    │ │
│   │  - Current session                                         │ │
│   │  - Messages array                                          │ │
│   │  - Generated documents                                     │ │
│   │  - UI state                                                │ │
│   └────────────────────────────┬──────────────────────────────┘ │
│                                │                                 │
│   ┌────────────────────────────┴──────────────────────────────┐ │
│   │                    Tauri Commands (IPC)                    │ │
│   │  - send_message                                            │ │
│   │  - generate_documents                                      │ │
│   │  - save_to_folder                                          │ │
│   │  - web_search                                              │ │
│   │  - get_sessions / delete_session                           │ │
│   └────────────────────────────┬──────────────────────────────┘ │
│                                │                                 │
├────────────────────────────────┼────────────────────────────────┤
│                                │                                 │
│   ┌────────────────────────────┴──────────────────────────────┐ │
│   │                 Rust Backend (Tauri Core)                  │ │
│   │                                                            │ │
│   │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │ │
│   │  │  LLM Client  │  │   Search     │  │   Document       │ │ │
│   │  │  (Ollama)    │  │   Client     │  │   Generator      │ │ │
│   │  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘ │ │
│   │         │                 │                    │           │ │
│   │  ┌──────┴─────────────────┴────────────────────┴─────────┐ │ │
│   │  │                    SQLite Database                     │ │ │
│   │  │  - sessions                                            │ │ │
│   │  │  - messages                                            │ │ │
│   │  │  - documents                                           │ │ │
│   │  │  - preferences                                         │ │ │
│   │  └────────────────────────────────────────────────────────┘ │ │
│   └────────────────────────────────────────────────────────────┘ │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
                    │                           │
                    ▼                           ▼
             ┌──────────────┐           ┌──────────────┐
             │    Ollama    │           │   Tavily /   │
             │  localhost   │           │  DuckDuckGo  │
             │   :11434     │           │    (Web)     │
             └──────────────┘           └──────────────┘
```

### Technology Stack

| Layer | Technology | Version | Justification |
|-------|------------|---------|---------------|
| **App Shell** | Tauri | 2.0 | Lightweight, Rust-powered, native feel, cross-platform potential |
| **Frontend** | React | 19 | Modern, well-known, excellent ecosystem |
| **Language** | TypeScript | 5.x | Type safety, better IDE experience |
| **Styling** | Tailwind CSS | 4.x | Utility-first, rapid prototyping, dark mode support |
| **State** | Zustand | Latest | Minimal, no boilerplate, TypeScript-friendly |
| **Backend** | Rust | Latest stable | Tauri requirement, performance, safety |
| **Database** | SQLite | Via rusqlite | Embedded, no setup, portable |
| **LLM** | Ollama | Latest | Local, free, OpenAI-compatible API |
| **Model** | Qwen3-Coder-30B-A3B | GGUF Q4 | MoE architecture (fast), excellent for planning/code |
| **Web Search** | Tavily | API | LLM-optimized, 1000 free/month |
| **Fallback Search** | DuckDuckGo | Direct | Free, no API key required |

### Rust Dependencies (Cargo.toml)

```toml
[package]
name = "auraforge"
version = "0.1.0"
edition = "2021"

[dependencies]
# Tauri core
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-dialog = "2"        # Native file dialogs
tauri-plugin-fs = "2"            # File system access
tauri-plugin-shell = "2"         # Shell commands (optional)

# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP client for Ollama and search APIs
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Utilities
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"                  # Error handling
anyhow = "1"                     # Error propagation
dirs = "5"                       # Platform-specific directories
futures = "0.3"                  # Stream utilities

# Async channels for streaming
tokio-stream = "0.1"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

### Frontend Dependencies (package.json)

```json
{
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "zustand": "^5.0.0",
    "react-markdown": "^9.0.0",
    "remark-gfm": "^4.0.0",
    "react-syntax-highlighter": "^15.5.0",
    "lucide-react": "^0.400.0",
    "clsx": "^2.1.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "typescript": "^5.5.0",
    "vite": "^6.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "tailwindcss": "^4.0.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0"
  }
}
```

### Database Schema

```sql
-- Planning sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,                    -- UUID
    name TEXT NOT NULL,                     -- User-provided or auto-generated
    description TEXT,                       -- Brief summary (auto-generated)
    status TEXT DEFAULT 'active',           -- 'active', 'completed', 'archived'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Conversation messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions
    role TEXT NOT NULL,                     -- 'user', 'assistant', 'system'
    content TEXT NOT NULL,                  -- Message content
    metadata TEXT,                          -- JSON: search results, etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Generated documents (cached until regenerated)
CREATE TABLE documents (
    id TEXT PRIMARY KEY,                    -- UUID
    session_id TEXT NOT NULL,               -- FK to sessions
    filename TEXT NOT NULL,                 -- 'README.md', 'SPEC.md', etc.
    content TEXT NOT NULL,                  -- Document content
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- User preferences
CREATE TABLE preferences (
    key TEXT PRIMARY KEY,                   -- Preference key
    value TEXT NOT NULL                     -- JSON value
);

-- Indexes
CREATE INDEX idx_messages_session ON messages(session_id);
CREATE INDEX idx_documents_session ON documents(session_id);
CREATE INDEX idx_sessions_updated ON sessions(updated_at DESC);
```

### Configuration

**Config File Location**: `~/.auraforge/config.yaml`

```yaml
# AuraForge Configuration

# LLM Provider Settings
llm:
  provider: ollama                          # ollama | anthropic | openai
  model: qwen3-coder:30b-a3b-instruct-q4_K_M
  base_url: http://localhost:11434          # Ollama default
  temperature: 0.7
  max_tokens: 65536

# Web Search Settings
search:
  enabled: true
  provider: tavily                          # tavily | duckduckgo | searxng | none
  tavily_api_key: ""                        # Required if using Tavily
  searxng_url: ""                           # Required if using SearXNG
  proactive: true                           # Auto-search during conversation

# UI Preferences
ui:
  theme: dark                               # dark | light (dark is default)
  
# Output Preferences  
output:
  include_conversation: true                # Include CONVERSATION.md
  default_save_path: ~/Projects             # Default folder picker location
```

### Config Validation

On startup, validate config with clear error messages:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found. Creating default at {0}")]
    NotFound(String),
    
    #[error("Invalid YAML syntax at line {line}: {message}")]
    ParseError { line: usize, message: String },
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid Ollama URL: {0}. Expected format: http://localhost:11434")]
    InvalidOllamaUrl(String),
    
    #[error("Tavily API key required when search.provider is 'tavily'")]
    MissingTavilyKey,
}
```

**First-run behavior**: If no config exists, create default config with comments explaining each field. Show welcome modal explaining prerequisites (Ollama installed, model downloaded).

---

## Tauri Commands (IPC Interface)

All communication between React frontend and Rust backend happens through these commands. Each command is defined with its TypeScript interface and Rust signature.

### TypeScript Types (src/types.ts)

```typescript
// Session types
interface Session {
  id: string;
  name: string;
  description: string | null;
  status: 'active' | 'completed' | 'archived';
  created_at: string;
  updated_at: string;
}

interface CreateSessionRequest {
  name?: string;  // Auto-generated if not provided
}

// Message types
interface Message {
  id: string;
  session_id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  metadata: MessageMetadata | null;
  created_at: string;
}

interface MessageMetadata {
  search_query?: string;
  search_results?: SearchResult[];
  model_used?: string;
  tokens_used?: number;
}

interface SearchResult {
  title: string;
  url: string;
  snippet: string;
  score: number;
}

// Chat types
interface SendMessageRequest {
  session_id: string;
  content: string;
}

interface StreamChunk {
  type: 'content' | 'search_start' | 'search_result' | 'error' | 'done';
  content?: string;
  search_query?: string;
  search_results?: SearchResult[];
  error?: string;
}

// Document types
interface GeneratedDocument {
  id: string;
  session_id: string;
  filename: string;
  content: string;
  created_at: string;
}

interface GenerateDocumentsRequest {
  session_id: string;
}

interface SaveToFolderRequest {
  session_id: string;
  folder_path: string;
}

// Config types
interface AppConfig {
  llm: LLMConfig;
  search: SearchConfig;
  ui: UIConfig;
  output: OutputConfig;
}

interface LLMConfig {
  provider: 'ollama' | 'anthropic' | 'openai';
  model: string;
  base_url: string;
  temperature: number;
  max_tokens: number;
}

interface SearchConfig {
  enabled: boolean;
  provider: 'tavily' | 'duckduckgo' | 'searxng' | 'none';
  tavily_api_key: string;
  searxng_url: string;
  proactive: boolean;
}

// Health check
interface HealthStatus {
  ollama_connected: boolean;
  ollama_model_available: boolean;
  database_ok: boolean;
  config_valid: boolean;
  errors: string[];
}
```

### Rust Command Signatures

```rust
// src-tauri/src/commands/mod.rs

use tauri::State;
use crate::state::AppState;
use crate::types::*;

// ============ HEALTH & CONFIG ============

/// Check if Ollama is running and model is available
#[tauri::command]
pub async fn check_health(state: State<'_, AppState>) -> Result<HealthStatus, String>;

/// Get current configuration
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String>;

/// Update configuration (validates before saving)
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: AppConfig
) -> Result<(), String>;

// ============ SESSIONS ============

/// Create a new planning session
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    request: CreateSessionRequest
) -> Result<Session, String>;

/// Get all sessions (sorted by updated_at DESC)
#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String>;

/// Get a single session by ID
#[tauri::command]
pub async fn get_session(
    state: State<'_, AppState>,
    session_id: String
) -> Result<Session, String>;

/// Update session name or status
#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session_id: String,
    name: Option<String>,
    status: Option<String>
) -> Result<Session, String>;

/// Delete a session and all its messages/documents
#[tauri::command]
pub async fn delete_session(
    state: State<'_, AppState>,
    session_id: String
) -> Result<(), String>;

// ============ MESSAGES ============

/// Get all messages for a session
#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    session_id: String
) -> Result<Vec<Message>, String>;

/// Send a message and stream the response via Tauri events
/// Returns the user message immediately; assistant response comes via events
#[tauri::command]
pub async fn send_message(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SendMessageRequest
) -> Result<Message, String>;

/// Cancel an in-progress response
#[tauri::command]
pub async fn cancel_response(
    state: State<'_, AppState>,
    session_id: String
) -> Result<(), String>;

// ============ DOCUMENTS ============

/// Generate all planning documents for a session
/// Progress updates come via Tauri events
#[tauri::command]
pub async fn generate_documents(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: GenerateDocumentsRequest
) -> Result<Vec<GeneratedDocument>, String>;

/// Get cached documents for a session
#[tauri::command]
pub async fn get_documents(
    state: State<'_, AppState>,
    session_id: String
) -> Result<Vec<GeneratedDocument>, String>;

/// Save documents to a folder
#[tauri::command]
pub async fn save_to_folder(
    state: State<'_, AppState>,
    request: SaveToFolderRequest
) -> Result<String, String>;  // Returns the created folder path

// ============ SEARCH ============

/// Manual web search (for testing or explicit user request)
#[tauri::command]
pub async fn web_search(
    state: State<'_, AppState>,
    query: String
) -> Result<Vec<SearchResult>, String>;
```

### Tauri Event System (Streaming)

Streaming responses use Tauri's event system, not command return values.

**Event Names:**
- `stream:chunk` - LLM response chunk
- `stream:search` - Web search initiated
- `stream:error` - Error during streaming
- `stream:done` - Response complete
- `generate:progress` - Document generation progress
- `generate:complete` - All documents generated

**Frontend Event Listening:**

```typescript
// src/hooks/useChat.ts
import { listen } from '@tauri-apps/api/event';

export function useChat(sessionId: string) {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingContent, setStreamingContent] = useState('');
  
  useEffect(() => {
    const unlistenChunk = listen<StreamChunk>('stream:chunk', (event) => {
      if (event.payload.type === 'content') {
        setStreamingContent(prev => prev + event.payload.content);
      }
    });
    
    const unlistenDone = listen('stream:done', () => {
      setIsStreaming(false);
      // Refresh messages from DB to get final saved version
      refetchMessages();
    });
    
    const unlistenError = listen<StreamChunk>('stream:error', (event) => {
      setIsStreaming(false);
      setError(event.payload.error);
    });
    
    return () => {
      unlistenChunk.then(fn => fn());
      unlistenDone.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [sessionId]);
  
  const sendMessage = async (content: string) => {
    setIsStreaming(true);
    setStreamingContent('');
    await invoke('send_message', { request: { session_id: sessionId, content } });
  };
  
  return { sendMessage, isStreaming, streamingContent };
}
```

**Rust Streaming Implementation:**

```rust
// src-tauri/src/llm/ollama.rs
use futures::StreamExt;
use tauri::Emitter;

pub async fn stream_chat_response(
    app: &tauri::AppHandle,
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    temperature: f32,
) -> Result<String, LLMError> {
    let url = format!("{}/api/chat", base_url);
    
    let response = client
        .post(&url)
        .json(&OllamaRequest {
            model: model.to_string(),
            messages,
            stream: true,
            options: OllamaOptions { temperature },
        })
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(LLMError::ApiError(response.status().to_string()));
    }
    
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        
        // Ollama returns newline-delimited JSON
        for line in text.lines() {
            if line.is_empty() { continue; }
            
            let parsed: OllamaStreamResponse = serde_json::from_str(line)?;
            
            if let Some(content) = parsed.message.content {
                full_response.push_str(&content);
                
                // Emit chunk to frontend
                app.emit("stream:chunk", StreamChunk {
                    r#type: "content".to_string(),
                    content: Some(content),
                    ..Default::default()
                })?;
            }
            
            if parsed.done {
                app.emit("stream:done", ())?;
            }
        }
    }
    
    Ok(full_response)
}
```

---

## State Machines

### Session Lifecycle

```
                    ┌─────────────┐
                    │   (start)   │
                    └──────┬──────┘
                           │ create_session
                           ▼
                    ┌─────────────┐
          ┌────────►│   active    │◄────────┐
          │         └──────┬──────┘         │
          │                │                │
          │ reopen         │ generate_      │ continue
          │                │ documents      │ conversation
          │                ▼                │
          │         ┌─────────────┐         │
          │         │ generating  │─────────┘
          │         └──────┬──────┘  (success)
          │                │
          │                │ documents_saved
          │                ▼
          │         ┌─────────────┐
          └─────────│  completed  │
                    └──────┬──────┘
                           │ archive
                           ▼
                    ┌─────────────┐
                    │  archived   │
                    └──────┬──────┘
                           │ delete
                           ▼
                    ┌─────────────┐
                    │  (deleted)  │
                    └─────────────┘
```

### Chat State Machine (Frontend)

```
                    ┌─────────────┐
                    │    idle     │◄──────────────────────┐
                    └──────┬──────┘                       │
                           │ user types                   │
                           ▼                              │
                    ┌─────────────┐                       │
                    │   typing    │                       │
                    └──────┬──────┘                       │
                           │ send (Cmd+Enter)             │
                           ▼                              │
                    ┌─────────────┐                       │
                    │  sending    │                       │
                    └──────┬──────┘                       │
                           │ message saved                │
                           ▼                              │
        ┌───────────┬─────────────┬───────────┐          │
        │           │             │           │          │
        ▼           ▼             ▼           │          │
┌──────────┐ ┌───────────┐ ┌──────────┐      │          │
│searching │ │  waiting  │ │streaming │      │          │
└────┬─────┘ └─────┬─────┘ └────┬─────┘      │          │
     │             │            │             │          │
     │ results     │ first      │ chunks      │          │
     │             │ chunk      │             │          │
     └─────────────┴────────────┘             │          │
                           │                  │          │
                           │ stream:done      │          │
                           │ OR cancel        │          │
                           └──────────────────┼──────────┘
                                              │
                           ┌──────────────────┘
                           │ stream:error
                           ▼
                    ┌─────────────┐
                    │   error     │───────────────────────┐
                    └─────────────┘  (dismiss/retry)      │
                                                          │
                    ┌─────────────────────────────────────┘
                    ▼
              (back to idle)
```

### Document Generation State

```
                    ┌─────────────┐
                    │    none     │ (no documents cached)
                    └──────┬──────┘
                           │ generate_documents
                           ▼
                    ┌─────────────┐
                    │ generating  │ ← shows progress UI
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           │               │               │
           ▼               ▼               ▼
    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
    │   README    │ │    SPEC     │ │   CLAUDE    │ ...
    │ generating  │ │ generating  │ │ generating  │
    └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
           │               │               │
           └───────────────┼───────────────┘
                           │ all complete
                           ▼
                    ┌─────────────┐
                    │   cached    │ ← documents viewable
                    └──────┬──────┘
                           │ conversation continues
                           ▼
                    ┌─────────────┐
                    │    stale    │ ← shows "regenerate" badge
                    └──────┬──────┘
                           │ regenerate
                           ▼
                     (back to generating)
```

---

## Error Handling

### Error Categories and UI Responses

| Error | Detection | User Message | Recovery Action |
|-------|-----------|--------------|-----------------|
| Ollama not running | Health check fails, connection refused | "Can't connect to Ollama. Is it running?" | Link to Ollama install docs, "Retry" button |
| Model not found | Ollama 404 on model | "Model '{model}' not found. Download it first." | Show command: `ollama pull {model}` |
| Model loading | Ollama returns "loading" status | "Model is loading... ({progress}%)" | Show progress, auto-retry |
| Network timeout | Request timeout after 30s | "Request timed out. Ollama might be overloaded." | "Retry" button |
| Stream interrupted | Connection closed mid-stream | "Response was interrupted." | Show partial response, "Regenerate" option |
| Tavily rate limit | 429 response | "Search limit reached. Switching to fallback." | Auto-switch to DuckDuckGo |
| Tavily auth fail | 401 response | "Invalid Tavily API key." | Open settings, highlight field |
| DuckDuckGo blocked | Scraping blocked | "Web search unavailable." | Disable proactive search, continue without |
| Database error | SQLite error | "Database error. Your data is safe." | Auto-retry, show technical details in expandable |
| Disk full | Write fails | "Disk full. Free up space to save." | Show disk usage, suggest cleanup |
| Invalid config | Parse/validation fails | "Configuration error: {specific field}" | Open settings with field highlighted |
| Document gen fail | LLM returns invalid format | "Failed to generate {doc}. Retrying..." | Auto-retry up to 3x, then manual retry |

### Error UI Components

```typescript
// src/components/ErrorBanner.tsx
interface ErrorBannerProps {
  error: AppError;
  onDismiss: () => void;
  onRetry?: () => void;
}

// Inline error for specific fields
interface FieldErrorProps {
  message: string;
}

// Full-screen error for fatal issues (Ollama not running)
interface FatalErrorScreenProps {
  title: string;
  message: string;
  helpUrl?: string;
  retryAction?: () => void;
}
```

### Rust Error Types

```rust
// src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // LLM errors
    #[error("Cannot connect to Ollama at {url}: {message}")]
    OllamaConnection { url: String, message: String },
    
    #[error("Model '{model}' not found. Run: ollama pull {model}")]
    ModelNotFound { model: String },
    
    #[error("LLM request failed: {0}")]
    LLMRequest(String),
    
    #[error("Response stream interrupted")]
    StreamInterrupted,
    
    // Search errors
    #[error("Tavily API error: {0}")]
    TavilyError(String),
    
    #[error("Search rate limited. Daily limit reached.")]
    SearchRateLimit,
    
    #[error("Web search unavailable")]
    SearchUnavailable,
    
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    // Config errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    // File system errors
    #[error("Cannot write to {path}: {message}")]
    FileSystem { path: String, message: String },
    
    #[error("Folder already exists: {0}")]
    FolderExists(String),
}

// Convert to user-friendly JSON for frontend
impl AppError {
    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code(),
            message: self.to_string(),
            recoverable: self.is_recoverable(),
            action: self.suggested_action(),
        }
    }
}
```

---

## Core Features

### 1. Conversation Interface

**Purpose**: Natural language planning discussion with AI

**Behavior**:
- Chat-style interface with message bubbles
- User types project ideas, questions, requirements
- AI responds with questions, suggestions, challenges, and refinements
- Supports markdown rendering in messages
- Shows "thinking" indicator during LLM response
- Displays web search activity when triggered

**Conversation Style** (System Prompt Behavior):
- **Friendly and supportive**: Encouraging tone, celebrates good ideas
- **Challenging**: Pushes back on weak assumptions, asks probing questions
- **Alternatives-focused**: Offers different approaches when relevant
- **Proactive**: Searches web for current best practices when discussing tech choices
- **Structured**: Mentally tracks what's been decided vs. what needs resolution

**Example Interaction**:
```
User: I want to build a habit tracker app

AuraForge: That's a solid project idea! Before we dive in, let me understand 
the scope:

1. **Platform**: Web app, mobile app, or desktop? (Or multiple?)
2. **Users**: Just for yourself, or something others would use too?
3. **Core feature**: What's the ONE thing it must do really well?

Also—what's drawing you to build this vs. using an existing app like 
Habitica or Streaks?

User: Web app, just for me, mainly tracking daily habits with streaks

AuraForge: Got it—personal tool, web-based, streak tracking. A few more 
questions to nail down the approach:

**Data persistence**: 
- Browser localStorage (simplest, but loses data if you clear browser)
- Backend + database (more work, but data survives anywhere)

For a personal tool, localStorage is honestly fine to start. You can always 
add a backend later. Thoughts?

**Tech stack**: Any preferences? Or should I suggest something based on 
your goals?

[Searching: "best tech stack for personal web app 2026"...]

Based on current trends, for a simple personal app I'd suggest:
- **Vite + React** (fast dev experience)
- **Tailwind** (quick styling)
- **localStorage** for now

Want to go with this, or explore alternatives?
```

### 2. Web Search Integration

**Purpose**: Ground planning discussions in current best practices

**Behavior**:
- **Proactive search**: When user discusses tech choices, AuraForge automatically searches for current recommendations
- **Search indicator**: Shows "Searching: [query]..." in conversation
- **Source citation**: Mentions relevant findings naturally in response
- **Configurable**: Can be disabled in settings

**Search Triggers** (Proactive):
- Technology/framework discussions ("should I use React or Vue?")
- Best practices questions ("what's the folder structure for...")
- Current state queries ("is X library still maintained?")
- Version/compatibility questions ("does X work with Y?")

**Providers**:
- **Tavily** (default): API call, 1000 free/month, LLM-optimized results
- **DuckDuckGo** (fallback): Direct scraping, free, no API key
- **SearXNG** (optional): Self-hosted, requires Docker URL in config

#### Proactive Search Detection

The system analyzes each user message to determine if a web search would improve the response.

```rust
// src-tauri/src/search/trigger.rs

/// Keywords and patterns that trigger proactive search
const TECH_KEYWORDS: &[&str] = &[
    "react", "vue", "angular", "svelte", "next", "nuxt", "remix",
    "node", "deno", "bun", "python", "rust", "go", "java",
    "postgres", "mysql", "mongodb", "redis", "sqlite",
    "aws", "gcp", "azure", "vercel", "netlify", "railway",
    "docker", "kubernetes", "terraform",
    "tailwind", "bootstrap", "chakra", "shadcn",
    "prisma", "drizzle", "typeorm", "sequelize",
];

const TRIGGER_PATTERNS: &[&str] = &[
    "should i use",
    "which is better",
    "vs",
    "versus",
    "compare",
    "best practice",
    "recommended",
    "how to structure",
    "folder structure",
    "is * maintained",
    "is * deprecated",
    "does * work with",
    "compatible with",
    "latest version",
    "current version",
    "in 2025",
    "in 2026",
];

pub fn should_search(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    
    // Check for explicit comparison questions
    if lower.contains(" vs ") || lower.contains(" versus ") {
        // Extract the comparison terms
        return Some(extract_comparison_query(&lower));
    }
    
    // Check for "should I use X" patterns
    if lower.contains("should i use") || lower.contains("should we use") {
        return Some(extract_tech_query(&lower));
    }
    
    // Check for best practices questions with tech keywords
    for pattern in TRIGGER_PATTERNS {
        if pattern.contains('*') {
            // Wildcard pattern
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                if let Some(start) = lower.find(parts[0]) {
                    if let Some(_) = lower[start..].find(parts[1]) {
                        return Some(build_search_query(&lower));
                    }
                }
            }
        } else if lower.contains(pattern) {
            // Check if tech keyword is also present
            for tech in TECH_KEYWORDS {
                if lower.contains(tech) {
                    return Some(format!("{} {}", tech, pattern));
                }
            }
        }
    }
    
    None
}

fn build_search_query(message: &str) -> String {
    // Extract key terms, add year for freshness
    let mut terms: Vec<&str> = message
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .filter(|w| !["should", "would", "could", "what", "which", "the", "for"].contains(w))
        .take(5)
        .collect();
    
    terms.push("2026"); // Ensure recent results
    terms.join(" ")
}
```

#### Search Context Injection

When search results are available, they're injected into the conversation context:

```rust
// src-tauri/src/llm/context.rs

pub fn build_messages_with_search(
    messages: &[Message],
    search_results: Option<&[SearchResult]>,
) -> Vec<ChatMessage> {
    let mut chat_messages = Vec::new();
    
    // Add system prompt
    chat_messages.push(ChatMessage {
        role: "system".to_string(),
        content: AURAFORGE_SYSTEM_PROMPT.to_string(),
    });
    
    // Add conversation history
    for msg in messages {
        chat_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }
    
    // If search results exist, add them as context before the last user message
    if let Some(results) = search_results {
        if !results.is_empty() {
            let search_context = format!(
                "[Web Search Results]\n\
                 Use these results to inform your response. Cite sources naturally.\n\n\
                 {}\n\
                 [End Search Results]",
                results.iter()
                    .map(|r| format!("**{}**\n{}\nSource: {}\n", r.title, r.snippet, r.url))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            
            // Insert before the last message
            let last = chat_messages.pop();
            chat_messages.push(ChatMessage {
                role: "system".to_string(),
                content: search_context,
            });
            if let Some(msg) = last {
                chat_messages.push(msg);
            }
        }
    }
    
    chat_messages
}
```

#### Tavily API Client

```rust
// src-tauri/src/search/tavily.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};

const TAVILY_API_URL: &str = "https://api.tavily.com/search";

#[derive(Serialize)]
struct TavilyRequest {
    api_key: String,
    query: String,
    search_depth: String,      // "basic" or "advanced"
    include_answer: bool,
    include_raw_content: bool,
    max_results: u32,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f32,
}

pub async fn search(
    client: &Client,
    api_key: &str,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    let request = TavilyRequest {
        api_key: api_key.to_string(),
        query: query.to_string(),
        search_depth: "basic".to_string(),
        include_answer: false,
        include_raw_content: false,
        max_results: 5,
    };
    
    let response = client
        .post(TAVILY_API_URL)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    match response.status() {
        reqwest::StatusCode::OK => {
            let tavily_response: TavilyResponse = response.json().await?;
            Ok(tavily_response.results.into_iter().map(|r| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.content,
                score: r.score,
            }).collect())
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            Err(SearchError::InvalidApiKey)
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            Err(SearchError::RateLimited)
        }
        status => {
            Err(SearchError::ApiError(status.to_string()))
        }
    }
}
```

#### DuckDuckGo Fallback

```rust
// src-tauri/src/search/duckduckgo.rs

use reqwest::Client;
use scraper::{Html, Selector};

const DDG_HTML_URL: &str = "https://html.duckduckgo.com/html/";

pub async fn search(
    client: &Client,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    let response = client
        .post(DDG_HTML_URL)
        .form(&[("q", query)])
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(SearchError::ApiError(response.status().to_string()));
    }
    
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    let result_selector = Selector::parse(".result").unwrap();
    let title_selector = Selector::parse(".result__title a").unwrap();
    let snippet_selector = Selector::parse(".result__snippet").unwrap();
    
    let mut results = Vec::new();
    
    for (i, result) in document.select(&result_selector).take(5).enumerate() {
        let title = result
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();
            
        let url = result
            .select(&title_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or_default()
            .to_string();
            
        let snippet = result
            .select(&snippet_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();
        
        if !title.is_empty() && !url.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
                score: 1.0 - (i as f32 * 0.1), // Decreasing score by position
            });
        }
    }
    
    if results.is_empty() {
        Err(SearchError::NoResults)
    } else {
        Ok(results)
    }
}
```

#### Search Orchestration

```rust
// src-tauri/src/search/mod.rs

pub async fn execute_search(
    config: &SearchConfig,
    client: &Client,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    if !config.enabled {
        return Ok(vec![]);
    }
    
    match config.provider.as_str() {
        "tavily" => {
            match tavily::search(client, &config.tavily_api_key, query).await {
                Ok(results) => Ok(results),
                Err(SearchError::RateLimited) | Err(SearchError::InvalidApiKey) => {
                    // Fallback to DuckDuckGo
                    log::warn!("Tavily failed, falling back to DuckDuckGo");
                    duckduckgo::search(client, query).await
                }
                Err(e) => Err(e),
            }
        }
        "duckduckgo" => duckduckgo::search(client, query).await,
        "searxng" => searxng::search(client, &config.searxng_url, query).await,
        _ => Ok(vec![]),
    }
}
```

### 3. Document Generation

**Purpose**: Transform conversation into actionable planning documents

**Trigger**: User says "generate the plan" / "I'm ready" / clicks Generate button

**Process**:
1. Analyze full conversation history
2. Extract decisions, requirements, tech choices
3. Generate each document with appropriate detail
4. Store in database (cached until regenerated)
5. Display in Preview View with tabs

**Output Documents**:

#### README.md
```markdown
# [Project Name] - Project Plan

Generated by AuraForge on [Date]

## What Is This?

This folder contains everything you need to build [brief description] 
using Claude Code. The planning has been done—your job is to execute.

## Files Included

| File | Purpose |
|------|---------|
| SPEC.md | Complete specification—what you're building and why |
| CLAUDE.md | Drop this in your project root for Claude Code |
| PROMPTS.md | Step-by-step prompts to build the project |
| CONVERSATION.md | Full planning conversation for reference |
| [Other files] | [Descriptions] |

## How to Use

1. Create a new project folder
2. Copy `CLAUDE.md` into it
3. Open Claude Code in that folder
4. Follow the prompts in `PROMPTS.md` phase by phase
5. After each phase, run the verification steps before continuing

## Project Overview

[Auto-generated summary of what's being built]

## Key Decisions

[Bulleted list of major decisions made during planning]
```

#### SPEC.md
```markdown
# [Project Name] - Specification

## Overview

[Comprehensive description of the project]

## Problem Statement

[What problem this solves, why it matters]

## Goals & Non-Goals

### Goals
- [Goal 1]
- [Goal 2]

### Non-Goals (Explicitly Out of Scope)
- [Non-goal 1]

## User Stories

1. As a [user type], I want to [action] so that [benefit]
2. ...

## Technical Architecture

### Tech Stack
| Layer | Choice | Rationale |
|-------|--------|-----------|
| Frontend | [X] | [Why] |
| Backend | [X] | [Why] |
| Database | [X] | [Why] |

### System Design
[Architecture description, ASCII diagram if helpful]

### Data Models
[Entity descriptions, relationships]

## Features

### Core Features (Must Have)
1. **[Feature Name]**
   - Description: [What it does]
   - Acceptance criteria: [How we know it's done]

### Future Features (Nice to Have)
1. ...

## API Design (if applicable)

[Endpoint specifications]

## Security Considerations

[Auth approach, data handling, etc.]

## Open Questions

[Any unresolved items from the conversation]
```

#### CLAUDE.md
```markdown
# [Project Name]

## Project Overview

[Brief description for Claude Code context]

## Tech Stack

- **Frontend**: [X]
- **Backend**: [X]
- **Database**: [X]
- **Testing**: [X]

## Project Structure

```
project-root/
├── src/
│   ├── components/
│   ├── pages/
│   └── ...
├── tests/
├── package.json
└── ...
```

## Commands

- `npm install` - Install dependencies
- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run test` - Run tests

## Code Conventions

- [Naming conventions]
- [File organization rules]
- [Import ordering]

## Common Tasks

### Adding a new component
[Steps]

### Adding a new API endpoint
[Steps]

### Running tests
[Steps]

## Important Notes

- [Project-specific gotchas]
- [Things to remember]
```

#### PROMPTS.md
```markdown
# [Project Name] - Implementation Guide

## How to Use This File

This file contains phased prompts for building [project name] with Claude Code. 
Work through each phase in order. After each phase, complete the verification 
steps before moving on.

---

## Phase 1: Project Setup

### Prompt
```
[Exact prompt to paste into Claude Code]

Create the project foundation:
1. Initialize the project with [framework]
2. Set up the folder structure as defined in CLAUDE.md
3. Install core dependencies: [list]
4. Configure [tools]
5. Create a basic "Hello World" to verify setup

Reference CLAUDE.md for project conventions.
```

### What You Should Have After
- [ ] Project folder created with correct structure
- [ ] Dependencies installed (`npm install` succeeds)
- [ ] Dev server runs (`npm run dev` shows Hello World)
- [ ] CLAUDE.md is in the project root

### Common Issues
- If [X] fails, try [Y]
- Make sure [Z] is installed

---

## Phase 2: [Core Feature]

### Prompt
```
[Exact prompt for this phase]
```

### What You Should Have After
- [ ] [Checklist item]
- [ ] [Checklist item]

### Verification Command
```bash
[Command to verify this phase works]
```

---

[Additional phases...]

---

## Phase N: Polish & Deploy

### Prompt
```
[Final phase prompt]
```

### What You Should Have After
- [ ] All features working
- [ ] Tests passing
- [ ] Ready for deployment

---

## Congratulations!

You've built [project name]. Here's what you accomplished:
[Summary of completed project]

### Next Steps (Optional)
- [Enhancement idea 1]
- [Enhancement idea 2]
```

#### CONVERSATION.md
```markdown
# [Project Name] - Planning Conversation

This is the complete planning conversation that generated these documents.
Kept for reference—you can revisit to understand why decisions were made.

---

**Session started**: [Date/Time]

---

**User**: [First message]

**AuraForge**: [Response]

**User**: [Next message]

[...full conversation...]

---

**Session ended**: [Date/Time]
**Documents generated**: [Date/Time]
```

#### Supporting Files (Conditional)

- **SCHEMA.prisma** - If database was discussed
- **API.md** - If backend API was designed
- **ARCHITECTURE.md** - If complex system design was discussed
- **.env.example** - If environment variables were mentioned

---

### Document Generation Prompts

These are the actual LLM prompts used to generate each document. Each prompt receives the full conversation history as context.

#### System Prompt for Document Generation

```
You are a document generator for AuraForge, a project planning tool. Your job is to 
extract information from a planning conversation and generate specific documentation.

CRITICAL RULES:
1. Only include information that was explicitly discussed or decided in the conversation
2. If a topic wasn't covered, either omit it or mark it as "[To be determined]"
3. Never invent features, technologies, or requirements that weren't discussed
4. If the conversation was incomplete, note what's missing in an "Open Questions" section
5. Be specific—use exact names, versions, and details from the conversation
6. Format output as clean, valid Markdown

When extracting information, look for:
- Explicit decisions: "Let's use React" / "I'll go with PostgreSQL"
- Confirmed requirements: "It needs to..." / "The main feature is..."
- Rejected alternatives: "Not X because..." (mention as non-goals)
- Unresolved items: Questions asked but not answered
```

#### README.md Generation Prompt

```
Generate a README.md for this project based on the planning conversation below.

The README should:
1. Have a clear title with the project name
2. Include "Generated by AuraForge on [current date]"
3. Explain what's in the folder and how to use the files
4. Provide a brief project overview (2-3 sentences)
5. List key decisions made during planning

Structure:
- What Is This? (1 paragraph explaining the folder contents)
- Files Included (table of files with purposes)
- How to Use (numbered steps to get started with Claude Code)
- Project Overview (summary of what's being built)
- Key Decisions (bullet list of major choices made)

Keep it concise—this is a quick-start guide, not the full spec.

<conversation>
{conversation_history}
</conversation>

Generate the README.md now:
```

#### SPEC.md Generation Prompt

```
Generate a comprehensive SPEC.md based on the planning conversation below.

Extract and organize:

1. **Overview**: What is this project? Who is it for? (from early conversation)

2. **Problem Statement**: Why does this need to exist? What problem does it solve?

3. **Goals**: Explicit objectives mentioned. Be specific.
   - If user said "I want it to be fast" → "Sub-100ms response times for API calls"
   
4. **Non-Goals**: Things explicitly ruled out or deferred
   - "Not doing X for v1" / "Out of scope" / "Maybe later"

5. **User Stories**: Convert discussed features into user story format
   - "As a [user type], I want to [action] so that [benefit]"

6. **Technical Architecture**:
   - Tech stack with specific versions if mentioned
   - System design (create ASCII diagram if architecture was discussed)
   - Data models (entities and relationships)

7. **Features**: 
   - Core features (must have for v1)
   - Future features (mentioned but deferred)
   - For each: description + acceptance criteria

8. **API Design**: Only if backend/API was discussed
   - Endpoints with methods, paths, request/response

9. **Security Considerations**: Auth, data handling, etc. if discussed

10. **Open Questions**: Anything unresolved in the conversation

If a section has no information from the conversation, include the heading with 
"[Not discussed - to be determined]" rather than inventing content.

<conversation>
{conversation_history}
</conversation>

Generate the SPEC.md now:
```

#### CLAUDE.md Generation Prompt

```
Generate a CLAUDE.md file for Claude Code based on the planning conversation below.

This file will be placed in the project root to give Claude Code context about the project.
It should be concise and actionable—Claude Code will reference this while building.

Include:

1. **Project Overview**: 2-3 sentences max. What is this?

2. **Tech Stack**: Bullet list of technologies with versions
   - Only include what was decided in the conversation
   - Format: "- **Category**: Technology (version if specified)"

3. **Project Structure**: ASCII tree of the folder structure
   - Base on tech stack conventions if not explicitly discussed
   - Keep it to top 2-3 levels

4. **Commands**: Standard commands for this tech stack
   - Install, dev, build, test, lint
   - Use actual package manager/commands for the chosen stack

5. **Code Conventions**: 
   - Naming conventions (if discussed)
   - File organization rules
   - Any specific patterns mentioned

6. **Common Tasks**: Brief how-to for:
   - Adding a new component/module
   - Adding a new API endpoint (if applicable)
   - Running tests

7. **Important Notes**: Project-specific gotchas or rules mentioned

Keep each section brief. Claude Code needs quick reference, not exhaustive documentation.

<conversation>
{conversation_history}
</conversation>

Generate the CLAUDE.md now:
```

#### PROMPTS.md Generation Prompt

```
Generate a PROMPTS.md implementation guide based on the planning conversation below.

This file contains step-by-step prompts that the user will paste into Claude Code.
Each phase should be a self-contained unit of work with clear verification steps.

CRITICAL: The prompts must be copy-pasteable. Write them as if speaking directly to Claude Code.

Structure each phase:

## Phase N: [Name]

### Prompt
```
[The exact prompt to paste into Claude Code]

[Clear, specific instructions]
[Reference CLAUDE.md for conventions]
[List specific outcomes expected]
```

### What You Should Have After
- [ ] [Verifiable checklist item]
- [ ] [Another checklist item]

### Verification Command
```bash
[Command to verify this phase worked]
```

### Common Issues
- If [problem], try [solution]

---

Guidelines for creating phases:

1. **Phase 1 is always Project Setup**: Initialize, install dependencies, verify "hello world"

2. **Order by dependency**: Build foundation before features that depend on it

3. **Each phase = testable milestone**: User should be able to verify before continuing

4. **Prompts should be specific**: 
   - BAD: "Add user authentication"
   - GOOD: "Add user authentication using NextAuth.js with GitHub and email providers. 
     Create a login page at /login, protect the /dashboard route, and add a user 
     dropdown in the header showing the logged-in user's name."

5. **Include file paths**: Tell Claude Code where to create files

6. **Reference other docs**: "See SPEC.md for the data model" / "Follow CLAUDE.md conventions"

7. **Final phase is always Polish**: Error handling, edge cases, cleanup

Based on the conversation, identify the natural phases. Usually:
- Setup (always first)
- Core data/models (if database)
- Core feature 1
- Core feature 2
- ...
- Integration/connecting features
- Polish (always last)

<conversation>
{conversation_history}
</conversation>

Generate the PROMPTS.md now:
```

#### CONVERSATION.md Generation

This is the simplest—it's just a formatted dump of the conversation:

```rust
fn generate_conversation_md(session: &Session, messages: &[Message]) -> String {
    let mut output = format!(
        "# {} - Planning Conversation\n\n\
         This is the complete planning conversation that generated these documents.\n\
         Kept for reference—you can revisit to understand why decisions were made.\n\n\
         ---\n\n\
         **Session started**: {}\n\n\
         ---\n\n",
        session.name,
        session.created_at.format("%B %d, %Y at %H:%M")
    );
    
    for message in messages {
        let role_label = match message.role.as_str() {
            "user" => "**User**",
            "assistant" => "**AuraForge**",
            "system" => continue, // Skip system messages
            _ => "**Unknown**",
        };
        
        output.push_str(&format!("{}: {}\n\n", role_label, message.content));
        
        // Include search context if present
        if let Some(metadata) = &message.metadata {
            if let Some(query) = &metadata.search_query {
                output.push_str(&format!("*[Searched: {}]*\n\n", query));
            }
        }
    }
    
    output.push_str(&format!(
        "---\n\n\
         **Session ended**: {}\n\
         **Documents generated**: {}\n",
        session.updated_at.format("%B %d, %Y at %H:%M"),
        chrono::Local::now().format("%B %d, %Y at %H:%M")
    ));
    
    output
}
```

#### Generation Orchestration

Documents are generated in a specific order to allow cross-referencing:

```rust
pub async fn generate_all_documents(
    app: &tauri::AppHandle,
    state: &AppState,
    session_id: &str,
) -> Result<Vec<GeneratedDocument>, AppError> {
    let messages = state.db.get_messages(session_id)?;
    let conversation = format_conversation_for_prompt(&messages);
    
    let mut documents = Vec::new();
    
    // Order matters: SPEC first (most detail), then others can reference it
    let doc_configs = [
        ("SPEC.md", SPEC_PROMPT),
        ("CLAUDE.md", CLAUDE_PROMPT),
        ("PROMPTS.md", PROMPTS_PROMPT),
        ("README.md", README_PROMPT),
    ];
    
    for (i, (filename, prompt_template)) in doc_configs.iter().enumerate() {
        // Emit progress
        app.emit("generate:progress", GenerateProgress {
            current: i + 1,
            total: doc_configs.len() + 1, // +1 for CONVERSATION.md
            filename: filename.to_string(),
        })?;
        
        let prompt = prompt_template.replace("{conversation_history}", &conversation);
        
        let content = state.llm.generate(&prompt).await?;
        
        // Validate markdown structure (basic check)
        if !content.starts_with('#') {
            // Retry once if format seems wrong
            let content = state.llm.generate(&format!(
                "{}\n\nIMPORTANT: Start with a # heading. Output only valid Markdown.",
                prompt
            )).await?;
        }
        
        let doc = state.db.save_document(session_id, filename, &content)?;
        documents.push(doc);
    }
    
    // CONVERSATION.md is generated from data, not LLM
    let session = state.db.get_session(session_id)?;
    let conversation_md = generate_conversation_md(&session, &messages);
    let doc = state.db.save_document(session_id, "CONVERSATION.md", &conversation_md)?;
    documents.push(doc);
    
    app.emit("generate:complete", documents.len())?;
    
    Ok(documents)
}
```

---

### 4. Session Management

**Purpose**: Track and revisit planning sessions

**Features**:
- **Session List**: Sidebar showing all sessions (newest first)
- **Session Preview**: Name, date, status indicator
- **Resume Session**: Click to continue planning
- **Delete Session**: Remove with confirmation
- **Rename Session**: Edit session name
- **Archive Session**: Hide without deleting

**Auto-naming**: First session message is analyzed to generate a name (e.g., "Habit Tracker App", "E-commerce API")

### 5. Save & Export

**Purpose**: Output documents to file system

**Behavior**:
- Click "Save to Folder" button
- Native file picker opens (default to configured path)
- Creates folder with project name
- Writes all generated documents
- Shows success confirmation with folder path

**Folder Structure Created**:
```
habit-tracker-plan/
├── README.md
├── SPEC.md
├── CLAUDE.md
├── PROMPTS.md
├── CONVERSATION.md
└── [supporting files if applicable]
```

---

## User Interface

### Views

#### 0. First-Run Experience (Onboarding)

Shown on first launch when no config exists or when health check fails.

**Welcome Screen** (Modal, cannot be dismissed until resolved):

```
┌────────────────────────────────────────────────────────────┐
│                                                            │
│                    🔥 Welcome to AuraForge                 │
│                                                            │
│     Before we begin, let's make sure everything is ready   │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ☐  Ollama is running                               │   │
│  │      Status: ❌ Not detected                        │   │
│  │      → Install: brew install ollama && ollama serve │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ☐  Model downloaded                                │   │
│  │      Status: ⏳ Checking...                         │   │
│  │      → Download: ollama pull qwen3-coder:30b        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ☐  Web search (optional)                           │   │
│  │      Status: ⚠️ No API key                          │   │
│  │      → Get free key: tavily.com                     │   │
│  │      [Enter Tavily API Key: ________________]       │   │
│  │      Or: [ ] Skip, use DuckDuckGo fallback          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                            │
│                    [ Check Again ]  [ Continue → ]         │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**State Machine:**

```
                    ┌─────────────┐
                    │   Launch    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ Load Config │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
       Config Exists              No Config
              │                         │
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │Health Check │          │Create Default│
       └──────┬──────┘          └──────┬──────┘
              │                        │
              │                        ▼
              │                 ┌─────────────┐
              │                 │Health Check │
              │                 └──────┬──────┘
              │                        │
              ├────────────────────────┘
              │
    ┌─────────┴─────────┐
    │                   │
    ▼                   ▼
 All Pass           Any Fail
    │                   │
    ▼                   ▼
┌─────────────┐  ┌─────────────┐
│  Main App   │  │  Onboarding │
└─────────────┘  └──────┬──────┘
                        │
                        │ User fixes issues
                        │
                        ▼
                 ┌─────────────┐
                 │ Re-check    │──┐
                 └─────────────┘  │
                        │         │ Still failing
                        │         │
                  All Pass        └──────┐
                        │                │
                        ▼                ▼
                 ┌─────────────┐   (Stay in onboarding)
                 │  Main App   │
                 └─────────────┘
```

**Health Check Details:**

```typescript
interface HealthCheck {
  ollama_running: boolean;
  ollama_url: string;
  model_available: boolean;
  model_name: string;
  model_loading: boolean;
  model_loading_progress?: number;
  tavily_configured: boolean;
  tavily_valid?: boolean;
  database_ok: boolean;
  config_valid: boolean;
  errors: HealthError[];
}

interface HealthError {
  component: 'ollama' | 'model' | 'search' | 'database' | 'config';
  message: string;
  help_url?: string;
  action?: string;  // Command to run
}
```

**Retry Logic:**
- Health check runs every 3 seconds while onboarding is open
- Status indicators update in real-time
- "Check Again" forces immediate re-check
- "Continue" only enabled when ollama_running AND model_available

#### 1. Home / Session List
- List of existing sessions (sorted by last updated)
- "New Project" button (prominent)
- Each session shows: name, date, status badge
- Click session to open
- Search/filter sessions (future enhancement)

#### 2. Chat View (Main)
- Message history (scrollable)
- Input field at bottom
- Send button (and Cmd+Enter)
- "Generate Plan" button (appears after sufficient conversation)
- Settings gear icon
- Current session name in header

#### 3. Preview View
- Tabbed interface for generated documents
- Markdown rendering with syntax highlighting
- "Save to Folder" button
- "Regenerate" button (if conversation continued)
- Copy individual document to clipboard

#### 4. Settings Modal
- LLM configuration (model, temperature)
- Web search toggle and provider
- Theme selection
- Default save path
- About / version info

### Navigation

- **Sidebar**: Session list (collapsible)
- **Main Area**: Chat or Preview (toggle via tabs/buttons)
- **No complex routing**: Single-page feel with view switching

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+N` | New session |
| `Cmd+Enter` | Send message |
| `Cmd+G` | Generate documents (when available) |
| `Cmd+S` | Save to folder (when documents exist) |
| `Cmd+,` | Open settings |
| `Escape` | Close modal / cancel |

---

## Design Direction

> **Note**: Full design specifications will be developed in a dedicated design phase. 
> This section captures the intended direction.

### Theme: "The Forge"

**Inspiration**: Futuristic Sauron forging the One Ring. Dark, atmospheric, powerful. The sense of creation, of bringing something into existence from raw thought.

**Visual Elements**:
- **Dark mode default**: Deep, rich blacks and near-blacks
- **Accent colors**: Glowing embers, molten oranges, ethereal purples
- **Typography**: Clean, modern, slightly futuristic
- **Animations**: Subtle glows, particles, warmth effects
- **Iconography**: Custom icons with forge/creation theme

**Mood Board Keywords**:
- Forge fire glow
- Ancient meets futuristic
- Power and precision
- Mystical but professional
- Creating something from nothing

**UI Tone**:
- Messages emerge from darkness
- Generate button has "forging" animation
- Documents appear as if being inscribed
- Subtle ambient effects (optional, performance-conscious)

### Accessibility

- High contrast text
- Keyboard fully navigable
- Screen reader compatible
- Reduced motion option

---

## LLM System Prompt

The following system prompt defines AuraForge's conversational personality:

```
You are AuraForge, a senior engineering assistant specialized in project planning 
for AI-assisted development workflows. Your role is to help users transform 
project ideas into comprehensive, actionable plans.

## Your Personality

- **Friendly and supportive**: Encourage good ideas, celebrate clarity
- **Challenging when needed**: Push back on weak assumptions, ask probing questions
- **Alternatives-focused**: When you see a better approach, suggest it
- **Practical**: Focus on what actually works, not theoretical perfection
- **Honest**: If something is too complex or a bad idea, say so kindly

## Your Approach

1. **Understand first**: Ask clarifying questions before suggesting solutions
2. **Challenge assumptions**: "Are you sure you need X?" / "What if you just..."
3. **Offer trade-offs**: Present options with pros/cons, let user decide
4. **Stay grounded**: Use web search to verify current best practices
5. **Track progress**: Mentally note what's decided vs. what's open

## What You Help With

- Clarifying project scope and goals
- Choosing appropriate tech stacks
- Designing system architecture
- Breaking down features into phases
- Identifying potential challenges early
- Creating actionable implementation plans

## What You Don't Do

- Write actual code (that's Claude Code's job)
- Make decisions for the user without discussion
- Over-engineer simple projects
- Recommend technologies without justification
- Rush to solutions before understanding the problem

## Web Search

You have access to web search. Use it proactively when:
- User discusses technology choices
- Current best practices would be helpful
- You need to verify if something is maintained/current
- User asks about specific tools or libraries

When you search, briefly mention it: "[Searching: query...]"

## Conversation Flow

1. User describes idea → Ask clarifying questions
2. Understand scope → Suggest approach, get feedback
3. Refine together → Challenge, offer alternatives
4. Reach decisions → Confirm understanding
5. When solid → User triggers document generation

## Output

When asked to generate documents, you produce:
- README.md: Overview and usage instructions
- SPEC.md: Comprehensive specification
- CLAUDE.md: Claude Code configuration
- PROMPTS.md: Phased implementation guide
- CONVERSATION.md: Full transcript

Each document should be detailed enough that someone could execute 
the plan without needing to ask follow-up questions.
```

---

## Implementation Phases

### Phase 1: Foundation (6-8 hours)

**Objective**: Basic Tauri app structure, database, and Ollama integration

**Tasks**:
1. Initialize Tauri 2.0 project with React + TypeScript
2. Configure Tailwind CSS with dark mode
3. Set up project structure:
   ```
   auraforge/
   ├── src/                    # React frontend
   │   ├── components/
   │   ├── views/
   │   ├── stores/
   │   ├── hooks/
   │   ├── utils/
   │   └── App.tsx
   ├── src-tauri/              # Rust backend
   │   ├── src/
   │   │   ├── main.rs
   │   │   ├── commands/
   │   │   ├── db/
   │   │   ├── llm/
   │   │   └── search/
   │   └── Cargo.toml
   └── package.json
   ```
4. Set up SQLite database with rusqlite
5. Create database migrations and schema
6. Implement basic Ollama client (send message, receive response)
7. Create config file handling
8. Build simple "echo" test to verify IPC works

**Verification**:
- [ ] `npm run tauri dev` launches app
- [ ] Can send message to Ollama and receive response
- [ ] Database creates on first run
- [ ] Config file loads/saves

### Phase 2: Chat Interface (6-8 hours)

**Objective**: Functional chat UI with session management

**Tasks**:
1. Create Zustand store for conversation state
2. Build message components (user bubble, assistant bubble)
3. Build chat input component with send button
4. Implement message scrolling (auto-scroll to bottom)
5. Add "thinking" indicator during LLM response
6. Implement session CRUD (create, read, update, delete)
7. Build session list sidebar
8. Connect frontend to Tauri commands
9. Handle streaming responses from Ollama

**Verification**:
- [ ] Can start new session
- [ ] Messages display correctly
- [ ] Session persists after app restart
- [ ] Can switch between sessions
- [ ] Can delete session

### Phase 3: Web Search (4-6 hours)

**Objective**: Integrated web search with proactive triggering

**Tasks**:
1. Implement Tavily API client in Rust
2. Implement DuckDuckGo fallback (direct HTTP)
3. Add search indicator to chat UI
4. Integrate search into LLM conversation flow
5. Implement proactive search detection (keyword triggers)
6. Store search results in message metadata
7. Add search enable/disable setting

**Verification**:
- [ ] Tavily search returns results
- [ ] DuckDuckGo fallback works when Tavily unavailable
- [ ] Search results appear in conversation context
- [ ] Search can be disabled in settings

### Phase 4: Document Generation (8-10 hours)

**Objective**: Generate all planning documents from conversation

**Tasks**:
1. Design document generation prompts
2. Implement README.md generator
3. Implement SPEC.md generator
4. Implement CLAUDE.md generator
5. Implement PROMPTS.md generator
6. Implement CONVERSATION.md generator
7. Implement supporting file detection and generation
8. Build preview view with tabbed documents
9. Add markdown rendering with syntax highlighting
10. Implement document caching in database

**Verification**:
- [ ] All documents generate correctly
- [ ] Documents are contextually accurate
- [ ] Preview displays with proper formatting
- [ ] Regeneration updates cached documents

### Phase 5: Save & Export (3-4 hours)

**Objective**: Save generated documents to file system

**Tasks**:
1. Implement native folder picker dialog
2. Create folder with project name
3. Write all documents to folder
4. Handle file system errors gracefully
5. Add success/error notifications
6. Implement "copy to clipboard" for individual docs

**Verification**:
- [ ] Folder picker opens correctly
- [ ] All files written to chosen location
- [ ] Files contain correct content
- [ ] Error states handled gracefully

### Phase 6: Design Implementation (8-12 hours)

> **Note**: This phase follows the dedicated design session

**Objective**: Apply final visual design

**Tasks**:
1. Implement color palette and theming
2. Build custom components per design specs
3. Add animations and transitions
4. Implement "forge" visual effects
5. Polish all UI states
6. Ensure dark mode is cohesive
7. Test and refine interactions

**Verification**:
- [ ] Matches approved designs
- [ ] Animations are smooth
- [ ] Accessibility requirements met
- [ ] Performance is acceptable

### Phase 7: Polish & Testing (4-6 hours)

**Objective**: Bug fixes, edge cases, final polish

**Tasks**:
1. Handle edge cases (empty states, errors, long messages)
2. Add loading states throughout
3. Improve error messages
4. Test with various conversation lengths
5. Test document generation with different project types
6. Performance optimization if needed
7. Final cleanup and code organization

**Verification**:
- [ ] No console errors
- [ ] All edge cases handled
- [ ] Performance acceptable on target hardware
- [ ] Ready for personal use

---

## Test Strategy

### Test File Structure

```
auraforge/
├── src/
│   └── __tests__/                    # Frontend tests
│       ├── components/
│       │   ├── ChatMessage.test.tsx
│       │   ├── ChatInput.test.tsx
│       │   ├── SessionList.test.tsx
│       │   └── DocumentPreview.test.tsx
│       ├── hooks/
│       │   ├── useChat.test.ts
│       │   └── useSession.test.ts
│       └── stores/
│           └── chatStore.test.ts
├── src-tauri/
│   └── src/
│       ├── commands/
│       │   └── tests.rs              # Command integration tests
│       ├── db/
│       │   └── tests.rs              # Database tests
│       ├── llm/
│       │   └── tests.rs              # LLM client tests (mocked)
│       └── search/
│           └── tests.rs              # Search client tests (mocked)
└── e2e/                              # End-to-end tests
    ├── session.spec.ts
    ├── conversation.spec.ts
    └── document-generation.spec.ts
```

### Rust Backend Tests

#### Database Tests

```rust
// src-tauri/src/db/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        Database::new(&db_path).unwrap()
    }

    #[test]
    fn test_create_session() {
        let db = setup_test_db();
        let session = db.create_session(Some("Test Project")).unwrap();
        
        assert_eq!(session.name, "Test Project");
        assert_eq!(session.status, "active");
        assert!(!session.id.is_empty());
    }

    #[test]
    fn test_session_auto_naming() {
        let db = setup_test_db();
        let session = db.create_session(None).unwrap();
        
        assert!(session.name.starts_with("New Project"));
    }

    #[test]
    fn test_message_persistence() {
        let db = setup_test_db();
        let session = db.create_session(Some("Test")).unwrap();
        
        let msg = db.save_message(&session.id, "user", "Hello", None).unwrap();
        let messages = db.get_messages(&session.id).unwrap();
        
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[0].role, "user");
    }

    #[test]
    fn test_cascade_delete() {
        let db = setup_test_db();
        let session = db.create_session(Some("Test")).unwrap();
        db.save_message(&session.id, "user", "Hello", None).unwrap();
        db.save_document(&session.id, "README.md", "# Test").unwrap();
        
        db.delete_session(&session.id).unwrap();
        
        let messages = db.get_messages(&session.id).unwrap();
        let documents = db.get_documents(&session.id).unwrap();
        
        assert!(messages.is_empty());
        assert!(documents.is_empty());
    }

    #[test]
    fn test_document_update() {
        let db = setup_test_db();
        let session = db.create_session(Some("Test")).unwrap();
        
        db.save_document(&session.id, "README.md", "Version 1").unwrap();
        db.save_document(&session.id, "README.md", "Version 2").unwrap();
        
        let docs = db.get_documents(&session.id).unwrap();
        
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].content, "Version 2");
    }
}
```

#### LLM Client Tests (Mocked)

```rust
// src-tauri/src/llm/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_ollama_health_check_success() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/api/tags")
            .with_status(200)
            .with_body(r#"{"models":[{"name":"qwen3-coder:30b"}]}"#)
            .create_async()
            .await;
        
        let client = OllamaClient::new(&server.url());
        let health = client.check_health("qwen3-coder:30b").await.unwrap();
        
        assert!(health.connected);
        assert!(health.model_available);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_ollama_connection_refused() {
        let client = OllamaClient::new("http://localhost:99999");
        let result = client.check_health("test").await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            LLMError::Connection { .. } => {}
            other => panic!("Expected Connection error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_ollama_model_not_found() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/api/tags")
            .with_status(200)
            .with_body(r#"{"models":[]}"#)
            .create_async()
            .await;
        
        let client = OllamaClient::new(&server.url());
        let health = client.check_health("nonexistent").await.unwrap();
        
        assert!(health.connected);
        assert!(!health.model_available);
    }
}
```

#### Search Client Tests

```rust
// src-tauri/src/search/tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_trigger_comparison() {
        assert!(should_search("Should I use React vs Vue?").is_some());
        assert!(should_search("React versus Angular for my project").is_some());
    }

    #[test]
    fn test_search_trigger_best_practices() {
        assert!(should_search("What's the best practice for React folder structure?").is_some());
        assert!(should_search("Recommended approach for database migrations?").is_some());
    }

    #[test]
    fn test_search_trigger_maintenance() {
        assert!(should_search("Is moment.js still maintained?").is_some());
        assert!(should_search("Is lodash deprecated?").is_some());
    }

    #[test]
    fn test_no_search_trigger_general() {
        assert!(should_search("I want to build a todo app").is_none());
        assert!(should_search("Let's use React for the frontend").is_none());
        assert!(should_search("How many users do you expect?").is_none());
    }

    #[tokio::test]
    async fn test_tavily_rate_limit_fallback() {
        // Mock Tavily returning 429
        // Verify DuckDuckGo fallback is triggered
    }
}
```

### Frontend Tests (Vitest + React Testing Library)

```typescript
// src/__tests__/components/ChatMessage.test.tsx

import { render, screen } from '@testing-library/react';
import { ChatMessage } from '../../components/ChatMessage';

describe('ChatMessage', () => {
  it('renders user message on the right', () => {
    render(<ChatMessage role="user" content="Hello" />);
    
    const message = screen.getByText('Hello');
    expect(message.closest('.message')).toHaveClass('user');
  });

  it('renders assistant message on the left with glow', () => {
    render(<ChatMessage role="assistant" content="Hi there!" />);
    
    const message = screen.getByText('Hi there!');
    expect(message.closest('.message')).toHaveClass('ai');
  });

  it('renders markdown in assistant messages', () => {
    render(<ChatMessage role="assistant" content="**Bold** and `code`" />);
    
    expect(screen.getByText('Bold')).toHaveClass('font-bold');
    expect(screen.getByText('code')).toHaveClass('font-mono');
  });

  it('shows streaming indicator when streaming', () => {
    render(<ChatMessage role="assistant" content="Typing..." isStreaming />);
    
    expect(screen.getByTestId('streaming-cursor')).toBeInTheDocument();
  });
});
```

```typescript
// src/__tests__/hooks/useChat.test.ts

import { renderHook, act, waitFor } from '@testing-library/react';
import { useChat } from '../../hooks/useChat';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('useChat', () => {
  it('sends message and enters streaming state', async () => {
    const { result } = renderHook(() => useChat('session-123'));
    
    act(() => {
      result.current.sendMessage('Hello');
    });
    
    expect(result.current.isStreaming).toBe(true);
    expect(result.current.streamingContent).toBe('');
  });

  it('accumulates streaming content', async () => {
    const { result } = renderHook(() => useChat('session-123'));
    
    // Simulate stream chunks via mocked event listener
    act(() => {
      result.current.handleStreamChunk({ type: 'content', content: 'Hello' });
      result.current.handleStreamChunk({ type: 'content', content: ' world' });
    });
    
    expect(result.current.streamingContent).toBe('Hello world');
  });

  it('exits streaming state on done', async () => {
    const { result } = renderHook(() => useChat('session-123'));
    
    act(() => {
      result.current.sendMessage('Hello');
    });
    
    act(() => {
      result.current.handleStreamDone();
    });
    
    expect(result.current.isStreaming).toBe(false);
  });
});
```

```typescript
// src/__tests__/stores/chatStore.test.ts

import { useChatStore } from '../../stores/chatStore';

describe('chatStore', () => {
  beforeEach(() => {
    useChatStore.getState().reset();
  });

  it('adds messages to current session', () => {
    const store = useChatStore.getState();
    
    store.setCurrentSession('session-123');
    store.addMessage({
      id: 'msg-1',
      session_id: 'session-123',
      role: 'user',
      content: 'Hello',
      metadata: null,
      created_at: new Date().toISOString(),
    });
    
    expect(store.messages).toHaveLength(1);
    expect(store.messages[0].content).toBe('Hello');
  });

  it('clears messages when switching sessions', () => {
    const store = useChatStore.getState();
    
    store.setCurrentSession('session-1');
    store.addMessage({ /* ... */ });
    store.setCurrentSession('session-2');
    
    expect(store.messages).toHaveLength(0);
  });
});
```

### End-to-End Tests (Playwright + Tauri)

```typescript
// e2e/session.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Session Management', () => {
  test('creates new session and displays in sidebar', async ({ page }) => {
    await page.goto('/');
    
    await page.click('[data-testid="new-session-btn"]');
    
    const sidebar = page.locator('[data-testid="session-list"]');
    await expect(sidebar.locator('.session-item')).toHaveCount(1);
  });

  test('renames session via double-click', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="new-session-btn"]');
    
    const sessionItem = page.locator('.session-item').first();
    await sessionItem.dblclick();
    
    await page.keyboard.type('My Test Project');
    await page.keyboard.press('Enter');
    
    await expect(sessionItem).toContainText('My Test Project');
  });

  test('deletes session with confirmation', async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="new-session-btn"]');
    
    const sessionItem = page.locator('.session-item').first();
    await sessionItem.hover();
    await sessionItem.locator('[data-testid="delete-btn"]').click();
    
    // Confirm dialog
    await page.click('[data-testid="confirm-delete"]');
    
    await expect(page.locator('.session-item')).toHaveCount(0);
  });
});
```

```typescript
// e2e/conversation.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Conversation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('[data-testid="new-session-btn"]');
  });

  test('sends message and receives streaming response', async ({ page }) => {
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('I want to build a todo app');
    await input.press('Meta+Enter');
    
    // User message appears
    await expect(page.locator('.message.user')).toContainText('todo app');
    
    // Thinking indicator appears
    await expect(page.locator('[data-testid="thinking-indicator"]')).toBeVisible();
    
    // AI response appears (wait for streaming to complete)
    await expect(page.locator('.message.ai')).toBeVisible({ timeout: 30000 });
    await expect(page.locator('[data-testid="thinking-indicator"]')).not.toBeVisible();
  });

  test('shows search indicator when search is triggered', async ({ page }) => {
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Should I use React vs Vue for a dashboard?');
    await input.press('Meta+Enter');
    
    // Search indicator should appear
    await expect(page.locator('[data-testid="search-indicator"]')).toBeVisible();
    await expect(page.locator('[data-testid="search-indicator"]')).toContainText('Searching');
  });

  test('handles Ollama connection error gracefully', async ({ page }) => {
    // Simulate Ollama being down (would need test fixture)
    const input = page.locator('[data-testid="chat-input"]');
    await input.fill('Hello');
    await input.press('Meta+Enter');
    
    // Error banner should appear
    await expect(page.locator('[data-testid="error-banner"]')).toContainText('Ollama');
    await expect(page.locator('[data-testid="retry-btn"]')).toBeVisible();
  });
});
```

### Key Test Scenarios Checklist

#### Happy Path
- [ ] Create session → send message → receive response → generate docs → save to folder
- [ ] Resume existing session, conversation context preserved
- [ ] Web search triggers and results appear in context
- [ ] All 5 document types generate with valid markdown

#### Error Handling
- [ ] Ollama not running → shows connection error with retry
- [ ] Model not found → shows download instructions
- [ ] Stream interrupted → shows partial response with retry option
- [ ] Tavily rate limited → falls back to DuckDuckGo silently
- [ ] Invalid config → shows specific validation error
- [ ] Disk full → shows storage error

#### Edge Cases
- [ ] Empty session (no messages) → shows empty state
- [ ] Very long conversation (100+ messages) → scrolling works, no memory issues
- [ ] Very long single message → renders without breaking layout
- [ ] Special characters in project name → sanitized for folder creation
- [ ] Folder already exists → prompts for overwrite or rename
- [ ] Rapid message sending → queues properly, no race conditions

#### Accessibility
- [ ] All interactive elements focusable via Tab
- [ ] Screen reader announces new messages
- [ ] Focus returns to input after sending
- [ ] Error messages announced to screen reader

---

## Success Criteria

### Functional Requirements

- [ ] Can have multi-turn planning conversations
- [ ] AI challenges weak ideas and offers alternatives
- [ ] Web search provides current information
- [ ] Generates all five core documents
- [ ] Documents are detailed and actionable
- [ ] Can save to any folder
- [ ] Sessions persist and can be resumed
- [ ] Works fully offline (except web search)

### Quality Requirements

- [ ] Response time < 100ms for UI interactions
- [ ] LLM response starts streaming within 2 seconds
- [ ] Document generation completes within 30 seconds
- [ ] No crashes during normal usage
- [ ] Graceful degradation if Ollama unavailable

### User Experience Requirements

- [ ] Intuitive without documentation
- [ ] Dark mode is visually appealing
- [ ] Keyboard shortcuts work as expected
- [ ] Clear feedback for all actions

---

## Future Enhancements (Out of Scope for v1)

- Windows/Linux builds
- Cloud sync for sessions
- Multiple LLM provider UI selection
- Template library for common project types
- Import existing code for CLAUDE.md generation
- Collaboration features
- Voice input
- Mobile companion app

---

## Dependencies & Prerequisites

### User Must Have

1. **macOS** (Apple Silicon recommended, Intel supported)
2. **Ollama** installed and running
3. **Qwen3-Coder-30B-A3B-Instruct** model downloaded
4. **Tavily API key** (optional, for web search)

### Development Dependencies

- Node.js 20+
- Rust (latest stable)
- Xcode Command Line Tools (for Tauri build)

### Runtime Dependencies (Bundled)

- SQLite (via rusqlite)
- All React dependencies
- Tailwind CSS

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ollama not running | App unusable | Clear error message, setup instructions |
| Model too slow | Poor UX | Recommend appropriate model sizes, show progress |
| Web search rate limited | Reduced functionality | DuckDuckGo fallback, graceful degradation |
| Document generation hallucinations | Poor output | Structured prompts, validation, regenerate option |
| Large conversation history | Memory issues | Pagination, summarization for very long sessions |

---

## Appendix: File Locations

| File | Location |
|------|----------|
| Config | `~/.auraforge/config.yaml` |
| Database | `~/.auraforge/auraforge.db` |
| Logs | `~/.auraforge/logs/` |
| Cache | `~/.auraforge/cache/` |

---

## Next Steps

1. **Design Phase**: Dedicated session to develop full visual design
2. **Design Approval**: Review and finalize design decisions
3. **Implementation**: Execute phases 1-7 in Claude Code
4. **Testing**: Personal use testing and iteration
5. **Distribution**: Package .dmg for sharing

---

*This specification is ready for the design phase. All functional requirements are locked.*
