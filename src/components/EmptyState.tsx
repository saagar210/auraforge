import { Flame } from "lucide-react";

interface EmptyStateProps {
  hasSession: boolean;
  onNewProject: () => void;
}

export function EmptyState({ hasSession, onNewProject }: EmptyStateProps) {
  if (hasSession) {
    return (
      <div className="flex flex-col items-center justify-center flex-1 px-10 text-center">
        <Flame
          className="w-12 h-12 text-accent-glow/60 mb-4"
          style={{ filter: "drop-shadow(0 0 15px rgba(232,160,69,0.3))" }}
        />
        <p className="text-text-secondary text-base">
          Describe your vision...
        </p>
        <p className="text-text-muted text-xs mt-1">
          What would you like to build?
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center flex-1 px-10 text-center">
      <Flame
        className="w-20 h-20 text-accent-glow mb-6"
        style={{ filter: "drop-shadow(0 0 20px rgba(232,160,69,0.5))" }}
      />
      <h1
        className="text-3xl font-heading font-semibold mb-2 tracking-wide"
        style={{ textShadow: "0 0 40px rgba(232,160,69,0.2)" }}
      >
        Your forge awaits
      </h1>
      <p className="text-text-secondary text-base mb-6 max-w-sm">
        What will you create?
      </p>
      <button
        onClick={onNewProject}
        className="px-8 py-3 border-2 border-accent-gold rounded-xl text-accent-gold font-heading text-lg font-medium cursor-pointer transition-all duration-300 hover:bg-accent-gold/10 hover:shadow-glow-intense hover:-translate-y-0.5 active:translate-y-0 bg-transparent"
      >
        Begin
      </button>
    </div>
  );
}
