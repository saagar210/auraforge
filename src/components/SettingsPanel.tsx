import { useEffect, useState } from "react";
import { X, RotateCcw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../stores/chatStore";
import type {
  AppConfig,
  LLMConfig,
  SearchConfig,
  OutputConfig,
  UIConfig,
  ForgeTarget,
} from "../types";

interface SettingsPanelProps {
  open: boolean;
  onClose: () => void;
}

type Section = "llm" | "search" | "output";

const SECTIONS: { key: Section; label: string }[] = [
  { key: "llm", label: "LLM" },
  { key: "search", label: "Search" },
  { key: "output", label: "Output" },
];

export function SettingsPanel({ open, onClose }: SettingsPanelProps) {
  const { loadConfig, updateConfig, listModels, installedModels } =
    useChatStore();

  const [isAdvanced, setIsAdvanced] = useState(false);
  const [activeSection, setActiveSection] = useState<Section>("llm");
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // LLM state
  const [llmProvider, setLlmProvider] = useState("ollama");
  const [model, setModel] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [llmApiKey, setLlmApiKey] = useState("");
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(65536);

  // Search state
  const [searchEnabled, setSearchEnabled] = useState(true);
  const [searchProvider, setSearchProvider] = useState("tavily");
  const [tavilyApiKey, setTavilyApiKey] = useState("");
  const [searxngUrl, setSearxngUrl] = useState("");
  const [proactive, setProactive] = useState(true);

  // Output state
  const [includeConversation, setIncludeConversation] = useState(true);
  const [defaultSavePath, setDefaultSavePath] = useState("~/Projects");
  const [defaultTarget, setDefaultTarget] = useState<ForgeTarget>("generic");
  const [uiTheme, setUiTheme] = useState<UIConfig["theme"]>("dark");

  useEffect(() => {
    if (open) {
      loadConfig().then((config: AppConfig | null) => {
        if (!config) return;
        setLlmProvider(config.llm.provider);
        setModel(config.llm.model);
        setBaseUrl(config.llm.base_url);
        setLlmApiKey(config.llm.api_key ?? "");
        setTemperature(config.llm.temperature);
        setMaxTokens(config.llm.max_tokens);
        setSearchEnabled(config.search.enabled);
        setSearchProvider(config.search.provider);
        setTavilyApiKey(config.search.tavily_api_key);
        setSearxngUrl(config.search.searxng_url);
        setProactive(config.search.proactive);
        setIncludeConversation(config.output.include_conversation);
        setDefaultSavePath(config.output.default_save_path);
        setDefaultTarget(config.output.default_target);
        setUiTheme(config.ui.theme);
      });
      // Load installed models for dropdown
      listModels();
    }
  }, [open, loadConfig, listModels]);

  if (!open) return null;

  const handleSave = async () => {
    setSaving(true);

    const llm: LLMConfig = {
      provider: llmProvider as LLMConfig["provider"],
      model,
      base_url: baseUrl,
      api_key: llmApiKey,
      temperature,
      max_tokens: maxTokens,
    };

    const search: SearchConfig = {
      enabled: searchEnabled,
      provider: searchProvider as SearchConfig["provider"],
      tavily_api_key: tavilyApiKey,
      searxng_url: searxngUrl,
      proactive,
    };

    const output: OutputConfig = {
      include_conversation: includeConversation,
      default_save_path: defaultSavePath,
      default_target: defaultTarget,
    };

    const ui: UIConfig = {
      theme: uiTheme,
    };

    const result = await updateConfig({ llm, search, ui, output });
    setSaving(false);
    if (result) {
      setSaveError(null);
      onClose();
    } else {
      setSaveError("Invalid settings. Check your provider, API keys, and URLs.");
    }
  };

  const handleRerunSetup = async () => {
    await invoke("set_preference", { key: "wizard_completed", value: "false" });
    useChatStore.setState({ wizardCompleted: false, onboardingDismissed: false, wizardStep: "welcome" });
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center animate-[fade-in_0.2s_ease]"
      style={{
        background: "rgba(0, 0, 0, 0.7)",
        backdropFilter: "blur(4px)",
      }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-label="Settings"
    >
      <div
        className="bg-elevated border border-border-default rounded-2xl w-full max-w-lg mx-4 shadow-lg overflow-hidden animate-[modal-in_0.3s_ease]"
        style={{ maxHeight: "80vh" }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-subtle">
          <h2 className="text-xl font-heading font-semibold text-text-primary">
            Settings
          </h2>
          <button
            onClick={onClose}
            aria-label="Close settings"
            className="w-8 h-8 flex items-center justify-center rounded-lg text-text-secondary hover:bg-surface hover:text-text-primary transition-all cursor-pointer bg-transparent border-none"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div
          className="px-6 py-5 overflow-y-auto"
          style={{ maxHeight: "calc(80vh - 140px)" }}
        >
          {!isAdvanced ? (
            /* =================== SIMPLE MODE =================== */
            <div className="space-y-5">
              {/* AI Model Dropdown */}
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">
                  AI Model
                </label>
                {installedModels.length > 0 ? (
                  <select
                    value={model}
                    onChange={(e) => setModel(e.target.value)}
                    className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                  >
                    {installedModels.map((m) => (
                      <option key={m} value={m}>
                        {m}
                      </option>
                    ))}
                  </select>
                ) : (
                  <input
                    type="text"
                    value={model}
                    onChange={(e) => setModel(e.target.value)}
                    placeholder="e.g. qwen3-coder:30b"
                    className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                  />
                )}
              </div>

              {/* Web Search Toggle */}
              <div>
                <label className="flex items-center justify-between cursor-pointer">
                  <span className="text-sm text-text-primary">Web Search</span>
                  <Toggle
                    checked={searchEnabled}
                    onChange={setSearchEnabled}
                  />
                </label>

                {searchEnabled && !tavilyApiKey && (
                  <div className="mt-2 text-xs text-text-muted">
                    Using basic search. For better results, add a{" "}
                    <a
                      href="https://tavily.com"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-accent-glow hover:text-accent-gold underline"
                    >
                      Tavily API key
                    </a>{" "}
                    (free):
                    <input
                      type="password"
                      value={tavilyApiKey}
                      onChange={(e) => {
                        setTavilyApiKey(e.target.value);
                        if (e.target.value) setSearchProvider("tavily");
                      }}
                      placeholder="tvly-..."
                      className="w-full mt-1.5 px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                    />
                  </div>
                )}
              </div>

              {/* Default Save Path */}
              <div>
                <label className="block text-sm text-text-secondary mb-1.5">
                  Save Location
                </label>
                <input
                  type="text"
                  value={defaultSavePath}
                  onChange={(e) => setDefaultSavePath(e.target.value)}
                  placeholder="~/Projects"
                  className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                />
              </div>

              <div>
                <label className="block text-sm text-text-secondary mb-1.5">
                  Default Output Target
                </label>
                <select
                  value={defaultTarget}
                  onChange={(e) => setDefaultTarget(e.target.value as ForgeTarget)}
                  className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                >
                  <option value="generic">Any model</option>
                  <option value="codex">Codex</option>
                  <option value="claude">Claude</option>
                  <option value="cursor">Cursor</option>
                  <option value="gemini">Gemini</option>
                </select>
              </div>

              {/* Re-run Setup */}
              <button
                onClick={handleRerunSetup}
                className="flex items-center gap-2 text-xs text-text-secondary hover:text-text-primary transition-colors cursor-pointer bg-transparent border border-border-default rounded-lg px-3 py-2"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                Re-run Setup
              </button>

              {/* Advanced toggle */}
              <button
                onClick={() => setIsAdvanced(true)}
                className="text-xs text-text-muted hover:text-accent-gold transition-colors cursor-pointer bg-transparent border-none underline"
              >
                Show advanced settings
              </button>
            </div>
          ) : (
            /* =================== ADVANCED MODE =================== */
            <div className="space-y-5">
              <button
                onClick={() => setIsAdvanced(false)}
                className="text-xs text-text-muted hover:text-accent-gold transition-colors cursor-pointer bg-transparent border-none underline mb-2"
              >
                &larr; Simple settings
              </button>

              {/* Section Tabs */}
              <div className="flex gap-1">
                {SECTIONS.map((s) => (
                  <button
                    key={s.key}
                    onClick={() => setActiveSection(s.key)}
                    className={`px-3 py-1.5 text-xs font-heading font-medium rounded-md transition-colors cursor-pointer border-none ${
                      activeSection === s.key
                        ? "bg-surface text-accent-gold"
                        : "bg-transparent text-text-secondary hover:text-text-primary"
                    }`}
                  >
                    {s.label}
                  </button>
                ))}
              </div>

              {activeSection === "llm" && (
                <div className="space-y-5">
                  <h3 className="text-sm font-heading font-medium text-text-secondary uppercase tracking-wider">
                    Language Model
                  </h3>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Provider
                    </label>
                    <select
                      value={llmProvider}
                      onChange={(e) => setLlmProvider(e.target.value)}
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                    >
                      <option value="ollama">Ollama (Local)</option>
                      <option value="anthropic">Anthropic</option>
                      <option value="openai">OpenAI</option>
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Model
                    </label>
                    <input
                      type="text"
                      value={model}
                      onChange={(e) => setModel(e.target.value)}
                      placeholder="e.g. qwen3-coder:30b"
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                    />
                  </div>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Base URL
                    </label>
                    <input
                      type="text"
                      value={baseUrl}
                      onChange={(e) => setBaseUrl(e.target.value)}
                      placeholder="http://localhost:11434"
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                    />
                  </div>

                  {llmProvider !== "ollama" && (
                    <div>
                      <label className="block text-sm text-text-secondary mb-1.5">
                        API Key
                      </label>
                      <input
                        type="password"
                        value={llmApiKey}
                        onChange={(e) => setLlmApiKey(e.target.value)}
                        placeholder="Provider API key"
                        className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                      />
                    </div>
                  )}

                  <div>
                    <label className="flex items-center justify-between text-sm text-text-secondary mb-1.5">
                      <span>Temperature</span>
                      <span className="text-text-muted font-mono text-xs">
                        {temperature.toFixed(1)}
                      </span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="2"
                      step="0.1"
                      value={temperature}
                      onChange={(e) =>
                        setTemperature(parseFloat(e.target.value))
                      }
                      className="w-full accent-accent-gold"
                    />
                  </div>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Max Tokens
                    </label>
                    <input
                      type="number"
                      min="1"
                      value={maxTokens}
                      onChange={(e) =>
                        setMaxTokens(Math.max(1, parseInt(e.target.value) || 1))
                      }
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                    />
                  </div>
                </div>
              )}

              {activeSection === "search" && (
                <div className="space-y-5">
                  <h3 className="text-sm font-heading font-medium text-text-secondary uppercase tracking-wider">
                    Web Search
                  </h3>

                  <label className="flex items-center justify-between cursor-pointer">
                    <span className="text-sm text-text-primary">
                      Enable Web Search
                    </span>
                    <Toggle
                      checked={searchEnabled}
                      onChange={setSearchEnabled}
                    />
                  </label>

                  {searchEnabled && (
                    <>
                      <div>
                        <label className="block text-sm text-text-secondary mb-1.5">
                          Provider
                        </label>
                        <select
                          value={searchProvider}
                          onChange={(e) => setSearchProvider(e.target.value)}
                          className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                        >
                          <option value="tavily">Tavily</option>
                          <option value="duckduckgo">DuckDuckGo</option>
                          <option value="searxng">SearXNG</option>
                        </select>
                      </div>

                      {searchProvider === "tavily" && (
                        <div>
                          <label className="block text-sm text-text-secondary mb-1.5">
                            Tavily API Key
                          </label>
                          <input
                            type="password"
                            value={tavilyApiKey}
                            onChange={(e) => setTavilyApiKey(e.target.value)}
                            placeholder="tvly-..."
                            className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                          />
                        </div>
                      )}

                      {searchProvider === "searxng" && (
                        <div>
                          <label className="block text-sm text-text-secondary mb-1.5">
                            SearXNG URL
                          </label>
                          <input
                            type="text"
                            value={searxngUrl}
                            onChange={(e) => setSearxngUrl(e.target.value)}
                            placeholder="http://localhost:8080"
                            className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                          />
                        </div>
                      )}

                      <label className="flex items-center justify-between cursor-pointer">
                        <div>
                          <span className="text-sm text-text-primary block">
                            Proactive Search
                          </span>
                          <span className="text-[11px] text-text-muted">
                            Automatically search when tech topics are detected
                          </span>
                        </div>
                        <Toggle checked={proactive} onChange={setProactive} />
                      </label>
                    </>
                  )}
                </div>
              )}

              {activeSection === "output" && (
                <div className="space-y-5">
                  <h3 className="text-sm font-heading font-medium text-text-secondary uppercase tracking-wider">
                    Output
                  </h3>

                  <label className="flex items-center justify-between cursor-pointer">
                    <div>
                      <span className="text-sm text-text-primary block">
                        Include CONVERSATION.md
                      </span>
                      <span className="text-[11px] text-text-muted">
                        Export the full chat history as a document
                      </span>
                    </div>
                    <Toggle
                      checked={includeConversation}
                      onChange={setIncludeConversation}
                    />
                  </label>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Default Save Path
                    </label>
                    <input
                      type="text"
                      value={defaultSavePath}
                      onChange={(e) => setDefaultSavePath(e.target.value)}
                      placeholder="~/Projects"
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors font-mono text-[13px]"
                    />
                  </div>

                  <div>
                    <label className="block text-sm text-text-secondary mb-1.5">
                      Default Output Target
                    </label>
                    <select
                      value={defaultTarget}
                      onChange={(e) => setDefaultTarget(e.target.value as ForgeTarget)}
                      className="w-full px-3 py-2 bg-surface border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] transition-colors"
                    >
                      <option value="generic">Any model</option>
                      <option value="codex">Codex</option>
                      <option value="claude">Claude</option>
                      <option value="cursor">Cursor</option>
                      <option value="gemini">Gemini</option>
                    </select>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border-subtle">
          {saveError && (
            <p className="text-xs text-status-error mb-3">{saveError}</p>
          )}
          <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-text-secondary bg-transparent border border-border-default rounded-lg hover:text-text-primary hover:border-text-muted transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer disabled:opacity-50 border-none"
          >
            {saving ? "Saving..." : "Save"}
          </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function Toggle({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-5 rounded-full transition-colors duration-200 cursor-pointer border-none shrink-0 ${
        checked ? "bg-accent-gold" : "bg-surface"
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform duration-200 ${
          checked ? "translate-x-5" : "translate-x-0"
        }`}
      />
    </button>
  );
}
