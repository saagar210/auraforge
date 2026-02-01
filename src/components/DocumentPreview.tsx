import { useState, memo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl as openExternal } from "@tauri-apps/plugin-opener";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Copy, Check, RefreshCw, FolderDown } from "lucide-react";
import { clsx } from "clsx";
import type { GeneratedDocument } from "../types";

const TAB_ORDER = ["README.md", "SPEC.md", "CLAUDE.md", "PROMPTS.md", "CONVERSATION.md"];

const markdownComponents = {
  a({ href, children }: { href?: string; children?: React.ReactNode }) {
    const url = href ?? "";
    const isSafe = /^(https?:|mailto:)/i.test(url);
    return (
      <a
        href={url}
        className="text-accent-glow hover:text-accent-gold underline"
        rel="noreferrer"
        onClick={(e) => {
          if (!isSafe) {
            e.preventDefault();
            return;
          }
          e.preventDefault();
          void openExternal(url);
        }}
      >
        {children}
      </a>
    );
  },
  code({ className, children, ...props }: { className?: string; children?: React.ReactNode }) {
    const match = /language-(\w+)/.exec(className || "");
    const codeString = String(children).replace(/\n$/, "");

    if (match) {
      return (
        <SyntaxHighlighter
          style={oneDark}
          language={match[1]}
          PreTag="div"
          customStyle={{
            background: "#1A1517",
            border: "1px solid #3a3335",
            borderRadius: "8px",
            fontSize: "13px",
          }}
        >
          {codeString}
        </SyntaxHighlighter>
      );
    }

    return (
      <code
        className="bg-surface px-1.5 py-0.5 rounded text-accent-glow font-mono text-[13px]"
        {...props}
      >
        {children}
      </code>
    );
  },
  h1({ children }: { children?: React.ReactNode }) {
    return (
      <h1 className="font-heading text-2xl text-text-primary border-b border-border-subtle pb-3 mb-4 mt-6">
        {children}
      </h1>
    );
  },
  h2({ children }: { children?: React.ReactNode }) {
    return (
      <h2 className="font-heading text-xl text-text-primary mt-6 mb-3">
        {children}
      </h2>
    );
  },
  h3({ children }: { children?: React.ReactNode }) {
    return (
      <h3 className="font-heading text-lg text-text-primary mt-5 mb-2">
        {children}
      </h3>
    );
  },
  p({ children }: { children?: React.ReactNode }) {
    return <p className="text-text-primary leading-relaxed mb-4">{children}</p>;
  },
  ul({ children }: { children?: React.ReactNode }) {
    return (
      <ul className="text-text-primary mb-4 pl-6 list-disc space-y-1">
        {children}
      </ul>
    );
  },
  ol({ children }: { children?: React.ReactNode }) {
    return (
      <ol className="text-text-primary mb-4 pl-6 list-decimal space-y-1">
        {children}
      </ol>
    );
  },
  li({ children }: { children?: React.ReactNode }) {
    return <li className="text-text-primary">{children}</li>;
  },
  table({ children }: { children?: React.ReactNode }) {
    return (
      <div className="overflow-x-auto mb-4">
        <table className="w-full border-collapse border border-border-subtle text-sm">
          {children}
        </table>
      </div>
    );
  },
  th({ children }: { children?: React.ReactNode }) {
    return (
      <th className="border border-border-subtle bg-elevated px-3 py-2 text-left text-text-primary font-medium">
        {children}
      </th>
    );
  },
  td({ children }: { children?: React.ReactNode }) {
    return (
      <td className="border border-border-subtle px-3 py-2 text-text-primary">
        {children}
      </td>
    );
  },
  blockquote({ children }: { children?: React.ReactNode }) {
    return (
      <blockquote className="border-l-4 border-accent-gold/50 pl-4 my-4 text-text-secondary italic">
        {children}
      </blockquote>
    );
  },
  hr() {
    return <hr className="border-border-subtle my-6" />;
  },
  strong({ children }: { children?: React.ReactNode }) {
    return <strong className="text-text-primary font-semibold">{children}</strong>;
  },
};

const MarkdownContent = memo(function MarkdownContent({
  content,
}: {
  content: string;
}) {
  return (
    <ReactMarkdown remarkPlugins={[remarkGfm]} components={markdownComponents}>
      {content}
    </ReactMarkdown>
  );
});

interface DocumentPreviewProps {
  documents: GeneratedDocument[];
  stale: boolean;
  onRegenerate: () => void;
  regenerating: boolean;
  onSave: () => void;
}

export function DocumentPreview({
  documents,
  stale,
  onRegenerate,
  regenerating,
  onSave,
}: DocumentPreviewProps) {
  const [activeTab, setActiveTab] = useState("README.md");
  const [copied, setCopied] = useState(false);

  // Sort documents by TAB_ORDER
  const sortedDocs = TAB_ORDER
    .map((name) => documents.find((d) => d.filename === name))
    .filter((d): d is GeneratedDocument => d !== undefined);

  // If active tab doesn't match any doc, auto-select first available
  const effectiveTab = sortedDocs.some((d) => d.filename === activeTab)
    ? activeTab
    : sortedDocs[0]?.filename ?? activeTab;
  const activeDoc = sortedDocs.find((d) => d.filename === effectiveTab);

  const handleCopy = async () => {
    if (!activeDoc) return;
    await navigator.clipboard.writeText(activeDoc.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (sortedDocs.length === 0) return null;

  return (
    <div className="flex flex-col h-full">
      {/* Tab Bar */}
      <div
        className="flex items-center gap-1 px-4 border-b border-border-subtle bg-elevated overflow-x-auto"
        role="tablist"
        aria-label="Document tabs"
      >
        {sortedDocs.map((doc) => (
          <button
            key={doc.filename}
            role="tab"
            aria-selected={doc.filename === effectiveTab}
            onClick={() => {
              setActiveTab(doc.filename);
              setCopied(false);
            }}
            className={clsx(
              "px-4 py-3 bg-transparent border-none border-b-2 font-heading text-sm cursor-pointer transition-all duration-200 whitespace-nowrap",
              doc.filename === effectiveTab
                ? "text-accent-gold border-b-accent-gold"
                : "text-text-secondary border-b-transparent hover:text-text-primary",
            )}
            style={
              doc.filename === effectiveTab
                ? { textShadow: "0 0 20px rgba(212, 175, 55, 0.3)" }
                : undefined
            }
          >
            {doc.filename}
          </button>
        ))}

        {/* Spacer */}
        <div className="flex-1" />

        {/* Stale / Regenerate Badge */}
        {stale && (
          <button
            onClick={onRegenerate}
            disabled={regenerating}
            aria-label={regenerating ? "Regenerating documents" : "Regenerate documents"}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-status-warning bg-status-warning/10 border border-status-warning/30 rounded-full cursor-pointer hover:bg-status-warning/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RefreshCw
              className={clsx("w-3 h-3", regenerating && "animate-spin")}
              aria-hidden="true"
            />
            {regenerating ? "Regenerating..." : "Regenerate"}
          </button>
        )}

        {/* Save to Folder */}
        <button
          onClick={onSave}
          aria-label="Save documents to folder"
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-accent-gold bg-transparent border border-accent-gold/40 rounded-md cursor-pointer hover:bg-accent-gold/10 hover:border-accent-gold transition-colors"
        >
          <FolderDown className="w-3 h-3" aria-hidden="true" />
          Save to Folder
        </button>

        {/* Copy Button */}
        <button
          onClick={handleCopy}
          aria-label={copied ? "Copied to clipboard" : "Copy document content"}
          className="flex items-center gap-1.5 px-3 py-1.5 mr-2 text-xs text-text-secondary bg-transparent border border-border-subtle rounded-md cursor-pointer hover:text-text-primary hover:border-text-muted transition-colors"
        >
          {copied ? (
            <>
              <Check className="w-3 h-3 text-status-success" />
              Copied
            </>
          ) : (
            <>
              <Copy className="w-3 h-3" />
              Copy
            </>
          )}
        </button>
      </div>

      {/* Document Content */}
      <div className="flex-1 overflow-y-auto px-6 py-6 bg-void">
        <div className="max-w-[720px] mx-auto prose-auraforge">
          {activeDoc ? (
            <MarkdownContent content={activeDoc.content} />
          ) : (
            <p className="text-text-muted text-center py-8">
              Select a document tab to view
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
