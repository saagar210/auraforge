import { useEffect, useRef, useState, useCallback } from "react";
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
import { HelpPanel } from "./components/HelpPanel";
import { InfoTooltip } from "./components/InfoTooltip";
import { Toast } from "./components/Toast";
import { EmberParticles } from "./components/EmberParticles";
import { ThermalBackground } from "./components/ThermalBackground";
import { useChatStore } from "./stores/chatStore";
import { friendlyError } from "./utils/errorMessages";
import { open } from "@tauri-apps/plugin-dialog";
import { AlertCircle, X, FileText, MessageSquare } from "lucide-react";

function App() {
  const {
    currentSessionId,
    messages,
    isStreaming,
    streamingContent,
    streamError,
    searchQuery,
    messagesLoading,
    documents,
    isGenerating,
    generateProgress,
    documentsStale,
    showPreview,
    healthStatus,
    wizardCompleted,
    isFirstSession,
    showSettings,
    showHelp,
    checkHealth,
    loadPreferences,
    setShowSettings,
    setShowHelp,
    loadSessions,
    createSession,
    sendMessage,
    clearStreamError,
    retryLastMessage,
    generateDocuments,
    setShowPreview,
    saveToFolder,
    openFolder,
    markFirstSessionComplete,
    toast,
    dismissToast,
    initEventListeners,
    cleanupEventListeners,
  } = useChatStore();

  const [inputValue, setInputValue] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const chatContainerRef = useRef<HTMLDivElement>(null);

  // Initialize on mount
  useEffect(() => {
    loadPreferences();
    checkHealth();
    loadSessions();
    initEventListeners();
    return () => cleanupEventListeners();
  }, []);

  // Show onboarding wizard if not completed
  const showOnboarding =
    healthStatus !== null &&
    !wizardCompleted &&
    (!healthStatus.ollama_connected || !healthStatus.ollama_model_available);

  // Derived state
  const userMessageCount = messages.filter((m) => m.role === "user").length;
  const assistantMessageCount = messages.filter(
    (m) => m.role === "assistant",
  ).length;
  const canForge =
    userMessageCount >= 3 &&
    assistantMessageCount >= 3 &&
    !isStreaming &&
    !isGenerating;
  const hasDocuments = documents.length > 0;

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

  const handleSuggestionClick = (text: string) => {
    setInputValue(`I want to build ${text.toLowerCase()}`);
  };

  const handleSaveToFolder = useCallback(async () => {
    const selected = await open({
      directory: true,
      title: "Choose where to save your plan",
      defaultPath: "~/Projects",
    });
    if (selected) {
      await saveToFolder(selected);
    }
  }, [saveToFolder]);

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
        if (canForge) generateDocuments();
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
    hasDocuments,
    showSettings,
    showPreview,
    showOnboarding,
    showHelp,
  ]);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingContent, isStreaming, isGenerating]);

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
                onRegenerate={generateDocuments}
                regenerating={isGenerating}
                onSave={handleSaveToFolder}
              />
            ) : (
              <>
                {/* Chat Messages Area */}
                <div
                  ref={chatContainerRef}
                  className="flex-1 overflow-y-auto px-6 py-6"
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
                      />
                    ) : (
                      <>
                        {messages.map((msg) => (
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
                        {isGenerating && generateProgress && (
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
                              onClick={generateDocuments}
                              disabled={!canForge}
                              generating={isGenerating}
                            />
                            <InfoTooltip text="Generates 5 planning documents from your conversation" />
                          </div>
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
                  disabled={isStreaming || isGenerating}
                  value={inputValue}
                  onChange={setInputValue}
                />
              </>
            )}
          </>
        ) : (
          <EmptyState hasSession={false} onNewProject={handleNewProject} />
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

      {/* Onboarding Wizard */}
      {showOnboarding && healthStatus && <OnboardingWizard />}
    </div>
  );
}

export default App;
