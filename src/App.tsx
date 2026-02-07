import {
  useEffect,
  useRef,
  useState,
  useCallback,
  useMemo,
  useLayoutEffect,
} from "react";
import { Sidebar } from "./components/Sidebar";
import { ChatMessage, StreamingMessage } from "./components/ChatMessage";
import { ChatInput } from "./components/ChatInput";
import { ThinkingIndicator } from "./components/ThinkingIndicator";
import { SearchIndicator } from "./components/SearchIndicator";
import { ForgeButton } from "./components/ForgeButton";
import { ForgingProgress } from "./components/ForgingProgress";
import { DocumentPreview } from "./components/DocumentPreview";
import { EmptyState } from "./components/EmptyState";
import { OnboardingWizard } from "./components/OnboardingWizard";
import { SettingsPanel } from "./components/SettingsPanel";
import { HelpPanel } from "./components/HelpPanel";
import { InfoTooltip } from "./components/InfoTooltip";
import { Toast } from "./components/Toast";
import { EmberParticles } from "./components/EmberParticles";
import { ThermalBackground } from "./components/ThermalBackground";
import { useChatStore } from "./stores/chatStore";
import type { HealthStatus } from "./types";
import { friendlyError } from "./utils/errorMessages";
import { resolveDefaultPath } from "./utils/paths";
import { open } from "@tauri-apps/plugin-dialog";
import { AlertCircle, X, FileText, MessageSquare } from "lucide-react";

function App() {
  const INITIAL_MESSAGE_WINDOW = 120;
  const MESSAGE_WINDOW_STEP = 80;
  const SCROLL_TOP_THRESHOLD = 48;
  const SCROLL_BOTTOM_THRESHOLD = 80;

  const {
    currentSessionId,
    messages,
    templates,
    isStreaming,
    streamingContent,
    streamError,
    searchQuery,
    messagesLoading,
    documents,
    isGenerating,
    _generatingSessionId,
    generateProgress,
    documentsStale,
    showPreview,
    forgeTarget,
    planReadiness,
    healthStatus,
    wizardCompleted,
    preferencesLoaded,
    isFirstSession,
    showSettings,
    showHelp,
    config,
    checkHealth,
    loadPreferences,
    loadConfig,
    setShowSettings,
    setShowHelp,
    loadSessions,
    loadTemplates,
    createSession,
    createSessionFromTemplate,
    sendMessage,
    cancelResponse,
    clearStreamError,
    retryLastMessage,
    analyzePlanReadiness,
    generateDocuments,
    setForgeTarget,
    setShowPreview,
    saveToFolder,
    openFolder,
    markFirstSessionComplete,
    toast,
    showToast,
    dismissToast,
    initEventListeners,
    cleanupEventListeners,
  } = useChatStore();

  const [inputValue, setInputValue] = useState("");
  const [visibleCount, setVisibleCount] = useState(INITIAL_MESSAGE_WINDOW);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const chatContainerRef = useRef<HTMLDivElement>(null);
  const loadingOlderRef = useRef(false);
  const prevScrollHeightRef = useRef<number | null>(null);
  const isAtBottomRef = useRef(true);

  const prevHealthRef = useRef<HealthStatus | null>(null);

  // Initialize on mount
  useEffect(() => {
    loadPreferences();
    checkHealth();
    loadSessions();
    loadTemplates();
    loadConfig();
    initEventListeners();
    return () => cleanupEventListeners();
  }, []);

  // Periodic health re-check every 60s
  useEffect(() => {
    const interval = setInterval(() => {
      checkHealth();
    }, 60_000);
    return () => clearInterval(interval);
  }, []);

  // Detect health degradation and notify user
  useEffect(() => {
    if (!healthStatus || !prevHealthRef.current) {
      prevHealthRef.current = healthStatus;
      return;
    }
    const wasOk =
      prevHealthRef.current.ollama_connected &&
      prevHealthRef.current.ollama_model_available;
    const isOk =
      healthStatus.ollama_connected && healthStatus.ollama_model_available;
    if (wasOk && !isOk) {
      showToast(
        "Ollama connection lost. Check that Ollama is running.",
        "error",
      );
    }
    prevHealthRef.current = healthStatus;
  }, [healthStatus]);

  // Show onboarding wizard if health fails OR wizard hasn't been completed
  const showOnboarding =
    healthStatus !== null &&
    preferencesLoaded &&
    (!healthStatus.ollama_connected ||
      !healthStatus.ollama_model_available ||
      !healthStatus.database_ok ||
      !healthStatus.config_valid ||
      !wizardCompleted);

  // Derived state
  const { userMessageCount, assistantMessageCount } = useMemo(() => {
    let users = 0;
    let assistants = 0;
    for (const msg of messages) {
      if (msg.role === "user") users += 1;
      if (msg.role === "assistant") assistants += 1;
    }
    return { userMessageCount: users, assistantMessageCount: assistants };
  }, [messages]);
  const isCurrentSessionGenerating = useMemo(
    () => isGenerating && _generatingSessionId === currentSessionId,
    [isGenerating, _generatingSessionId, currentSessionId],
  );
  const canForge = useMemo(
    () =>
      userMessageCount >= 3 &&
      assistantMessageCount >= 3 &&
      !isStreaming &&
      !isCurrentSessionGenerating,
    [assistantMessageCount, isCurrentSessionGenerating, isStreaming, userMessageCount],
  );
  const hasDocuments = documents.length > 0;
  const hasOlderMessages = messages.length > visibleCount;
  const visibleMessages = useMemo(() => {
    if (messages.length <= visibleCount) return messages;
    return messages.slice(messages.length - visibleCount);
  }, [messages, visibleCount]);

  const handleSend = (content: string) => {
    // Mark first session complete on first message
    if (isFirstSession) {
      markFirstSessionComplete();
    }
    sendMessage(content);
    setInputValue("");
  };

  const handleNewProject = async () => {
    await createSession();
  };

  const handleTemplateStart = async (templateId: string) => {
    await createSessionFromTemplate(templateId);
  };

  const handleSuggestionClick = (text: string) => {
    setInputValue(`I want to build ${text.toLowerCase()}`);
  };

  const handleSaveToFolder = useCallback(async () => {
    const defaultPath = await resolveDefaultPath(
      config?.output.default_save_path ?? "~/Projects",
    );
    const selected = await open({
      directory: true,
      title: "Choose where to save your plan",
      defaultPath,
    });
    if (selected) {
      await saveToFolder(selected);
    }
  }, [config, saveToFolder]);

  const handleForgePlan = useCallback(async (options?: { ignoreThreshold?: boolean }) => {
    if (
      !currentSessionId ||
      isCurrentSessionGenerating ||
      isStreaming ||
      (!options?.ignoreThreshold && !canForge)
    ) {
      return;
    }
    const readiness = await analyzePlanReadiness();
    if (readiness && readiness.missing_must_haves.length > 0) {
      const proceed = window.confirm(
        `Readiness check found missing must-haves:\n\n- ${readiness.missing_must_haves.join(
          "\n- ",
        )}\n\nForge anyway with [TBD] sections?`,
      );
      if (!proceed) {
        return;
      }
      await generateDocuments({ target: forgeTarget, force: true });
      return;
    }
    await generateDocuments({ target: forgeTarget, force: false });
  }, [
    analyzePlanReadiness,
    canForge,
    currentSessionId,
    forgeTarget,
    generateDocuments,
    isCurrentSessionGenerating,
    isStreaming,
  ]);

  const loadOlderMessages = useCallback(() => {
    if (!hasOlderMessages) return;
    const el = chatContainerRef.current;
    if (!el) return;
    loadingOlderRef.current = true;
    prevScrollHeightRef.current = el.scrollHeight;
    setVisibleCount((count) =>
      Math.min(messages.length, count + MESSAGE_WINDOW_STEP),
    );
  }, [hasOlderMessages, messages.length]);

  const handleChatScroll = useCallback(() => {
    const el = chatContainerRef.current;
    if (!el) return;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    isAtBottomRef.current = distanceFromBottom < SCROLL_BOTTOM_THRESHOLD;

    if (
      el.scrollTop < SCROLL_TOP_THRESHOLD &&
      hasOlderMessages &&
      !loadingOlderRef.current
    ) {
      loadOlderMessages();
    }
  }, [hasOlderMessages, loadOlderMessages]);

  // Friendly error display
  const errorDisplay = streamError ? friendlyError(streamError) : null;

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMod = e.metaKey || e.ctrlKey;

      // Escape — Close panels
      if (e.key === "Escape") {
        if (showSettings) {
          setShowSettings(false);
        } else if (showHelp) {
          setShowHelp(false);
        } else if (showOnboarding) {
          // Can't dismiss onboarding via Escape
        } else if (showPreview) {
          setShowPreview(false);
        }
        return;
      }

      // Cmd+, — Toggle settings
      if (isMod && e.key === ",") {
        e.preventDefault();
        setShowSettings(!showSettings);
        return;
      }

      // Block other shortcuts when modals are open
      if (showSettings || showOnboarding) return;

      // Cmd+N — New project
      if (isMod && e.key === "n") {
        e.preventDefault();
        createSession();
        return;
      }

      // Cmd+G — Generate documents (forge)
      if (isMod && e.key === "g") {
        e.preventDefault();
        if (canForge) {
          void handleForgePlan();
        }
        return;
      }

      // Cmd+S — Save to folder
      if (isMod && e.key === "s") {
        if (hasDocuments) {
          e.preventDefault();
          handleSaveToFolder();
        }
        return;
      }

      // Cmd+? — Toggle help
      if (isMod && e.key === "/") {
        e.preventDefault();
        setShowHelp(!showHelp);
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    canForge,
    handleForgePlan,
    hasDocuments,
    showSettings,
    showPreview,
    showOnboarding,
    showHelp,
  ]);

  // Auto-scroll to bottom
  useEffect(() => {
    if (loadingOlderRef.current || !isAtBottomRef.current) return;
    const behavior = isStreaming ? "auto" : "smooth";
    messagesEndRef.current?.scrollIntoView({ behavior });
  }, [visibleMessages, streamingContent, isStreaming, isGenerating]);

  useEffect(() => {
    setVisibleCount(INITIAL_MESSAGE_WINDOW);
  }, [currentSessionId]);

  useLayoutEffect(() => {
    if (!loadingOlderRef.current) return;
    const el = chatContainerRef.current;
    if (!el) return;
    const previousHeight = prevScrollHeightRef.current;
    if (previousHeight != null) {
      const newHeight = el.scrollHeight;
      el.scrollTop = newHeight - previousHeight;
    }
    loadingOlderRef.current = false;
    prevScrollHeightRef.current = null;
  }, [visibleCount, messages.length]);

  return (
    <div className="h-full flex bg-void relative">
      {/* Atmospheric layers */}
      <ThermalBackground />
      <EmberParticles />

      {/* Sidebar */}
      <Sidebar />

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-w-0 relative z-10">
        {currentSessionId ? (
          <>
            {/* View Toggle — shown when documents exist */}
            {hasDocuments && (
              <div
                className="flex items-center gap-1 px-4 py-2 border-b border-border-subtle bg-elevated"
                role="tablist"
                aria-label="View toggle"
              >
                <button
                  onClick={() => setShowPreview(false)}
                  role="tab"
                  aria-selected={!showPreview}
                  aria-controls="chat-panel"
                  className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-colors cursor-pointer border-none ${
                    !showPreview
                      ? "bg-surface text-text-primary"
                      : "bg-transparent text-text-secondary hover:text-text-primary"
                  }`}
                >
                  <MessageSquare
                    className="w-3.5 h-3.5"
                    aria-hidden="true"
                  />
                  Chat
                </button>
                <button
                  onClick={() => setShowPreview(true)}
                  role="tab"
                  aria-selected={showPreview}
                  aria-controls="documents-panel"
                  className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md transition-colors cursor-pointer border-none ${
                    showPreview
                      ? "bg-surface text-text-primary"
                      : "bg-transparent text-text-secondary hover:text-text-primary"
                  }`}
                >
                  <FileText className="w-3.5 h-3.5" aria-hidden="true" />
                  Documents
                  {documentsStale && (
                    <span
                      className="w-2 h-2 rounded-full bg-status-warning"
                      aria-label="Documents are outdated"
                    />
                  )}
                </button>
              </div>
            )}

            {/* Document Preview View */}
            {showPreview && hasDocuments ? (
              <DocumentPreview
                documents={documents}
                stale={documentsStale}
                onRegenerate={() => handleForgePlan({ ignoreThreshold: true })}
                regenerating={isCurrentSessionGenerating}
                onSave={handleSaveToFolder}
              />
            ) : (
              <>
                {/* Chat Messages Area */}
                <div
                  ref={chatContainerRef}
                  className="flex-1 overflow-y-auto px-6 py-6"
                  onScroll={handleChatScroll}
                >
                  <div className="flex flex-col gap-4 max-w-[720px] mx-auto">
                    {messagesLoading ? (
                      <div className="flex items-center justify-center py-10">
                        <div className="w-5 h-5 border-2 border-accent-glow/30 border-t-accent-glow rounded-full animate-spin" />
                      </div>
                    ) : messages.length === 0 && !isStreaming ? (
                      <EmptyState
                        hasSession={true}
                        onNewProject={handleNewProject}
                        isFirstSession={isFirstSession}
                        onSuggestionClick={handleSuggestionClick}
                        templates={templates}
                        onTemplateSelect={handleTemplateStart}
                      />
                    ) : (
                      <>
                        {hasOlderMessages && (
                          <button
                            onClick={loadOlderMessages}
                            className="mx-auto text-xs text-text-muted hover:text-text-primary border border-border-subtle rounded-full px-3 py-1 transition-colors"
                          >
                            Load earlier messages
                          </button>
                        )}
                        {visibleMessages.map((msg) => (
                          <ChatMessage key={msg.id} message={msg} />
                        ))}

                        {/* Streaming response */}
                        {isStreaming && streamingContent && (
                          <StreamingMessage content={streamingContent} />
                        )}

                        {/* Search indicator */}
                        {isStreaming && searchQuery && !streamingContent && (
                          <SearchIndicator query={searchQuery} />
                        )}

                        {/* Thinking indicator */}
                        {isStreaming && !searchQuery && !streamingContent && (
                          <ThinkingIndicator />
                        )}

                        {/* Forging progress */}
                        {isCurrentSessionGenerating && generateProgress && (
                          <ForgingProgress
                            current={generateProgress.current}
                            total={generateProgress.total}
                            filename={generateProgress.filename}
                          />
                        )}

                        {/* Forge the Plan button */}
                        {canForge && (
                          <div className="flex items-center gap-2">
                            <ForgeButton
                              onClick={handleForgePlan}
                              disabled={!canForge}
                              generating={isCurrentSessionGenerating}
                              target={forgeTarget}
                              onTargetChange={setForgeTarget}
                            />
                            <InfoTooltip text="Generates planning docs + model handoff from your conversation" />
                          </div>
                        )}
                        {canForge && planReadiness && (
                          <p className="text-xs text-text-muted text-center">
                            Readiness: {planReadiness.score}/100
                          </p>
                        )}
                      </>
                    )}

                    <div ref={messagesEndRef} />
                  </div>
                </div>

                {/* Stream Error */}
                {errorDisplay && (
                  <div className="px-6 mb-2" role="alert">
                    <div className="max-w-[720px] mx-auto">
                      <div className="flex items-start gap-3 p-3 bg-status-error/10 border border-status-error/30 rounded-lg">
                        <AlertCircle
                          className="w-4 h-4 text-status-error shrink-0 mt-0.5"
                          aria-hidden="true"
                        />
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-text-primary">
                            {errorDisplay.message}
                          </p>
                          <p className="text-xs text-text-secondary mt-0.5">
                            {errorDisplay.suggestion}
                          </p>
                        </div>
                        <button
                          onClick={retryLastMessage}
                          aria-label="Retry last message"
                          className="text-xs text-accent-gold hover:text-accent-gold/80 cursor-pointer bg-transparent border border-accent-gold/40 rounded px-2 py-1 whitespace-nowrap transition-colors"
                        >
                          Retry
                        </button>
                        <button
                          onClick={clearStreamError}
                          aria-label="Dismiss error"
                          className="text-text-muted hover:text-text-primary cursor-pointer bg-transparent border-none"
                        >
                          <X className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                {/* Chat Input */}
                <ChatInput
                  onSend={handleSend}
                  disabled={isStreaming || isCurrentSessionGenerating}
                  onCancel={cancelResponse}
                  isStreaming={isStreaming}
                  value={inputValue}
                  onChange={setInputValue}
                />
              </>
            )}
          </>
        ) : (
          <EmptyState
            hasSession={false}
            onNewProject={handleNewProject}
            templates={templates}
            onTemplateSelect={handleTemplateStart}
          />
        )}
      </main>

      {/* Toast Notifications */}
      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          action={
            toast.actionPath
              ? {
                  label: "Open Folder",
                  onClick: () => openFolder(toast.actionPath!),
                }
              : undefined
          }
          onDismiss={dismissToast}
        />
      )}

      {/* Help Panel */}
      <HelpPanel open={showHelp} onClose={() => setShowHelp(false)} />

      {/* Settings Panel */}
      <SettingsPanel
        open={showSettings}
        onClose={() => setShowSettings(false)}
      />

      {/* Onboarding Wizard */}
      {showOnboarding && healthStatus && <OnboardingWizard />}
    </div>
  );
}

export default App;
