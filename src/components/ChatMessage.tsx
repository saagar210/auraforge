import { memo, type ReactNode } from "react";
import { clsx } from "clsx";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { openUrl as openExternal } from "@tauri-apps/plugin-opener";
import type { Message } from "../types";

interface ChatMessageProps {
  message: Message;
}

const markdownComponents = {
  a({ href, children }: { href?: string; children?: ReactNode }) {
    const url = href ?? "";
    const isSafe = /^(https?:|mailto:)/i.test(url);
    return (
      <a
        href={url}
        className="text-accent-glow underline underline-offset-2"
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
};

export const ChatMessage = memo(
  function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";
  const sources = !isUser ? message.metadata?.search_results ?? [] : [];
  const freshness = !isUser ? message.metadata?.search_timestamp : undefined;
  const freshnessLabel = freshness
    ? new Date(freshness).toLocaleString([], {
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
      })
    : null;

  return (
    <div
      className={clsx(
        "max-w-[85%] px-4 py-3 shadow-sm",
        isUser
          ? "self-end bg-surface rounded-xl rounded-br-sm animate-[message-in-right_0.3s_ease]"
          : "self-start bg-warm rounded-xl rounded-bl-sm border-l-2 border-l-accent-glow animate-[message-in-left_0.3s_ease]",
        !isUser && "relative",
      )}
      style={
        !isUser
          ? {
              boxShadow:
                "0 2px 4px rgba(0,0,0,0.3), -4px 0 20px rgba(232,160,69,0.15)",
            }
          : undefined
      }
    >
      {/* Glow dot for AI messages */}
      {!isUser && (
        <div
          className="absolute -left-3 top-3 w-2 h-2 bg-accent-glow rounded-full"
          style={{ boxShadow: "0 0 10px #E8A045" }}
        />
      )}
      <div className="text-sm text-text-primary leading-relaxed whitespace-pre-wrap break-words">
        {isUser ? (
          message.content
        ) : (
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={markdownComponents}
          >
            {message.content}
          </ReactMarkdown>
        )}
      </div>
      {!isUser && sources.length > 0 && (
        <div className="mt-3 border-t border-border-subtle pt-2">
          <div className="flex items-center justify-between gap-2 mb-1">
            <span className="text-[11px] text-text-muted">Sources ({sources.length})</span>
            {freshnessLabel && (
              <span className="text-[11px] text-text-muted">Fresh as of {freshnessLabel}</span>
            )}
          </div>
          <ul className="space-y-1">
            {sources.slice(0, 3).map((result, idx) => (
              <li key={`${result.url}-${idx}`} className="text-xs text-text-secondary truncate">
                <a
                  href={result.url}
                  className="text-accent-glow underline underline-offset-2"
                  onClick={(e) => {
                    e.preventDefault();
                    void openExternal(result.url);
                  }}
                >
                  {result.title || result.url}
                </a>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
  },
  (prev, next) =>
    prev.message.id === next.message.id &&
    prev.message.content === next.message.content &&
    prev.message.role === next.message.role &&
    prev.message.created_at === next.message.created_at &&
    prev.message.metadata === next.message.metadata,
);

interface StreamingMessageProps {
  content: string;
}

export function StreamingMessage({ content }: StreamingMessageProps) {
  return (
    <div
      className="self-start max-w-[85%] px-4 py-3 bg-warm rounded-xl rounded-bl-sm border-l-2 border-l-accent-glow relative animate-[message-in-left_0.3s_ease]"
      style={{
        boxShadow:
          "0 2px 4px rgba(0,0,0,0.3), -4px 0 20px rgba(232,160,69,0.15)",
      }}
    >
      <div
        className="absolute -left-3 top-3 w-2 h-2 bg-accent-glow rounded-full"
        style={{ boxShadow: "0 0 10px #E8A045" }}
      />
      <div className="text-sm text-text-primary leading-relaxed whitespace-pre-wrap break-words">
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={markdownComponents}
        >
          {content}
        </ReactMarkdown>
        <span className="inline-block w-0.5 h-4 bg-accent-glow ml-0.5 align-middle animate-pulse" />
      </div>
    </div>
  );
}
