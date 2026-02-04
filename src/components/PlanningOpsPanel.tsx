import { useEffect, useState, type ComponentType } from "react";
import { X, Sparkles, GitBranch, FolderSearch, ListChecks } from "lucide-react";
import { useChatStore } from "../stores/chatStore";
import type { PlanTemplate } from "../types";

type OpsTab = "templates" | "import" | "branches" | "export";

interface PlanningOpsPanelProps {
  open: boolean;
  onClose: () => void;
  onApplyTemplate: (template: PlanTemplate) => void;
}

const TABS: Array<{ key: OpsTab; label: string; icon: ComponentType<{ className?: string }> }> =
  [
    { key: "templates", label: "Templates", icon: Sparkles },
    { key: "import", label: "Repo Import", icon: FolderSearch },
    { key: "branches", label: "Branches", icon: GitBranch },
    { key: "export", label: "Issue Export", icon: ListChecks },
  ];

export function PlanningOpsPanel({ open, onClose, onApplyTemplate }: PlanningOpsPanelProps) {
  const {
    currentSessionId,
    templates,
    repoImportContext,
    branches,
    exportPreview,
    loadTemplates,
    importRepositoryContext,
    loadBranches,
    createBranch,
    loadIssueExportPreview,
  } = useChatStore();

  const [activeTab, setActiveTab] = useState<OpsTab>("templates");
  const [repoPath, setRepoPath] = useState("");
  const [branchName, setBranchName] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;
    if (templates.length === 0) {
      void loadTemplates();
    }
    if (currentSessionId) {
      void loadBranches();
    }
  }, [open, templates.length, currentSessionId, loadTemplates, loadBranches]);

  if (!open) return null;

  const runWithLoading = async (fn: () => Promise<void>) => {
    setLoading(true);
    try {
      await fn();
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: "rgba(0,0,0,0.72)", backdropFilter: "blur(4px)" }}
      onClick={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-label="Planning tools"
    >
      <div className="w-full max-w-3xl mx-4 bg-elevated border border-border-default rounded-2xl overflow-hidden shadow-lg">
        <div className="flex items-center justify-between px-5 py-3 border-b border-border-subtle">
          <h2 className="text-lg font-heading text-text-primary">Planning Tools</h2>
          <button
            onClick={onClose}
            aria-label="Close planning tools"
            className="w-8 h-8 flex items-center justify-center rounded-lg text-text-secondary hover:bg-surface hover:text-text-primary transition-colors bg-transparent border-none cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="flex border-b border-border-subtle px-3 pt-2 gap-1">
          {TABS.map((tab) => {
            const Icon = tab.icon;
            const active = tab.key === activeTab;
            return (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`px-3 py-2 text-xs rounded-t-lg border-none cursor-pointer flex items-center gap-1.5 transition-colors ${
                  active
                    ? "bg-surface text-text-primary"
                    : "bg-transparent text-text-secondary hover:text-text-primary"
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                {tab.label}
              </button>
            );
          })}
        </div>

        <div className="px-5 py-4 min-h-[340px] max-h-[65vh] overflow-y-auto">
          {activeTab === "templates" && (
            <div className="space-y-3">
              {templates.map((template) => (
                <div
                  key={template.id}
                  className="border border-border-subtle rounded-xl px-4 py-3 bg-surface/40"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="text-sm text-text-primary font-medium">{template.name}</p>
                      <p className="text-xs text-text-secondary mt-1">{template.description}</p>
                      <p className="text-[11px] text-text-muted mt-2">
                        {template.tags.join(" · ")}
                      </p>
                    </div>
                    <button
                      onClick={() => onApplyTemplate(template)}
                      className="px-3 py-1.5 text-xs rounded-md border border-accent-gold/40 text-accent-gold bg-transparent hover:bg-accent-gold/10 transition-colors cursor-pointer"
                    >
                      Use
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {activeTab === "import" && (
            <div className="space-y-3">
              <p className="text-sm text-text-secondary">
                Scan an existing repo to seed planning with detected stack and key files.
              </p>
              <div className="flex gap-2">
                <input
                  value={repoPath}
                  onChange={(e) => setRepoPath(e.target.value)}
                  placeholder="/path/to/repository"
                  className="flex-1 px-3 py-2 rounded-lg bg-surface border border-border-default text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow"
                />
                <button
                  onClick={() =>
                    runWithLoading(async () => {
                      await importRepositoryContext(repoPath.trim());
                    })
                  }
                  disabled={!repoPath.trim() || loading}
                  className="px-3 py-2 text-xs rounded-lg border border-accent-gold/40 text-accent-gold bg-transparent hover:bg-accent-gold/10 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                >
                  Analyze
                </button>
              </div>
              {repoImportContext && (
                <div className="border border-border-subtle rounded-xl px-4 py-3 bg-surface/40">
                  <p className="text-xs text-text-muted">{repoImportContext.summary}</p>
                  <p className="text-xs text-text-secondary mt-2">
                    Languages: {repoImportContext.detected_languages.join(", ") || "Unknown"}
                  </p>
                  <ul className="mt-2 space-y-1">
                    {repoImportContext.key_files.slice(0, 8).map((file) => (
                      <li key={file} className="text-[11px] text-text-muted font-mono truncate">
                        {file}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {activeTab === "branches" && (
            <div className="space-y-3">
              {!currentSessionId ? (
                <p className="text-sm text-text-muted">Open a session first to manage branches.</p>
              ) : (
                <>
                  <p className="text-sm text-text-secondary">
                    Capture alternate decision paths while keeping your main plan clean.
                  </p>
                  <div className="flex gap-2">
                    <input
                      value={branchName}
                      onChange={(e) => setBranchName(e.target.value)}
                      placeholder="e.g. Event-driven architecture"
                      className="flex-1 px-3 py-2 rounded-lg bg-surface border border-border-default text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-glow"
                    />
                    <button
                      onClick={() =>
                        runWithLoading(async () => {
                          const created = await createBranch(branchName.trim());
                          if (created) setBranchName("");
                        })
                      }
                      disabled={!branchName.trim() || loading}
                      className="px-3 py-2 text-xs rounded-lg border border-accent-gold/40 text-accent-gold bg-transparent hover:bg-accent-gold/10 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                    >
                      Create
                    </button>
                  </div>
                  <div className="space-y-2">
                    {branches.length === 0 ? (
                      <p className="text-xs text-text-muted">No branches yet.</p>
                    ) : (
                      branches.map((branch) => (
                        <div
                          key={branch.id}
                          className="border border-border-subtle rounded-lg px-3 py-2 bg-surface/40"
                        >
                          <p className="text-sm text-text-primary">{branch.name}</p>
                          <p className="text-[11px] text-text-muted">{branch.created_at}</p>
                        </div>
                      ))
                    )}
                  </div>
                </>
              )}
            </div>
          )}

          {activeTab === "export" && (
            <div className="space-y-3">
              <p className="text-sm text-text-secondary">
                Preview issue tickets generated from your planning documents.
              </p>
              <button
                onClick={() =>
                  runWithLoading(async () => {
                    await loadIssueExportPreview();
                  })
                }
                disabled={!currentSessionId || loading}
                className="px-3 py-2 text-xs rounded-lg border border-accent-gold/40 text-accent-gold bg-transparent hover:bg-accent-gold/10 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
              >
                Build Preview
              </button>
              <div className="space-y-2">
                {exportPreview.map((item, idx) => (
                  <div
                    key={`${item.title}-${idx}`}
                    className="border border-border-subtle rounded-lg px-3 py-2 bg-surface/40"
                  >
                    <p className="text-sm text-text-primary">{item.title}</p>
                    <p className="text-xs text-text-secondary mt-1">{item.body}</p>
                    <p className="text-[11px] text-text-muted mt-1">{item.labels.join(", ")}</p>
                  </div>
                ))}
                {exportPreview.length === 0 && (
                  <p className="text-xs text-text-muted">No preview generated yet.</p>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
