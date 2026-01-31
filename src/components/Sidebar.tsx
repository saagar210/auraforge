import { useState } from "react";
import { Flame, Trash2, MessageSquare, Settings, HelpCircle } from "lucide-react";
import { clsx } from "clsx";
import { useChatStore } from "../stores/chatStore";
import { SettingsPanel } from "./SettingsPanel";

export function Sidebar() {
  const {
    sessions,
    currentSessionId,
    createSession,
    selectSession,
    deleteSession,
    showSettings,
    setShowSettings,
    setShowHelp,
    sidebarCollapsed,
  } = useChatStore();

  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  const handleDelete = async (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    if (confirmDeleteId === sessionId) {
      await deleteSession(sessionId);
      setConfirmDeleteId(null);
    } else {
      setConfirmDeleteId(sessionId);
      setTimeout(() => setConfirmDeleteId(null), 3000);
    }
  };

  const formatTimestamp = (ts: string) => {
    try {
      const date = new Date(ts + "Z");
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffMins = Math.floor(diffMs / 60000);

      if (diffMins < 1) return "Just now";
      if (diffMins < 60) return `${diffMins}m ago`;
      const diffHours = Math.floor(diffMins / 60);
      if (diffHours < 24) return `${diffHours}h ago`;
      const diffDays = Math.floor(diffHours / 24);
      if (diffDays < 7) return `${diffDays}d ago`;
      return date.toLocaleDateString();
    } catch {
      return "";
    }
  };

  if (sidebarCollapsed) {
    return null;
  }

  return (
    <aside
      className="w-[260px] min-w-[260px] bg-elevated border-r border-border-subtle flex flex-col h-full relative z-10"
      aria-label="Project sidebar"
    >
      {/* Header */}
      <div className="flex items-center gap-2.5 px-4 py-4 border-b border-border-subtle">
        <Flame
          className="w-5 h-5 text-accent-glow"
          style={{ filter: "drop-shadow(0 0 8px rgba(232, 160, 69, 0.4))" }}
          aria-hidden="true"
        />
        <span className="font-heading text-base font-semibold text-text-primary tracking-wide">
          AuraForge
        </span>
      </div>

      {/* New Project Button */}
      <div className="px-3 pt-3 pb-1">
        <button
          onClick={() => createSession()}
          aria-label="Create new project"
          className="w-full px-4 py-2.5 border border-accent-gold rounded-lg text-accent-gold font-heading text-sm font-medium cursor-pointer transition-all duration-300 hover:bg-accent-gold/10 hover:shadow-glow active:scale-[0.98] bg-transparent"
        >
          + New Project
        </button>
      </div>

      {/* Session List */}
      <nav className="flex-1 overflow-y-auto px-2 py-2" aria-label="Project list">
        {sessions.length === 0 ? (
          <div className="px-3 py-6 text-center">
            <p className="text-text-muted text-xs">No projects yet</p>
          </div>
        ) : (
          sessions.map((session) => (
            <button
              key={session.id}
              onClick={() => selectSession(session.id)}
              aria-label={`Select project: ${session.name}`}
              aria-current={session.id === currentSessionId ? "true" : undefined}
              className={clsx(
                "w-full text-left px-3 py-2.5 my-0.5 rounded-lg border transition-all duration-200 cursor-pointer group",
                session.id === currentSessionId
                  ? "bg-warm border-l-[3px] border-l-accent-glow border-t-transparent border-r-transparent border-b-transparent shadow-glow"
                  : "bg-transparent border-transparent hover:bg-surface hover:border-border-subtle",
              )}
            >
              <div className="flex items-start gap-2">
                <MessageSquare
                  className={clsx(
                    "w-4 h-4 mt-0.5 shrink-0",
                    session.id === currentSessionId
                      ? "text-accent-glow"
                      : "text-text-muted",
                  )}
                  aria-hidden="true"
                />
                <div className="flex-1 min-w-0">
                  <div className="text-sm text-text-primary truncate font-medium">
                    {session.name}
                  </div>
                  <div className="text-[11px] text-text-muted mt-0.5">
                    {formatTimestamp(session.updated_at)}
                  </div>
                </div>
                <button
                  onClick={(e) => handleDelete(e, session.id)}
                  aria-label={
                    confirmDeleteId === session.id
                      ? `Confirm delete: ${session.name}`
                      : `Delete project: ${session.name}`
                  }
                  className={clsx(
                    "shrink-0 p-1 rounded transition-all duration-200 bg-transparent border-none cursor-pointer",
                    confirmDeleteId === session.id
                      ? "text-status-error opacity-100"
                      : "text-text-muted opacity-0 group-hover:opacity-100 hover:text-status-error",
                  )}
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </button>
          ))
        )}
      </nav>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-border-subtle flex items-center gap-3">
        <button
          onClick={() => setShowSettings(true)}
          aria-label="Open settings"
          className="flex items-center gap-2 text-text-secondary text-xs hover:text-text-primary transition-colors duration-200 cursor-pointer bg-transparent border-none"
        >
          <Settings className="w-4 h-4" aria-hidden="true" />
          <span>Settings</span>
        </button>
        <button
          onClick={() => setShowHelp(true)}
          aria-label="Open help"
          className="flex items-center gap-2 text-text-secondary text-xs hover:text-text-primary transition-colors duration-200 cursor-pointer bg-transparent border-none ml-auto"
        >
          <HelpCircle className="w-4 h-4" aria-hidden="true" />
        </button>
      </div>

      <SettingsPanel
        open={showSettings}
        onClose={() => setShowSettings(false)}
      />
    </aside>
  );
}
