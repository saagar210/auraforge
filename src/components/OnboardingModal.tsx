import { useEffect, useRef } from "react";
import { CheckCircle, XCircle, RefreshCw, ExternalLink } from "lucide-react";
import type { HealthStatus } from "../types";

interface OnboardingModalProps {
  health: HealthStatus;
  onCheckAgain: () => void;
  onContinue: () => void;
  checking: boolean;
}

export function OnboardingModal({
  health,
  onCheckAgain,
  onContinue,
  checking,
}: OnboardingModalProps) {
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Auto-poll health every 3 seconds
  useEffect(() => {
    intervalRef.current = setInterval(onCheckAgain, 3000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [onCheckAgain]);

  const canContinue = health.ollama_connected && health.ollama_model_available;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center animate-[fade-in_0.2s_ease]"
      style={{
        background: "rgba(0, 0, 0, 0.8)",
        backdropFilter: "blur(8px)",
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Setup required"
    >
      <div
        className="bg-elevated border border-border-default rounded-2xl w-full max-w-md mx-4 shadow-lg overflow-hidden animate-[modal-in_0.3s_ease]"
      >
        {/* Header */}
        <div className="px-6 pt-6 pb-4">
          <h2 className="text-xl font-heading font-semibold text-text-primary">
            Welcome to AuraForge
          </h2>
          <p className="text-sm text-text-secondary mt-1">
            Let's make sure everything is set up.
          </p>
        </div>

        {/* Status Checks */}
        <div className="px-6 space-y-4">
          {/* Ollama Connection */}
          <StatusRow
            ok={health.ollama_connected}
            label="Ollama"
            okText="Connected"
            failText="Not detected"
          >
            {!health.ollama_connected && (
              <div className="mt-2 text-xs text-text-muted space-y-1">
                <p>Ollama needs to be running locally.</p>
                <a
                  href="https://ollama.com"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 text-accent-glow hover:text-accent-gold underline"
                >
                  Install Ollama
                  <ExternalLink className="w-3 h-3" />
                </a>
              </div>
            )}
          </StatusRow>

          {/* Model Available */}
          <StatusRow
            ok={health.ollama_model_available}
            label="Model"
            okText="Available"
            failText="Not found"
          >
            {health.ollama_connected && !health.ollama_model_available && (
              <div className="mt-2 text-xs text-text-muted space-y-1.5">
                <p>Pull the default model to get started:</p>
                <code className="block bg-surface px-3 py-2 rounded-lg text-accent-glow font-mono text-[12px] select-all">
                  ollama pull qwen3-coder:30b
                </code>
              </div>
            )}
          </StatusRow>

          {/* Database */}
          <StatusRow
            ok={health.database_ok}
            label="Database"
            okText="Ready"
            failText="Error"
          />
        </div>

        {/* Footer */}
        <div className="px-6 py-5 mt-4 border-t border-border-subtle flex items-center justify-between">
          <button
            onClick={onCheckAgain}
            disabled={checking}
            className="flex items-center gap-1.5 px-3 py-2 text-xs text-text-secondary bg-transparent border border-border-default rounded-lg hover:text-text-primary hover:border-text-muted transition-colors cursor-pointer disabled:opacity-50"
          >
            <RefreshCw
              className={`w-3 h-3 ${checking ? "animate-spin" : ""}`}
              aria-hidden="true"
            />
            Check Again
          </button>

          <button
            onClick={onContinue}
            disabled={!canContinue}
            className="px-5 py-2 bg-accent-gold text-void text-sm font-medium rounded-lg hover:bg-accent-gold/90 transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed border-none"
          >
            Continue
          </button>
        </div>
      </div>
    </div>
  );
}

function StatusRow({
  ok,
  label,
  okText,
  failText,
  children,
}: {
  ok: boolean;
  label: string;
  okText: string;
  failText: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="bg-surface rounded-lg px-4 py-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-text-primary">{label}</span>
        <span
          className={`flex items-center gap-1.5 text-xs font-medium ${
            ok ? "text-status-success" : "text-status-error"
          }`}
        >
          {ok ? (
            <CheckCircle className="w-3.5 h-3.5" aria-hidden="true" />
          ) : (
            <XCircle className="w-3.5 h-3.5" aria-hidden="true" />
          )}
          {ok ? okText : failText}
        </span>
      </div>
      {children}
    </div>
  );
}
