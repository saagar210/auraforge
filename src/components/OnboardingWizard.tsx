import { useEffect, useRef, useCallback, useState } from "react";
import {
  Flame,
  CheckCircle,
  Download,
  Loader2,
  ExternalLink,
} from "lucide-react";
import { openUrl as shellOpen } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../stores/chatStore";
import type { OnboardingStep, DiskSpace } from "../types";

const STEPS: OnboardingStep[] = [
  "welcome",
  "install-ollama",
  "download-model",
  "search",
  "ready",
];

function StepDots({ current }: { current: OnboardingStep }) {
  const currentIdx = STEPS.indexOf(current);
  return (
    <div className="flex items-center gap-2 justify-center mb-6">
      {STEPS.map((step, i) => (
        <div
          key={step}
          className={`w-2 h-2 rounded-full transition-all duration-300 ${
            i < currentIdx
              ? "bg-accent-gold"
              : i === currentIdx
                ? "bg-accent-glow ring-2 ring-accent-glow/30"
                : "bg-surface"
          }`}
        />
      ))}
    </div>
  );
}

export function OnboardingWizard() {
  const {
    wizardStep,
    setWizardStep,
    completeWizard,
    healthStatus,
    checkHealth,
    modelPullProgress,
    isModelPulling,
    pullModel,
    cancelPullModel,
    createSession,
    setShowSettings,
    updateSearchConfig,
  } = useChatStore();

  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [diskSpace, setDiskSpace] = useState<DiskSpace | null>(null);
  const [configModel, setConfigModel] = useState<string>("");
  const [searchEnabled, setSearchEnabled] = useState(true);
  const [searchProvider, setSearchProvider] = useState<"tavily" | "duckduckgo">("duckduckgo");
  const [tavilyKey, setTavilyKey] = useState("");

  // Load config model name on mount
  useEffect(() => {
    invoke<{ llm: { model: string }; search: { enabled: boolean; provider: string; tavily_api_key: string } }>(
      "get_config",
    ).then((config) => {
      setConfigModel(config.llm.model);
      setSearchEnabled(config.search.enabled);
      setSearchProvider(config.search.provider === "tavily" ? "tavily" : "duckduckgo");
      setTavilyKey(config.search.tavily_api_key);
    }).catch((e) => {
      console.error("Failed to load config in onboarding:", e);
    });
  }, []);

  // Auto-poll health every 3s while onboarding is open
  useEffect(() => {
    intervalRef.current = setInterval(() => checkHealth(), 3000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [wizardStep, checkHealth]);

  // Auto-advance from install-ollama when Ollama connects
  useEffect(() => {
    if (
      wizardStep === "install-ollama" &&
      healthStatus?.ollama_connected
    ) {
      const timer = setTimeout(() => setWizardStep("download-model"), 1000);
      return () => clearTimeout(timer);
    }
  }, [wizardStep, healthStatus?.ollama_connected, setWizardStep]);

  // Auto-advance from download-model when model available
  useEffect(() => {
    if (
      wizardStep === "download-model" &&
      healthStatus?.ollama_model_available
    ) {
      const timer = setTimeout(() => setWizardStep("search"), 1000);
      return () => clearTimeout(timer);
    }
  }, [wizardStep, healthStatus?.ollama_model_available, setWizardStep]);

  // Check disk space when entering download-model step
  useEffect(() => {
    if (wizardStep === "download-model") {
      invoke<DiskSpace>("check_disk_space").then(setDiskSpace).catch(() => {});
      // Also check if model already exists
      checkHealth();
    }
  }, [wizardStep, checkHealth]);

  const handleDownloadOllama = useCallback(async () => {
    await shellOpen("https://ollama.com/download");
  }, []);

  const handlePullModel = useCallback(() => {
    pullModel(configModel);
  }, [pullModel, configModel]);

  const handleFinish = useCallback(async () => {
    await completeWizard();
    await createSession();
  }, [completeWizard, createSession]);

  const handleSaveSearch = useCallback(async () => {
    await updateSearchConfig({
      enabled: searchEnabled,
      provider: searchEnabled ? searchProvider : "none",
      tavily_api_key: searchProvider === "tavily" ? tavilyKey : "",
      searxng_url: "",
      proactive: true,
    });
    setWizardStep("ready");
  }, [searchEnabled, searchProvider, tavilyKey, updateSearchConfig, setWizardStep]);

  const progressPercent =
    modelPullProgress?.total && modelPullProgress?.completed
      ? Math.round(
          (modelPullProgress.completed / modelPullProgress.total) * 100,
        )
      : 0;

  const formatBytes = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(0)} MB`;
  };

  return (
    <div
      className="fixed inset-0 z-60 flex items-center justify-center animate-[fade-in_0.2s_ease]"
      style={{
        background: "rgba(0, 0, 0, 0.85)",
        backdropFilter: "blur(12px)",
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Setup wizard"
    >
      <div className="bg-elevated border border-border-default rounded-2xl w-full max-w-md mx-4 shadow-lg overflow-hidden animate-[modal-in_0.3s_ease]">
        <div className="px-6 pt-6">
          <StepDots current={wizardStep} />
        </div>

        <div className="px-6 pb-6">
          {/* Step 1: Welcome */}
          {wizardStep === "welcome" && (
            <div className="text-center">
              <Flame
                className="w-16 h-16 text-accent-glow mx-auto mb-4"
                style={{
                  filter: "drop-shadow(0 0 20px rgba(232,160,69,0.5))",
                }}
              />
              <h2 className="text-2xl font-heading font-semibold text-text-primary mb-2">
                Welcome to AuraForge
              </h2>
              <p className="text-sm text-text-secondary mb-1">
                AuraForge helps you plan software projects through conversation.
                Describe what you want to build, and it creates everything you
                need to start.
              </p>
              <p className="text-xs text-text-muted mb-6">
                Let's make sure you're set up.
              </p>
              <button
                onClick={() => setWizardStep("install-ollama")}
                className="px-6 py-2.5 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none"
              >
                Get Started
              </button>
            </div>
          )}

          {/* Step 2: Install Ollama */}
          {wizardStep === "install-ollama" && (
            <div>
              <h2 className="text-xl font-heading font-semibold text-text-primary mb-3 text-center">
                Install Ollama
              </h2>

              {healthStatus?.ollama_connected ? (
                <div className="flex flex-col items-center gap-3 py-4">
                  <CheckCircle className="w-10 h-10 text-status-success" />
                  <p className="text-sm text-status-success font-medium">
                    Ollama is running
                  </p>
                </div>
              ) : (
                <>
                  <p className="text-sm text-text-secondary mb-4">
                    AuraForge needs Ollama to run AI locally on your machine.
                    Everything stays private.
                  </p>

                  <div className="bg-surface rounded-lg px-4 py-3 mb-4 space-y-2">
                    <div className="flex items-start gap-2.5">
                      <span className="text-xs font-mono text-accent-gold mt-0.5">
                        1
                      </span>
                      <span className="text-sm text-text-primary">
                        Download Ollama
                      </span>
                    </div>
                    <div className="flex items-start gap-2.5">
                      <span className="text-xs font-mono text-accent-gold mt-0.5">
                        2
                      </span>
                      <span className="text-sm text-text-primary">
                        Install and open it
                      </span>
                    </div>
                    <div className="flex items-start gap-2.5">
                      <span className="text-xs font-mono text-accent-gold mt-0.5">
                        3
                      </span>
                      <span className="text-sm text-text-primary">
                        Come back here
                      </span>
                    </div>
                  </div>

                  <button
                    onClick={handleDownloadOllama}
                    className="w-full px-4 py-2.5 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none flex items-center justify-center gap-2 mb-3"
                  >
                    <Download className="w-4 h-4" />
                    Download Ollama
                    <ExternalLink className="w-3 h-3" />
                  </button>

                  <div className="flex items-center justify-center gap-2 mb-3">
                    <Loader2 className="w-3.5 h-3.5 text-accent-glow animate-spin" />
                    <span className="text-xs text-text-muted">
                      Waiting for Ollama...
                    </span>
                  </div>

                  <button
                    onClick={() => checkHealth()}
                    className="text-xs text-text-secondary hover:text-text-primary transition-colors cursor-pointer bg-transparent border-none underline mx-auto block"
                  >
                    I already have Ollama
                  </button>
                </>
              )}
            </div>
          )}

          {/* Step 3: Download Model */}
          {wizardStep === "download-model" && (
            <div>
              <h2 className="text-xl font-heading font-semibold text-text-primary mb-3 text-center">
                Download AI Model
              </h2>

              {healthStatus?.ollama_model_available ? (
                <div className="flex flex-col items-center gap-3 py-4">
                  <CheckCircle className="w-10 h-10 text-status-success" />
                  <p className="text-sm text-status-success font-medium">
                    Model ready
                  </p>
                </div>
              ) : isModelPulling ? (
                <div>
                  <p className="text-sm text-text-secondary mb-4 text-center">
                    Downloading{" "}
                    <span className="text-text-primary font-medium">
                      {configModel}
                    </span>
                  </p>

                  {/* Progress bar */}
                  <div className="bg-surface rounded-full h-3 mb-2 overflow-hidden">
                    <div
                      className="h-full bg-accent-gold rounded-full transition-all duration-300"
                      style={{ width: `${progressPercent}%` }}
                    />
                  </div>

                  <div className="flex items-center justify-between text-xs text-text-muted mb-1">
                    <span>
                      {modelPullProgress?.status === "downloading" &&
                      modelPullProgress?.completed &&
                      modelPullProgress?.total
                        ? `${formatBytes(modelPullProgress.completed)} of ${formatBytes(modelPullProgress.total)}`
                        : modelPullProgress?.status || "Starting..."}
                    </span>
                    {progressPercent > 0 && <span>{progressPercent}%</span>}
                  </div>

                  <button
                    onClick={cancelPullModel}
                    className="mt-3 text-xs text-text-secondary hover:text-status-error transition-colors cursor-pointer bg-transparent border-none underline mx-auto block"
                  >
                    Cancel
                  </button>
                </div>
              ) : (
                <>
                  <p className="text-sm text-text-secondary mb-4">
                    AuraForge needs to download its AI model. This is a one-time
                    download.
                  </p>

                  {diskSpace && (
                    <div
                      className={`bg-surface rounded-lg px-4 py-3 mb-4 text-xs ${
                        diskSpace.sufficient
                          ? "text-text-muted"
                          : "text-status-warning"
                      }`}
                    >
                      Requires ~18 GB. You have{" "}
                      {diskSpace.available_gb.toFixed(0)} GB available.
                      {!diskSpace.sufficient && (
                        <p className="mt-1 text-status-error">
                          Not enough disk space. Free up some space and try
                          again.
                        </p>
                      )}
                    </div>
                  )}

                  {modelPullProgress?.status.startsWith("error") && (
                    <div className="bg-status-error/10 border border-status-error/30 rounded-lg px-4 py-3 mb-4 text-xs text-status-error">
                      Download failed. Click below to try again &mdash; Ollama
                      will resume where it left off.
                    </div>
                  )}

                  <button
                    onClick={handlePullModel}
                    disabled={diskSpace !== null && !diskSpace.sufficient}
                    className="w-full px-4 py-2.5 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                  >
                    <Download className="w-4 h-4" />
                    Download Model
                  </button>
                </>
              )}
            </div>
          )}

          {/* Step 4: Web Search */}
          {wizardStep === "search" && (
            <div>
              <h2 className="text-xl font-heading font-semibold text-text-primary mb-3 text-center">
                Web Search (Optional)
              </h2>
              <p className="text-sm text-text-secondary mb-4">
                Enable web search to ground answers in current best practices. You can use free
                DuckDuckGo or add a Tavily API key for higher-quality results.
              </p>

              <div className="flex items-center justify-between mb-4">
                <span className="text-sm text-text-primary">Enable web search</span>
                <button
                  onClick={() => setSearchEnabled((prev) => !prev)}
                  className={`relative w-10 h-5 rounded-full transition-colors duration-200 cursor-pointer border-none ${
                    searchEnabled ? "bg-accent-gold" : "bg-surface"
                  }`}
                  role="switch"
                  aria-checked={searchEnabled}
                >
                  <span
                    className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform duration-200 ${
                      searchEnabled ? "translate-x-5" : "translate-x-0"
                    }`}
                  />
                </button>
              </div>

              {searchEnabled && (
                <div className="space-y-3">
                  <label className="block text-sm text-text-secondary">
                    Provider
                    <select
                      value={searchProvider}
                      onChange={(e) =>
                        setSearchProvider(e.target.value as "tavily" | "duckduckgo")
                      }
                      className="w-full mt-1.5 px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                    >
                      <option value="duckduckgo">DuckDuckGo (Free)</option>
                      <option value="tavily">Tavily (API Key)</option>
                    </select>
                  </label>

                  {searchProvider === "tavily" && (
                    <label className="block text-sm text-text-secondary">
                      Tavily API Key
                      <input
                        type="password"
                        value={tavilyKey}
                        onChange={(e) => setTavilyKey(e.target.value)}
                        placeholder="tvly-..."
                        className="w-full mt-1.5 px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                      />
                    </label>
                  )}
                </div>
              )}

              <button
                onClick={handleSaveSearch}
                className="mt-5 w-full px-4 py-2.5 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none"
              >
                Continue
              </button>
            </div>
          )}

          {/* Step 5: Ready */}
          {wizardStep === "ready" && (
            <div className="text-center">
              <Flame
                className="w-16 h-16 text-accent-glow mx-auto mb-4 animate-[flame-flicker_3s_ease-in-out_infinite]"
                style={{
                  filter: "drop-shadow(0 0 25px rgba(232,160,69,0.6))",
                }}
              />
              <h2 className="text-2xl font-heading font-semibold text-text-primary mb-2">
                You're all set!
              </h2>
              <p className="text-sm text-text-secondary mb-6">
                Your forge is ready. Start by describing what you want to build.
              </p>
              <button
                onClick={handleFinish}
                className="px-8 py-2.5 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer border-none mb-3"
              >
                Start Planning
              </button>
              <button
                onClick={() => {
                  completeWizard();
                  setShowSettings(true);
                }}
                className="text-xs text-text-secondary hover:text-text-primary transition-colors cursor-pointer bg-transparent border-none underline block mx-auto"
              >
                Configure web search (optional)
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
