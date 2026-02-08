import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openPath as openFolder } from "@tauri-apps/plugin-opener";
import type {
  Session,
  Message,
  StreamChunk,
  SearchResult,
  AppConfig,
  SearchConfig,
  HealthStatus,
  PlanningTemplate,
  CodebaseImportSummary,
  GeneratedDocument,
  GenerateProgress,
  GenerateComplete,
  ModelPullProgress,
  OnboardingStep,
  ForgeTarget,
  QualityReport,
  CoverageReport,
  ConfidenceReport,
  GenerationMetadata,
} from "../types";
import { normalizeError } from "../utils/errorMessages";
import { resolveDefaultPath } from "../utils/paths";

interface ChatState {
  // Sessions
  sessions: Session[];
  templates: PlanningTemplate[];
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
  forgeTarget: ForgeTarget;
  planReadiness: QualityReport | null;
  planningCoverage: CoverageReport | null;
  generationConfidence: ConfidenceReport | null;
  generationMetadata: GenerationMetadata | null;
  latestImportSummary: CodebaseImportSummary | null;

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

  // Config
  config: AppConfig | null;

  // Loading
  sessionsLoading: boolean;
  messagesLoading: boolean;
  preferencesLoaded: boolean;

  // Actions
  checkHealth: () => Promise<HealthStatus | null>;
  dismissOnboarding: () => void;
  retryLastMessage: () => Promise<void>;
  loadSessions: () => Promise<void>;
  createSession: () => Promise<Session | null>;
  createSessionFromTemplate: (templateId: string) => Promise<Session | null>;
  createBranchFromMessage: (messageId?: string) => Promise<Session | null>;
  loadTemplates: () => Promise<void>;
  selectSession: (sessionId: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<void>;
  deleteSessions: (sessionIds: string[]) => Promise<void>;
  renameSession: (sessionId: string, name: string) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  cancelResponse: () => Promise<void>;
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
  updateConfig: (config: AppConfig) => Promise<AppConfig | null>;

  // Document actions
  setForgeTarget: (target: ForgeTarget) => void;
  analyzePlanReadiness: () => Promise<QualityReport | null>;
  getPlanningCoverage: () => Promise<CoverageReport | null>;
  getGenerationConfidence: () => Promise<ConfidenceReport | null>;
  generateDocuments: (options?: {
    target?: ForgeTarget;
    force?: boolean;
  }) => Promise<boolean>;
  loadDocuments: () => Promise<void>;
  checkStale: (sessionIdOverride?: string) => Promise<void>;
  setShowPreview: (show: boolean) => void;
  importCodebaseContext: (rootPath: string) => Promise<CodebaseImportSummary | null>;

  // Export actions
  saveToFolder: (folderPath: string) => Promise<string | null>;
  openFolder: (path: string) => Promise<void>;

  // Toast
  toast: { message: string; type: "success" | "error"; actionPath?: string } | null;
  showToast: (message: string, type: "success" | "error") => void;
  dismissToast: () => void;

  // Internal tracking
  _generatingSessionId: string | null;

  // Event listeners
  _unlisteners: UnlistenFn[];
  initEventListeners: () => Promise<void>;
  cleanupEventListeners: () => void;
}

export const useChatStore = create<ChatState>((set, get) => {
  let streamBuffer = "";
  let streamRafId: number | null = null;
  let cancelSafetyTimer: ReturnType<typeof setTimeout> | null = null;
  let cancelSafetySessionId: string | null = null;

  const flushStreamBuffer = () => {
    if (!streamBuffer) return;
    set((state) => ({
      streamingContent: state.streamingContent + streamBuffer,
    }));
    streamBuffer = "";
  };

  const scheduleStreamFlush = () => {
    if (streamRafId !== null) return;
    if (typeof requestAnimationFrame === "function") {
      streamRafId = requestAnimationFrame(() => {
        streamRafId = null;
        flushStreamBuffer();
      });
    } else {
      flushStreamBuffer();
    }
  };

  const clearCancelSafetyTimeout = (sessionId?: string) => {
    if (sessionId && cancelSafetySessionId !== sessionId) {
      return;
    }
    if (cancelSafetyTimer) {
      clearTimeout(cancelSafetyTimer);
    }
    cancelSafetyTimer = null;
    cancelSafetySessionId = null;
  };

  return ({
  sessions: [],
  templates: [],
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
  forgeTarget: "generic",
  planReadiness: null,
  planningCoverage: null,
  generationConfidence: null,
  generationMetadata: null,
  latestImportSummary: null,
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
  config: null,
  sessionsLoading: false,
  messagesLoading: false,
  preferencesLoaded: false,
  _generatingSessionId: null,
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
    const { messages, currentSessionId, isStreaming, isGenerating } = get();
    if (!currentSessionId || isStreaming || isGenerating) return;
    clearCancelSafetyTimeout(currentSessionId);

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
        request: {
          session_id: currentSessionId,
          content: lastUserMsg.content,
          retry: true,
        },
      });

      if (get().currentSessionId !== currentSessionId) return;

      const allMessages = await invoke<Message[]>("get_messages", {
        session_id: currentSessionId,
      });

      if (get().currentSessionId !== currentSessionId) return;

      set({
        messages: allMessages,
        isStreaming: false,
        streamingContent: "",
        searchQuery: null,
        searchResults: null,
      });
      void get().getPlanningCoverage();
    } catch (e) {
      if (get().currentSessionId === currentSessionId) {
        set({
          isStreaming: false,
          streamError: normalizeError(e),
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
      const session = await invoke<Session>("create_session", {
        request: { name: null },
      });
      set((state) => ({
        sessions: [session, ...state.sessions],
        currentSessionId: session.id,
        messages: [],
        streamingContent: "",
        streamError: null,
        documents: [],
        showPreview: false,
        documentsStale: false,
        planReadiness: null,
        planningCoverage: null,
        generationConfidence: null,
        generationMetadata: null,
        latestImportSummary: null,
      }));
      return session;
    } catch (e) {
      console.error("Failed to create session:", e);
      return null;
    }
  },

  createSessionFromTemplate: async (templateId: string) => {
    try {
      const template = get().templates.find((item) => item.id === templateId);
      const recommendedTarget = template?.recommended_target;
      const validTargets: ForgeTarget[] = [
        "generic",
        "codex",
        "claude",
        "cursor",
        "gemini",
      ];
      const resolvedTarget = validTargets.includes(recommendedTarget as ForgeTarget)
        ? (recommendedTarget as ForgeTarget)
        : get().forgeTarget;
      const session = await invoke<Session>("create_session_from_template", {
        request: { template_id: templateId, name: null },
      });

      set((state) => ({
        sessions: [session, ...state.sessions.filter((s) => s.id !== session.id)],
        currentSessionId: session.id,
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
        planReadiness: null,
        planningCoverage: null,
        generationConfidence: null,
        generationMetadata: null,
        latestImportSummary: null,
        forgeTarget: resolvedTarget,
      }));

      const messages = await invoke<Message[]>("get_messages", {
        session_id: session.id,
      });

      if (get().currentSessionId === session.id) {
        set({ messages, messagesLoading: false });
      }
      void get().getPlanningCoverage();
      return session;
    } catch (e) {
      console.error("Failed to create session from template:", e);
      set({ messagesLoading: false });
      return null;
    }
  },

  createBranchFromMessage: async (messageId?: string) => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const session = await invoke<Session>("create_branch_from_message", {
        request: {
          session_id: sessionId,
          from_message_id: messageId ?? null,
          name: null,
        },
      });
      set((state) => ({
        sessions: [session, ...state.sessions.filter((s) => s.id !== session.id)],
      }));
      await get().selectSession(session.id);
      set({
        toast: {
          message: `Created branch: ${session.name}`,
          type: "success",
        },
      });
      return session;
    } catch (e) {
      set({
        toast: {
          message: normalizeError(e),
          type: "error",
        },
      });
      return null;
    }
  },

  loadTemplates: async () => {
    try {
      const templates = await invoke<PlanningTemplate[]>("list_templates");
      set({ templates });
    } catch (e) {
      console.error("Failed to load templates:", e);
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
      planReadiness: null,
      planningCoverage: null,
      generationConfidence: null,
      generationMetadata: null,
      latestImportSummary: null,
    });
    try {
      const messages = await invoke<Message[]>("get_messages", {
        session_id: sessionId,
      });
      set({ messages, messagesLoading: false });

      // Load cached documents if any
      get().loadDocuments();
      void get().getPlanningCoverage();
    } catch (e) {
      console.error("Failed to load messages:", e);
      set({ messagesLoading: false });
    }
  },

  deleteSession: async (sessionId: string) => {
    try {
      await invoke("delete_session", { session_id: sessionId });
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
          planReadiness:
            state.currentSessionId === sessionId ? null : state.planReadiness,
          planningCoverage:
            state.currentSessionId === sessionId ? null : state.planningCoverage,
          generationConfidence:
            state.currentSessionId === sessionId ? null : state.generationConfidence,
          generationMetadata:
            state.currentSessionId === sessionId ? null : state.generationMetadata,
          latestImportSummary:
            state.currentSessionId === sessionId ? null : state.latestImportSummary,
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

  deleteSessions: async (sessionIds: string[]) => {
    if (sessionIds.length === 0) return;
    try {
      await invoke("delete_sessions", { session_ids: sessionIds });
      const deletedSet = new Set(sessionIds);
      set((state) => {
        const sessions = state.sessions.filter((s) => !deletedSet.has(s.id));
        const activeDeleted = state.currentSessionId
          ? deletedSet.has(state.currentSessionId)
          : false;
        const newCurrentId = activeDeleted
          ? sessions[0]?.id ?? null
          : state.currentSessionId;
        return {
          sessions,
          currentSessionId: newCurrentId,
          messages: activeDeleted ? [] : state.messages,
          documents: activeDeleted ? [] : state.documents,
          showPreview: activeDeleted ? false : state.showPreview,
          planReadiness: activeDeleted ? null : state.planReadiness,
          planningCoverage: activeDeleted ? null : state.planningCoverage,
          generationConfidence: activeDeleted ? null : state.generationConfidence,
          generationMetadata: activeDeleted ? null : state.generationMetadata,
          latestImportSummary: activeDeleted ? null : state.latestImportSummary,
        };
      });
      const newId = get().currentSessionId;
      if (newId) {
        get().selectSession(newId);
      }
    } catch (e) {
      console.error("Failed to delete sessions:", e);
    }
  },

  renameSession: async (sessionId: string, name: string) => {
    try {
      const updated = await invoke<Session>("update_session", {
        session_id: sessionId,
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
    if (!sessionId) return;
    if (get().isStreaming || get().isGenerating) {
      if (get().isGenerating) {
        set({
          toast: {
            message: "Wait for generation to complete before sending a new message.",
            type: "error",
          },
        });
      }
      return;
    }
    clearCancelSafetyTimeout(sessionId);

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
        request: { session_id: sessionId, content },
      });

      // Guard: if user switched sessions during the await, discard result
      if (get().currentSessionId !== sessionId) return;

      set((state) => {
        const exists = state.messages.some((m) => m.id === userMsg.id);
        return exists ? {} : { messages: [...state.messages, userMsg] };
      });

      const allMessages = await invoke<Message[]>("get_messages", {
        session_id: sessionId,
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
      void get().getPlanningCoverage();
    } catch (e) {
      // Only set error if we're still on the same session
      if (get().currentSessionId === sessionId) {
        set({
          isStreaming: false,
          streamError: normalizeError(e),
          searchQuery: null,
          searchResults: null,
        });
      }
    }
  },

  cancelResponse: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return;
    try {
      await invoke("cancel_response", { session_id: sessionId });
    } catch (e) {
      console.error("Failed to cancel response:", e);
    }
    // Safety net: if backend fails to emit stream:done/error within 2s, force-reset.
    clearCancelSafetyTimeout();
    cancelSafetySessionId = sessionId;
    cancelSafetyTimer = setTimeout(() => {
      if (
        cancelSafetySessionId === sessionId &&
        get().isStreaming &&
        get().currentSessionId === sessionId
      ) {
        set({ isStreaming: false, streamingContent: "" });
      }
      clearCancelSafetyTimeout(sessionId);
    }, 2000);
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
        preferencesLoaded: true,
      });
    } catch (e) {
      console.error("Failed to load preferences:", e);
      set({ preferencesLoaded: true });
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
      await invoke("pull_model", { model_name: name });
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
      const config = await invoke<AppConfig>("get_config");
      set({
        config,
        forgeTarget: config.output.default_target,
      });
      return config;
    } catch (e) {
      console.error("Failed to load config:", e);
      return null;
    }
  },

  updateSearchConfig: async (searchConfig: SearchConfig) => {
    try {
      await invoke("update_search_config", { search_config: searchConfig });
      set((state) =>
        state.config
          ? { config: { ...state.config, search: searchConfig } }
          : state,
      );
    } catch (e) {
      console.error("Failed to update search config:", e);
    }
  },

  updateConfig: async (config) => {
    try {
      const updated = await invoke<AppConfig>("update_config", { config });
      set({
        config: updated,
        forgeTarget: updated.output.default_target,
      });
      return updated;
    } catch (e) {
      console.error("Failed to update config:", e);
      return null;
    }
  },

  // ============ DOCUMENT GENERATION ============

  setForgeTarget: (target: ForgeTarget) => {
    set({ forgeTarget: target });
  },

  analyzePlanReadiness: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const report = await invoke<QualityReport>("analyze_plan_readiness", {
        session_id: sessionId,
      });
      set({ planReadiness: report });
      return report;
    } catch (e) {
      console.error("Failed to analyze readiness:", e);
      return null;
    }
  },

  getPlanningCoverage: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const report = await invoke<CoverageReport>("get_planning_coverage", {
        session_id: sessionId,
      });
      if (get().currentSessionId === sessionId) {
        set({ planningCoverage: report });
      }
      return report;
    } catch (e) {
      console.error("Failed to load planning coverage:", e);
      return null;
    }
  },

  getGenerationConfidence: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;
    try {
      const report = await invoke<ConfidenceReport | null>(
        "get_generation_confidence",
        { session_id: sessionId },
      );
      set({ generationConfidence: report });
      return report;
    } catch (e) {
      console.error("Failed to load generation confidence:", e);
      return null;
    }
  },

  generateDocuments: async (options) => {
    const sessionId = get().currentSessionId;
    if (!sessionId || get().isGenerating) return false;
    if (get().isStreaming) {
      set({
        toast: {
          message: "Wait for the current response to finish before forging documents.",
          type: "error",
        },
      });
      return false;
    }
    const target = options?.target ?? get().forgeTarget;
    const force = options?.force ?? false;

    set({ isGenerating: true, generateProgress: null, _generatingSessionId: sessionId });

    try {
      const documents = await invoke<GeneratedDocument[]>("generate_documents", {
        request: {
          session_id: sessionId,
          target,
          force,
        },
      });
      let generationMetadata: GenerationMetadata | null = null;
      try {
        generationMetadata = await invoke<GenerationMetadata | null>(
          "get_generation_metadata",
          { session_id: sessionId },
        );
      } catch (metaError) {
        console.warn("Failed to load generation metadata:", metaError);
      }

      let planReadiness: QualityReport | null = get().planReadiness;
      if (generationMetadata?.quality_json) {
        try {
          planReadiness = JSON.parse(generationMetadata.quality_json) as QualityReport;
        } catch {
          // keep previously known readiness
        }
      }
      let generationConfidence: ConfidenceReport | null = get().generationConfidence;
      if (generationMetadata?.confidence_json) {
        try {
          generationConfidence = JSON.parse(
            generationMetadata.confidence_json,
          ) as ConfidenceReport;
        } catch {
          // keep previously known confidence
        }
      } else {
        try {
          generationConfidence = await invoke<ConfidenceReport | null>(
            "get_generation_confidence",
            { session_id: sessionId },
          );
        } catch {
          // ignore confidence lookup failures
        }
      }

      // If user is still on the same session, show the documents
      if (get().currentSessionId === sessionId) {
        set({
          documents,
          isGenerating: false,
          generateProgress: null,
          documentsStale: false,
          showPreview: true,
          _generatingSessionId: null,
          generationMetadata,
          planReadiness,
          generationConfidence,
        });
      } else {
        // User switched away — docs are in DB, loadDocuments() will pick them up on navigate-back
        set({
          isGenerating: false,
          generateProgress: null,
          _generatingSessionId: null,
        });
      }
      return true;
    } catch (e) {
      console.error("Failed to generate documents:", e);
      set({
        isGenerating: false,
        generateProgress: null,
        _generatingSessionId: null,
        streamError: get().currentSessionId === sessionId
          ? `Document generation failed: ${normalizeError(e)}`
          : null,
      });
      return false;
    }
  },

  loadDocuments: async () => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return;

    try {
      const documents = await invoke<GeneratedDocument[]>("get_documents", {
        session_id: sessionId,
      });
      let generationMetadata: GenerationMetadata | null = null;
      try {
        generationMetadata = await invoke<GenerationMetadata | null>(
          "get_generation_metadata",
          { session_id: sessionId },
        );
      } catch (metaError) {
        console.warn("Failed to load generation metadata:", metaError);
      }
      let planReadiness: QualityReport | null = null;
      if (generationMetadata?.quality_json) {
        try {
          planReadiness = JSON.parse(generationMetadata.quality_json) as QualityReport;
        } catch {
          // Ignore malformed metadata and continue with documents
        }
      }
      let generationConfidence: ConfidenceReport | null = null;
      if (generationMetadata?.confidence_json) {
        try {
          generationConfidence = JSON.parse(
            generationMetadata.confidence_json,
          ) as ConfidenceReport;
        } catch {
          // Ignore malformed metadata and continue with documents
        }
      } else {
        try {
          generationConfidence = await invoke<ConfidenceReport | null>(
            "get_generation_confidence",
            { session_id: sessionId },
          );
        } catch {
          // ignore confidence lookup failures
        }
      }
      if (get().currentSessionId !== sessionId) {
        return;
      }
      set({
        documents,
        documentsStale: false,
        generationMetadata,
        planReadiness,
        generationConfidence,
      });
      // Check staleness for this same session in the background.
      void get().checkStale(sessionId);
    } catch (e) {
      console.error("Failed to load documents:", e);
    }
  },

  checkStale: async (sessionIdOverride?: string) => {
    const sessionId = sessionIdOverride ?? get().currentSessionId;
    if (!sessionId) return;

    try {
      const stale = await invoke<boolean>("check_documents_stale", {
        session_id: sessionId,
      });
      if (get().currentSessionId !== sessionId) {
        return;
      }
      set({ documentsStale: stale });
    } catch (e) {
      console.error("Failed to check staleness:", e);
    }
  },

  setShowPreview: (show: boolean) => set({ showPreview: show }),

  importCodebaseContext: async (rootPath: string) => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const summary = await invoke<CodebaseImportSummary>("import_codebase_context", {
        request: { session_id: sessionId, root_path: rootPath },
      });
      const messages = await invoke<Message[]>("get_messages", {
        session_id: sessionId,
      });
      if (get().currentSessionId === sessionId) {
        set({
          messages,
          latestImportSummary: summary,
          toast: {
            message: `Imported ${summary.files_included} files of codebase context`,
            type: "success",
          },
        });
      }
      void get().getPlanningCoverage();
      return summary;
    } catch (e) {
      set({
        toast: {
          message: normalizeError(e),
          type: "error",
        },
      });
      return null;
    }
  },

  // ============ EXPORT ============

  saveToFolder: async (folderPath: string) => {
    const sessionId = get().currentSessionId;
    if (!sessionId) return null;

    try {
      const savedPath = await invoke<string>("save_to_folder", {
        request: { session_id: sessionId, folder_path: folderPath },
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
          message: normalizeError(e),
          type: "error",
        },
      });
      return null;
    }
  },

  openFolder: async (path: string) => {
    try {
      await openFolder(path);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  },

  showToast: (message: string, type: "success" | "error") =>
    set({ toast: { message, type } }),
  dismissToast: () => set({ toast: null }),

  // ============ EVENT LISTENERS ============

  initEventListeners: async () => {
    const unlisteners: UnlistenFn[] = [];

    const unlChunk = await listen<StreamChunk>("stream:chunk", (event) => {
      const { type, content, session_id } = event.payload;
      const current = get().currentSessionId;
      if (session_id && current && session_id !== current) return;

      if (type === "content" && content) {
        streamBuffer += content;
        scheduleStreamFlush();
      }
    });
    unlisteners.push(unlChunk);

    const unlSearch = await listen<StreamChunk>("stream:search", (event) => {
      const { type, search_query, search_results, session_id } = event.payload;
      const current = get().currentSessionId;
      if (session_id && current && session_id !== current) return;

      if (type === "search_start" && search_query) {
        set({ searchQuery: search_query });
      } else if (type === "search_result" && search_results) {
        set({ searchResults: search_results });
      }
    });
    unlisteners.push(unlSearch);

    const unlError = await listen<StreamChunk>("stream:error", (event) => {
      const { session_id } = event.payload;
      const current = get().currentSessionId;
      if (session_id && current && session_id !== current) return;
      clearCancelSafetyTimeout(session_id);
      flushStreamBuffer();
      set({
        isStreaming: false,
        streamingContent: "",
        streamError: event.payload.error ?? "Unknown streaming error",
        searchQuery: null,
        searchResults: null,
      });
    });
    unlisteners.push(unlError);

    const unlDone = await listen<StreamChunk>("stream:done", (event) => {
      const { session_id } = event.payload;
      const current = get().currentSessionId;
      if (session_id && current && session_id !== current) return;
      clearCancelSafetyTimeout(session_id);
      flushStreamBuffer();
      set({
        isStreaming: false,
        streamingContent: "",
        searchQuery: null,
        searchResults: null,
      });
    });
    unlisteners.push(unlDone);

    const unlProgress = await listen<GenerateProgress>(
      "generate:progress",
      (event) => {
        const { session_id } = event.payload;
        const genSession = get()._generatingSessionId;
        // Only update UI if this event matches our generating session AND we're viewing it
        if (session_id !== genSession || session_id !== get().currentSessionId) return;
        set({ generateProgress: event.payload });
      },
    );
    unlisteners.push(unlProgress);

    const unlComplete = await listen<GenerateComplete>("generate:complete", (event) => {
      const { session_id } = event.payload;
      const genSession = get()._generatingSessionId;
      if (session_id !== genSession) return;
      set({ generateProgress: null, _generatingSessionId: null });
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
        case "save_to_folder": {
          const sessionId = store.currentSessionId;
          if (!sessionId) break;
          void (async () => {
            const defaultPath = await resolveDefaultPath(
              store.config?.output.default_save_path ?? "~/Projects",
            );
            const selected = await openDialog({
              directory: true,
              title: "Choose where to save your plan",
              defaultPath,
            });
            if (typeof selected === "string") {
              store.saveToFolder(selected);
            }
          })();
          break;
        }
        case "rename_session": {
          const sessionId = store.currentSessionId;
          const session = store.sessions.find((s) => s.id === sessionId);
          if (!sessionId || !session) break;
          const name = window.prompt("Rename session", session.name);
          if (name && name.trim().length > 0) {
            store.renameSession(sessionId, name.trim());
          }
          break;
        }
        case "delete_session": {
          const sessionId = store.currentSessionId;
          const session = store.sessions.find((s) => s.id === sessionId);
          if (!sessionId || !session) break;
          const confirmed = window.confirm(
            `Delete "${session.name}"? This cannot be undone.`,
          );
          if (confirmed) {
            store.deleteSession(sessionId);
          }
          break;
        }
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
    clearCancelSafetyTimeout();
    if (streamRafId !== null && typeof cancelAnimationFrame === "function") {
      cancelAnimationFrame(streamRafId);
    }
    streamRafId = null;
    streamBuffer = "";
    get()._unlisteners.forEach((fn) => fn());
    set({ _unlisteners: [] });
  },
  });
});
