import { useState, useRef, useEffect, useCallback } from "react";
import {
  Flame,
  Trash2,
  MessageSquare,
  Settings,
  HelpCircle,
  CheckSquare,
  Square,
  XCircle,
} from "lucide-react";
import { clsx } from "clsx";
import { useChatStore } from "../stores/chatStore";

export function Sidebar() {
  const {
    sessions,
    currentSessionId,
    createSession,
    selectSession,
    deleteSession,
    deleteSessions,
    renameSession,
    setShowSettings,
    setShowHelp,
    sidebarCollapsed,
  } = useChatStore();

  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const deleteTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Select mode state
  const [selectMode, setSelectMode] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Inline rename state
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const renameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    return () => {
      if (deleteTimerRef.current) clearTimeout(deleteTimerRef.current);
    };
  }, []);

  // Exit select mode if sessions drop below 2
  useEffect(() => {
    if (selectMode && sessions.length < 2) {
      setSelectMode(false);
      setSelectedIds(new Set());
    }
  }, [sessions.length, selectMode]);

  const handleDelete = async (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    if (confirmDeleteId === sessionId) {
      if (deleteTimerRef.current) clearTimeout(deleteTimerRef.current);
      await deleteSession(sessionId);
      setConfirmDeleteId(null);
    } else {
      setConfirmDeleteId(sessionId);
      if (deleteTimerRef.current) clearTimeout(deleteTimerRef.current);
      deleteTimerRef.current = setTimeout(() => setConfirmDeleteId(null), 3000);
    }
  };

  const toggleSelection = useCallback((sessionId: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(sessionId)) {
        next.delete(sessionId);
      } else {
        next.add(sessionId);
      }
      return next;
    });
  }, []);

  const toggleSelectAll = useCallback(() => {
    setSelectedIds((prev) => {
      if (prev.size === sessions.length) {
        return new Set();
      }
      return new Set(sessions.map((s) => s.id));
    });
  }, [sessions]);

  const exitSelectMode = useCallback(() => {
    setSelectMode(false);
    setSelectedIds(new Set());
  }, []);

  const handleBatchDelete = useCallback(async () => {
    if (selectedIds.size === 0) return;
    const count = selectedIds.size;
    const confirmed = window.confirm(
      `Delete ${count} project${count > 1 ? "s" : ""}? This cannot be undone.`,
    );
    if (!confirmed) return;
    await deleteSessions(Array.from(selectedIds));
    setSelectMode(false);
    setSelectedIds(new Set());
  }, [selectedIds, deleteSessions]);

  const startRename = useCallback(
    (e: React.MouseEvent, session: { id: string; name: string }) => {
      e.stopPropagation();
      if (selectMode) return;
      setRenamingId(session.id);
      setEditName(session.name);
      // Auto-focus + select-all happens via onMount effect on the input
    },
    [selectMode],
  );

  const commitRename = useCallback(
    async (sessionId: string) => {
      const trimmed = editName.trim();
      const session = sessions.find((s) => s.id === sessionId);
      if (trimmed && session && trimmed !== session.name) {
        await renameSession(sessionId, trimmed);
      }
      setRenamingId(null);
    },
    [editName, sessions, renameSession],
  );

  const cancelRename = useCallback(() => {
    setRenamingId(null);
  }, []);

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

  const allSelected = sessions.length > 0 && selectedIds.size === sessions.length;

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

      {/* Action Bar */}
      <div className="px-3 pt-3 pb-1">
        {selectMode ? (
          <div className="flex items-center gap-1.5">
            <button
              onClick={toggleSelectAll}
              aria-label={allSelected ? "Deselect all" : "Select all"}
              className="flex-1 px-3 py-2 rounded-lg text-text-secondary text-xs font-medium cursor-pointer transition-all duration-200 hover:bg-surface bg-transparent border border-border-subtle"
            >
              {allSelected ? "Deselect All" : "Select All"}
            </button>
            {selectedIds.size > 0 && (
              <button
                onClick={handleBatchDelete}
                aria-label={`Delete ${selectedIds.size} selected`}
                className="flex-1 px-3 py-2 rounded-lg text-status-error text-xs font-medium cursor-pointer transition-all duration-200 hover:bg-status-error/10 bg-transparent border border-status-error/40"
              >
                Delete ({selectedIds.size})
              </button>
            )}
            <button
              onClick={exitSelectMode}
              aria-label="Exit select mode"
              className="shrink-0 p-2 rounded-lg text-text-muted cursor-pointer transition-all duration-200 hover:text-text-primary hover:bg-surface bg-transparent border-none"
            >
              <XCircle className="w-4 h-4" />
            </button>
          </div>
        ) : (
          <div className="flex items-center gap-1.5">
            <button
              onClick={() => createSession()}
              aria-label="Create new project"
              className="flex-1 px-4 py-2.5 border border-accent-gold rounded-lg text-accent-gold font-heading text-sm font-medium cursor-pointer transition-all duration-300 hover:bg-accent-gold/10 hover:shadow-glow active:scale-[0.98] bg-transparent"
            >
              + New Project
            </button>
            {sessions.length >= 2 && (
              <button
                onClick={() => setSelectMode(true)}
                aria-label="Enter select mode"
                className="shrink-0 p-2.5 rounded-lg text-text-muted cursor-pointer transition-all duration-200 hover:text-text-primary hover:bg-surface bg-transparent border border-border-subtle"
              >
                <CheckSquare className="w-4 h-4" />
              </button>
            )}
          </div>
        )}
      </div>

      {/* Session List */}
      <nav className="flex-1 overflow-y-auto px-2 py-2" aria-label="Project list">
        {sessions.length === 0 ? (
          <div className="px-3 py-6 text-center">
            <p className="text-text-muted text-xs">No projects yet</p>
          </div>
        ) : (
          sessions.map((session) => {
            const isSelected = selectedIds.has(session.id);
            return (
              <button
                key={session.id}
                onClick={
                  selectMode
                    ? () => toggleSelection(session.id)
                    : () => selectSession(session.id)
                }
                aria-label={
                  selectMode
                    ? `${isSelected ? "Deselect" : "Select"}: ${session.name}`
                    : `Select project: ${session.name}`
                }
                aria-current={
                  !selectMode && session.id === currentSessionId
                    ? "true"
                    : undefined
                }
                className={clsx(
                  "w-full text-left px-3 py-2.5 my-0.5 rounded-lg border transition-all duration-200 cursor-pointer group",
                  selectMode && isSelected
                    ? "bg-accent-gold/10 border-accent-gold/40"
                    : !selectMode && session.id === currentSessionId
                      ? "bg-warm border-l-[3px] border-l-accent-glow border-t-transparent border-r-transparent border-b-transparent shadow-glow"
                      : "bg-transparent border-transparent hover:bg-surface hover:border-border-subtle",
                )}
              >
                <div className="flex items-start gap-2">
                  {selectMode ? (
                    isSelected ? (
                      <CheckSquare
                        className="w-4 h-4 mt-0.5 shrink-0 text-accent-glow"
                        aria-hidden="true"
                      />
                    ) : (
                      <Square
                        className="w-4 h-4 mt-0.5 shrink-0 text-text-muted"
                        aria-hidden="true"
                      />
                    )
                  ) : (
                    <MessageSquare
                      className={clsx(
                        "w-4 h-4 mt-0.5 shrink-0",
                        session.id === currentSessionId
                          ? "text-accent-glow"
                          : "text-text-muted",
                      )}
                      aria-hidden="true"
                    />
                  )}
                  <div className="flex-1 min-w-0">
                    {renamingId === session.id ? (
                      <input
                        ref={renameInputRef}
                        value={editName}
                        onChange={(e) => setEditName(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") {
                            e.preventDefault();
                            commitRename(session.id);
                          } else if (e.key === "Escape") {
                            cancelRename();
                          }
                        }}
                        onBlur={() => commitRename(session.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="w-full text-sm text-text-primary font-medium bg-transparent border border-accent-gold/60 rounded px-1 py-0 outline-none focus:border-accent-glow"
                        autoFocus
                        onFocus={(e) => e.target.select()}
                      />
                    ) : (
                      <div
                        className="text-sm text-text-primary truncate font-medium"
                        onDoubleClick={(e) => startRename(e, session)}
                      >
                        {session.name}
                      </div>
                    )}
                    <div className="text-[11px] text-text-muted mt-0.5">
                      {formatTimestamp(session.updated_at)}
                    </div>
                  </div>
                  {!selectMode && (
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
                  )}
                </div>
              </button>
            );
          })
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

    </aside>
  );
}
