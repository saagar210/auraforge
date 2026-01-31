import { useEffect } from "react";
import { CheckCircle, AlertCircle, X, FolderOpen } from "lucide-react";

interface ToastProps {
  message: string;
  type: "success" | "error";
  action?: {
    label: string;
    onClick: () => void;
  };
  onDismiss: () => void;
}

export function Toast({ message, type, action, onDismiss }: ToastProps) {
  useEffect(() => {
    const timer = setTimeout(onDismiss, 5000);
    return () => clearTimeout(timer);
  }, [onDismiss]);

  const Icon = type === "success" ? CheckCircle : AlertCircle;
  const borderClass =
    type === "success"
      ? "border-l-[3px] border-l-status-success"
      : "border-l-[3px] border-l-status-error";
  const iconClass =
    type === "success" ? "text-status-success" : "text-status-error";

  return (
    <div
      className="fixed bottom-6 right-6 z-50 animate-[toast-in_0.3s_ease]"
      role="alert"
      aria-live="polite"
    >
      <div
        className={`flex items-center gap-3 px-4 py-3 rounded-lg bg-elevated border border-border-default shadow-md max-w-[400px] ${borderClass}`}
      >
        <Icon className={`w-4 h-4 shrink-0 ${iconClass}`} />
        <p className="text-sm text-text-primary flex-1">{message}</p>

        {action && (
          <button
            onClick={action.onClick}
            className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium text-accent-gold bg-accent-gold/10 border border-accent-gold/30 rounded-md cursor-pointer hover:bg-accent-gold/20 transition-colors whitespace-nowrap"
          >
            <FolderOpen className="w-3 h-3" />
            {action.label}
          </button>
        )}

        <button
          onClick={onDismiss}
          aria-label="Dismiss notification"
          className="text-text-muted hover:text-text-primary cursor-pointer bg-transparent border-none ml-1"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
