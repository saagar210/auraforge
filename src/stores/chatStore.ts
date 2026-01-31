import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Session,
  Message,
  StreamChunk,
  SearchResult,
  AppConfig,
  SearchConfig,
  HealthStatus,
  GeneratedDocument,
  GenerateProgress,
  ModelPullProgress,
  OnboardingStep,
} from "../types";

interface ChatState {
  // Sessions
  sessions: Session[];
  currentSessionId: string | null;

  // Messages
  messages: Message[];

  // Streaming
  isStreaming: boolean;
  streamingContent: string;
  streamError: string | null;

  // Search
  searchQuery: string | null;
  searchResults: SearchResult[] | null;

  // Document generation
  documents: GeneratedDocument[];
  isGenerating: boolean;
  generateProgress: GenerateProgress | null;
  documentsStale: boolean;
  showPreview: boolean;

  // Health
  healthStatus: HealthStatus | null;
  onboardingDismissed: boolean;

  // Wizard / Onboarding
  wizardCompleted: boolean;
  wizardStep: OnboardingStep;
  modelPullProgress: ModelPullProgress | null;
  isModelPulling: boolean;
  installedModels: string[];
  isFirstSession: boolean;

  // UI
  showSettings: boolean;
  setShowSettings: (show: boolean) => void;
  showHelp: boolean;
  sidebarCollapsed: boolean;

  // Loading
  sessionsLoading: boolean;
  messagesLoading: boolean;

  // Actions
  checkHealth: () => Promise<HealthStatus | null>;
  dismissOnboarding: () => void;
  retryLastMessage: () => Promise<void>;
  loadSessions: () => Promise<void>;
  createSession: () => Promise<Session | null>;
  selectSession: (sessionId: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<void>;
  renameSession: (sessionId: string, name: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  clearStreamError: () => void;

  // Wizard actions
  loadPreferences: () => Promise<void>;
  setWizardStep: (step: OnboardingStep) => void;
  completeWizard: () => Promise<void>;
  listModels: () => Promise<string[]>;
  pullModel: (name: string) => Promise<void>;
  cancelPullModel: () => Promise<void>;
  setShowHelp: (show: boolean) => void;
  toggleSidebar: () => void;
  markFirstSessionComplete: () => Promise<void>;

  // Config actions
  loadConfig: () => Promise<AppConfig | null>;
  updateSearchConfig: (config: SearchConfig) => Promise<void>;
  updateConfig: (updates: {
    llm?: import("../types").LLMConfig;
    search?: SearchConfig;
    ui?: import("../types").UIConfig;
    output?: import("../types").OutputConfig;
  }) => Promise<AppConfig | null>;

  // Document actions
  generateDocuments: () => Promise<void>;
  loadDocuments: () => Promise<void>;
  checkStale: () => Promise<void>;
  setShowPreview: (show: boolean) => void;

  // Export actions
  saveToFolder: (folderPath: string) => Promise<string | null>;
  openFolder: (path: string) => Promise<void>;

  // Toast
  toast: { message: string; type: "success" | "error"; actionPath?: string } | null;
  dismissToast: () => void;

  // Event listeners
  _unlisteners: UnlistenFn[];
  initEventListeners: () => Promise<void>;
  cleanupEventListeners: () => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  sessions: [],
  currentSessionId: null,
  messages: [],
  isStreaming: false,
  streamingContent: "",
  streamError: null,
  searchQuery: null,
  searchResults: null,
  documents: [],
  isGenerating: false,
  generateProgress: null,
  documentsStale: false,
  showPreview: false,
  toast: null,
  healthStatus: null,
  onboardingDismissed: false,
  wizardCompleted: false,
  wizardStep: "welcome" as OnboardingStep,
  modelPullProgress: null,
  isModelPulling: false,
  installedModels: [],
  isFirstSession: true,
  showSettings: false,
  showHelp: false,
  sidebarCollapsed: false,
  sessionsLoading: false,
  messagesLoading: false,
  _unlisteners: [],

  setShowSettings: (show: boolean) => set({ showSettings: show }),

  checkHealth: async () => {
    try {
      const status = await invoke<HealthStatus>("check_health");
      set({ healthStatus: status });
      return status;
    } catch (e) {
      console.error("Health check failed:", e);
      return null;
    }
  },

  dismissOnboarding: () => set({ onboardingDismissed: true }),

  retryLastMessage: async () => {
    const { messages, currentSessionId, isStreaming } = get();
    if (!currentSessionId || isStreaming) return;

    const lastUserMsg = [...messages].reverse().find((m) => m.role === "user");
    if (!lastUserMsg) return;

    set({
      isStreaming: true,
      streamingContent: "",
      streamError: null,
      searchQuery: null,
      searchResults: null,
    });

    try {
      await invoke<Message>("send_message", {
        sessionId: currentSessionId,
        content: lastUserMsg.content,
        retry: true,
      });

      if (get().currentSessionId !== currentSessionId) return;

      const allMessages = await invoke<Message[]>("get_messages", {
        sessionId: currentSessionId,
      });

      if (get().currentSessionId !== currentSessionId) return;

      set({
        messages: allMessages,
        isStreaming: false,
        streamingContent: "",
        searchQuery: null,
        searchResults: null,
      });
    } catch (e) {
      if (get().currentSessionId === currentSessionId) {
        set({
          isStreaming: false,
          streamError: String(e),
          searchQuery: null,
          searchResults: null,
        });
      }
    }
  },

  loadSessions: async () => {
    set({ sessionsLoading: true });
    try {
      const sessions = await invoke<Session[]>("get_sessions");
      set({ sessions, sessionsLoading: false });
    } catch (e) {
      console.error("Failed to load sessions:", e);
      set({ sessionsLoading: false });
    }
  },

  createSession: async () => {
    try {
      const session = await invoke<Session>("create_session", { name: null });
      set((state) => ({
        sessions: [session, ...state.sessions],
        currentSessionId: session.id,
        messages: [],
        streamingContent: "",
        streamError: null,
        documents: [],
        showPreview: false,
        documentsStale: false,
      }));
      return session;
    } catch (e) {
      console.error("Failed to create session:", e);
      return null;
    }
  },

  selectSession: async (sessionId: string) => {
    if (get().currentSessionId === sessionId) return;
    set({
      currentSessionId: sessionId,
      messages: [],
      messagesLoading: true,
      streamingContent: "",
      streamError: null,
      isStreaming: false,
      searchQuery: null,
      searchResults: null,
      documents: [],
      showPreview: false,
      documentsStale: false,
      isGenerating: false,
      generateProgress: null,
    });
    try {
      const messages = await invoke<Message[]>("get_messages", {
        sessionId,
      });
      set({ messages, messagesLoading: false });

      // Load cached documents if any
      get().loadDocuments();
    } catch (e) {
      console.error("Failed to load messages:", e);
      set({ messagesLoading: false });
    }
  },

  deleteSession: async (sessionId: string) => {
    try {
      await invoke("delete_session", { sessionId });
      set((state) => {
        const sessions = state.sessions.filter((s) => s.id !== sessionId);
        const newCurrentId =
          state.currentSessionId === sessionId
            ? sessions[0]?.id ?? null
            : state.currentSessionId;
        return {
          sessions,
          currentSessionId: newCurrentId,
          messages:
            state.currentSessionId === sessionId ? [] : state.messages,
          documents:
            state.currentSessionId === sessionId ? [] : state.documents,
          showPreview: state.currentSessionId === sessionId ? false : state.showPreview,
        };
      });
      const newId = get().currentSessionId;
      if (newId && newId !== sessionId) {
        get().selectSession(newId);
      }
    } catch (e) {
      console.error("Failed to delete session:", e);
    }
  },

  renameSession: async (sessionId: string, name: string) => {
    try {
      const updated = await invoke<Session>("update_session", {
        sessionId,
        name,
        status: null,
      });
      set((state) => ({
        sessions: state.sessions.map((s) =>
          s.id === sessionId ? updated : s,
        ),
      }));
    } catch (e) {
      console.error("Failed to rename session:", e);
    }
  },

  sendMessage: async (content: string) => {
    const sessionId = get().currentSessionId;
    if (!sessionId || get().isStreaming) return;

    set({
      isStreaming: true,
      streamingContent: "",
      streamError: null,
      searchQuery: null,
      searchResults: null,
      showPreview: false,
    });

    try {
      const userMsg = await invoke<Message>("send_message", {
        sessionId,
        content,
      });

      // Guard: if user switched sessions during the await, discard result
      if (get().currentSessionId !== sessionId) return;

      set((state) => {
        const exists = state.messages.some((m) => m.id === userMsg.id);
        return exists ? {} : { messages: [...state.messages, userMsg] };
      });

      const allMessages = await invoke<Message[]>("get_messages", {
        sessionId,
      });

      // Guard again after second await
      if (get().currentSessionId !== sessionId) return;

      set({
        messages: allMessages,
        isStreaming: false,
        streamingContent: "",
        searchQuery: null,
        searchResults: null,
      });

      // Mark documents as stale if they exist
      if (get().documents.length > 0) {
        set({ documentsStale: true });
      }

      get().loadSessions();
    } catch (e) {
      // Only set error if we're still on the same session
      if (get().currentSessionId === sessionId) {
        set({
          isStreaming: false,
          streamError: String(e),
          searchQuery: null,
          searchResults: null,
        });
      }
    }
  },

  clearStreamError: () => set({ streamError: null }),

  loadPreferences: async () => {
    try {
      const wizardDone = await invoke<string | null>("get_preference", {
        key: "wizard_completed",
      });
      const firstSessionDone = await invoke<string | null>("get_preference", {
        key: "first_session_completed",
      });
      set({
        wizardCompleted: wizardDone === "true",
        onboardingDismissed: wizardDone === "true",
        isFirstSession: firstSessionDone !== "true",
      });
    } catch (e) {
      console.error("Failed to load preferences:", e);
    }
  },

  setWizardStep: (step: OnboardingStep) => set({ wizardStep: step }),

  completeWizard: async () => {
    try {
      await invoke("set_preference", {
        key: "wizard_completed",
        value: "true",
      });
      set({ wizardCompleted: true, onboardingDismissed: true });
    } catch (e) {
      console.error("Failed to complete wizard:", e);
    }
  },

  listModels: async () => {
    try {
      const models = await invoke<string[]>("list_models");
      set({ installedModels: models });
      return models;
    } catch (e) {
      console.error("Failed to list models:", e);
      return [];
    }
  },

  pullModel: async (name: string) => {
    set({ isModelPulling: true, modelPullProgress: null });
    try {
      await invoke("pull_model", { modelName: name });
    } catch (e) {
      console.error("Model pull failed:", e);
      set({ isModelPulling: false });
    }
  },

  cancelPullModel: async () => {
    try {
      await invoke("cancel_pull_model");
      set({ isModelPulling: false, modelPullProgress: null });
    } catch (e) {
      console.error("Failed to cancel pull:", e);
    }
  },

  setShowHelp: (show: boolean) => set({ showHelp: show }),

  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  markFirstSessionComplete: async () => {
    try {
      await invoke("set_preference", {
        key: "first_session_completed",
        value: "true",
      });
      set({ isFirstSession: false });
    } catch (e) {
      console.error("Failed to mark first session complete:", e);
    }
  },

  loadConfig: async () => {
    try {
      return await invoke<AppConfig>("get_config");
    } catch (e) {
      console.error("Failed to load config:", e);
      return null;
    }
  },

  updateSearchConfig: async (searchConfig: SearchConfig) => {
    try {
      await invoke("update_search_config", { searchConfig });
    } catch (e) {
      console.error("Failed to update search config:", e);
    }
  },

  updateConfig: async (updates) => {
    try {
      return await invoke<AppConfig>("update_config", updates);
    } catch (e) {
      console.error("Failed to update config:", e);
      return null;
    }
  },

  // ============ DOCUMENT GENERATION ============

  generateDocuments: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId || get().isGenerating) return;

    set({ isGenerating: true, generateProgress: null });

    try {
      const documents = await invoke<GeneratedDocument[]>(
        "generate_documents",
        { sessionId },
      );
      set({
        documents,
        isGenerating: false,
        generateProgress: null,
        documentsStale: false,
        showPreview: true,
      });
    } catch (e) {
      console.error("Failed to generate documents:", e);
      set({
        isGenerating: false,
        generateProgress: null,
        streamError: `Document generation failed: ${String(e)}`,
      });
    }
  },

  loadDocuments: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return;

    try {
      const documents = await invoke<GeneratedDocument[]>("get_documents", {
        sessionId,
      });
      if (documents.length > 0) {
        set({ documents });
        // Check staleness
        get().checkStale();
      }
    } catch (e) {
      console.error("Failed to load documents:", e);
    }
  },

  checkStale: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return;

    try {
      const stale = await invoke<boolean>("check_documents_stale", {
        sessionId,
      });
      set({ documentsStale: stale });
    } catch (e) {
      console.error("Failed to check staleness:", e);
    }
  },

  setShowPreview: (show: boolean) => set({ showPreview: show }),

  // ============ EXPORT ============

  saveToFolder: async (folderPath: string) => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const savedPath = await invoke<string>("save_to_folder", {
        sessionId,
        folderPath,
      });
      set({
        toast: {
          message: `Plan saved to ${savedPath}`,
          type: "success",
          actionPath: savedPath,
        },
      });
      return savedPath;
    } catch (e) {
      set({
        toast: {
          message: String(e),
          type: "error",
        },
      });
      return null;
    }
  },

  openFolder: async (path: string) => {
    try {
      await invoke("open_folder", { path });
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  },

  dismissToast: () => set({ toast: null }),

  // ============ EVENT LISTENERS ============

  initEventListeners: async () => {
    const unlisteners: UnlistenFn[] = [];

    const unlChunk = await listen<StreamChunk>("stream:chunk", (event) => {
      const { type, content, search_query, search_results } = event.payload;

      if (type === "content" && content) {
        set((state) => ({
          streamingContent: state.streamingContent + content,
        }));
      } else if (type === "search_start" && search_query) {
        set({ searchQuery: search_query });
      } else if (type === "search_result" && search_results) {
        set({ searchResults: search_results });
      }
    });
    unlisteners.push(unlChunk);

    const unlError = await listen<StreamChunk>("stream:error", (event) => {
      set({
        isStreaming: false,
        streamError: event.payload.error ?? "Unknown streaming error",
        searchQuery: null,
        searchResults: null,
      });
    });
    unlisteners.push(unlError);

    const unlProgress = await listen<GenerateProgress>(
      "generate:progress",
      (event) => {
        set({ generateProgress: event.payload });
      },
    );
    unlisteners.push(unlProgress);

    const unlComplete = await listen<number>("generate:complete", () => {
      // generateDocuments handles the final state, but ensure progress clears
      set({ generateProgress: null });
    });
    unlisteners.push(unlComplete);

    const unlPullProgress = await listen<ModelPullProgress>(
      "model:pull_progress",
      (event) => {
        const progress = event.payload;
        set({ modelPullProgress: progress });
        if (
          progress.status === "success" ||
          progress.status === "cancelled" ||
          progress.status.startsWith("error")
        ) {
          set({ isModelPulling: false });
        }
      },
    );
    unlisteners.push(unlPullProgress);

    const unlMenu = await listen<string>("menu:action", (event) => {
      const action = event.payload;
      const store = get();
      switch (action) {
        case "new_session":
          store.createSession();
          break;
        case "toggle_sidebar":
          store.toggleSidebar();
          break;
        case "toggle_preview":
          store.setShowPreview(!store.showPreview);
          break;
        case "help_panel":
          store.setShowHelp(!store.showHelp);
          break;
      }
    });
    unlisteners.push(unlMenu);

    set({ _unlisteners: unlisteners });
  },

  cleanupEventListeners: () => {
    get()._unlisteners.forEach((fn) => fn());
    set({ _unlisteners: [] });
  },
}));
