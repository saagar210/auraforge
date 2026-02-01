import { useRef, useEffect, useCallback } from "react";
import { ArrowUp, X } from "lucide-react";
import { clsx } from "clsx";

interface ChatInputProps {
  onSend: (content: string) => void;
  disabled: boolean;
  isStreaming: boolean;
  onCancel?: () => void;
  value: string;
  onChange: (value: string) => void;
}

export function ChatInput({
  onSend,
  disabled,
  isStreaming,
  onCancel,
  value,
  onChange,
}: ChatInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const hasContent = value.trim().length > 0;

  const adjustHeight = useCallback(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 200) + "px";
  }, []);

  useEffect(() => {
    adjustHeight();
  }, [value, adjustHeight]);

  // Auto-focus on mount
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleSend = () => {
    if (!hasContent || disabled) return;
    onSend(value.trim());
    onChange("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && e.metaKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="px-6 py-4 bg-elevated border-t border-border-subtle">
      <div className="flex items-end gap-3 max-w-[720px] mx-auto">
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          placeholder="Describe your project idea..."
          aria-label="Message input"
          rows={1}
          className="flex-1 min-h-[44px] max-h-[200px] py-3 px-4 bg-surface border border-border-default rounded-xl text-text-primary text-sm font-body resize-none transition-all duration-200 placeholder:text-text-muted focus:outline-none focus:border-accent-glow focus:shadow-[0_0_0_3px_rgba(232,160,69,0.15)] disabled:opacity-50"
        />
        {isStreaming && onCancel ? (
          <button
            onClick={onCancel}
            aria-label="Cancel response"
            className="w-11 h-11 shrink-0 rounded-full border flex items-center justify-center transition-all duration-200 cursor-pointer bg-surface border-border-default text-text-muted hover:text-text-primary hover:border-accent-glow"
          >
            <X className="w-5 h-5" />
          </button>
        ) : (
          <button
            onClick={handleSend}
            disabled={!hasContent || disabled}
            aria-label="Send message"
            className={clsx(
              "w-11 h-11 shrink-0 rounded-full border flex items-center justify-center transition-all duration-200 cursor-pointer",
              hasContent && !disabled
                ? "bg-accent-primary border-accent-primary text-text-inverse hover:bg-accent-glow hover:scale-105"
                : "bg-surface border-border-default text-text-muted opacity-50 cursor-not-allowed",
            )}
            style={
              hasContent && !disabled
                ? { animation: "pulse-glow 2s ease-in-out infinite" }
                : undefined
            }
          >
            <ArrowUp className="w-5 h-5" />
          </button>
        )}
      </div>
      <div className="max-w-[720px] mx-auto mt-1.5">
        <span className="text-[11px] text-text-muted">
          {disabled ? "Waiting for response..." : "\u2318 + Enter to send"}
        </span>
      </div>
    </div>
  );
}
