import { X } from "lucide-react";

interface HelpPanelProps {
  open: boolean;
  onClose: () => void;
}

export function HelpPanel({ open, onClose }: HelpPanelProps) {
  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div
        className="absolute right-0 top-0 h-full w-full max-w-sm bg-elevated border-l border-border-default shadow-lg overflow-y-auto animate-[slide-in-right_0.3s_ease]"
        role="dialog"
        aria-label="Help"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border-subtle sticky top-0 bg-elevated z-10">
          <h2 className="text-lg font-heading font-semibold text-text-primary">
            Help
          </h2>
          <button
            onClick={onClose}
            aria-label="Close help"
            className="w-8 h-8 flex items-center justify-center rounded-lg text-text-secondary hover:bg-surface hover:text-text-primary transition-all cursor-pointer bg-transparent border-none"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="px-5 py-4 space-y-2">
          {/* What is AuraForge */}
          <details className="group">
            <summary className="text-sm font-medium text-text-primary cursor-pointer py-2 hover:text-accent-gold transition-colors list-none flex items-center justify-between">
              What is AuraForge?
              <span className="text-text-muted text-xs group-open:rotate-90 transition-transform">
                ›
              </span>
            </summary>
            <div className="text-sm text-text-secondary pb-3 pl-1">
              AuraForge is a project planning assistant that runs entirely on
              your Mac. Describe your project idea in a conversation, and it
              generates planning documents you can use with Claude Code or any
              AI coding tool.
            </div>
          </details>

          {/* Generated Documents */}
          <details className="group">
            <summary className="text-sm font-medium text-text-primary cursor-pointer py-2 hover:text-accent-gold transition-colors list-none flex items-center justify-between">
              What are the generated documents?
              <span className="text-text-muted text-xs group-open:rotate-90 transition-transform">
                ›
              </span>
            </summary>
            <div className="text-sm text-text-secondary pb-3 pl-1">
              <table className="w-full text-xs mt-1">
                <tbody className="divide-y divide-border-subtle">
                  <tr>
                    <td className="py-1.5 pr-3 font-mono text-accent-glow whitespace-nowrap">
                      README.md
                    </td>
                    <td className="py-1.5 text-text-muted">
                      Project overview and setup instructions
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1.5 pr-3 font-mono text-accent-glow whitespace-nowrap">
                      SPEC.md
                    </td>
                    <td className="py-1.5 text-text-muted">
                      Technical specification and architecture
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1.5 pr-3 font-mono text-accent-glow whitespace-nowrap">
                      CLAUDE.md
                    </td>
                    <td className="py-1.5 text-text-muted">
                      Instructions for Claude Code
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1.5 pr-3 font-mono text-accent-glow whitespace-nowrap">
                      PROMPTS.md
                    </td>
                    <td className="py-1.5 text-text-muted">
                      Prompts to kick off each phase
                    </td>
                  </tr>
                  <tr>
                    <td className="py-1.5 pr-3 font-mono text-accent-glow whitespace-nowrap">
                      CONVERSATION.md
                    </td>
                    <td className="py-1.5 text-text-muted">
                      Full chat history export
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </details>

          {/* What is Ollama */}
          <details className="group">
            <summary className="text-sm font-medium text-text-primary cursor-pointer py-2 hover:text-accent-gold transition-colors list-none flex items-center justify-between">
              What is Ollama?
              <span className="text-text-muted text-xs group-open:rotate-90 transition-transform">
                ›
              </span>
            </summary>
            <div className="text-sm text-text-secondary pb-3 pl-1">
              Ollama runs AI models locally on your Mac. Your conversations
              never leave your machine &mdash; everything is completely private.
              AuraForge uses Ollama to power its planning assistant.
            </div>
          </details>

          {/* What is Web Search */}
          <details className="group">
            <summary className="text-sm font-medium text-text-primary cursor-pointer py-2 hover:text-accent-gold transition-colors list-none flex items-center justify-between">
              What is web search?
              <span className="text-text-muted text-xs group-open:rotate-90 transition-transform">
                ›
              </span>
            </summary>
            <div className="text-sm text-text-secondary pb-3 pl-1">
              Optionally, AuraForge can search the web to provide up-to-date
              information about frameworks, libraries, and best practices. You
              can use the free DuckDuckGo provider or add a Tavily API key for
              better results. This is configured in Settings.
            </div>
          </details>

          {/* Keyboard Shortcuts */}
          <details className="group" open>
            <summary className="text-sm font-medium text-text-primary cursor-pointer py-2 hover:text-accent-gold transition-colors list-none flex items-center justify-between">
              Keyboard shortcuts
              <span className="text-text-muted text-xs group-open:rotate-90 transition-transform">
                ›
              </span>
            </summary>
            <div className="pb-3 pl-1">
              <table className="w-full text-xs mt-1">
                <tbody className="divide-y divide-border-subtle">
                  {[
                    ["⌘ N", "New project"],
                    ["⌘ G", "Generate documents"],
                    ["⌘ S", "Save to folder"],
                    ["⌘ ,", "Settings"],
                    ["⌘ \\", "Toggle sidebar"],
                    ["⇧⌘ P", "Toggle preview"],
                    ["⌘ /", "Toggle help"],
                    ["⌘ Enter", "Send message"],
                    ["Escape", "Close panel"],
                  ].map(([key, desc]) => (
                    <tr key={key}>
                      <td className="py-1.5 pr-3">
                        <kbd className="px-1.5 py-0.5 bg-surface rounded text-text-muted font-mono text-[11px]">
                          {key}
                        </kbd>
                      </td>
                      <td className="py-1.5 text-text-secondary">{desc}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </details>
        </div>
      </div>
    </div>
  );
}
