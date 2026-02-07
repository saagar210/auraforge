import { Flame, ArrowRight } from "lucide-react";
import type { PlanningTemplate } from "../types";

interface EmptyStateProps {
  hasSession: boolean;
  onNewProject: () => void;
  isFirstSession?: boolean;
  onSuggestionClick?: (text: string) => void;
  templates?: PlanningTemplate[];
  onTemplateSelect?: (templateId: string) => void;
}

const SUGGESTIONS = [
  "A SaaS web app",
  "A mobile app",
  "A CLI tool",
];

export function EmptyState({
  hasSession,
  onNewProject,
  isFirstSession,
  onSuggestionClick,
  templates = [],
  onTemplateSelect,
}: EmptyStateProps) {
  if (hasSession && isFirstSession) {
    return (
      <div className="flex flex-col items-center justify-center flex-1 px-10 text-center">
        <Flame
          className="w-12 h-12 text-accent-glow/60 mb-4"
          style={{ filter: "drop-shadow(0 0 15px rgba(232,160,69,0.3))" }}
        />
        <h2 className="text-lg font-heading font-semibold text-text-primary mb-3">
          Here's how it works
        </h2>

        <div className="bg-surface rounded-xl px-5 py-4 mb-5 max-w-sm text-left space-y-2">
          {[
            ["1", "Tell your idea", "Describe what you want to build"],
            ["2", "Answer questions", "I'll help you think through details"],
            ["3", "Refine together", "We'll shape the plan through conversation"],
            ["4", "Generate docs", "Get README, SPEC, and more to start coding"],
          ].map(([num, title, desc]) => (
            <div key={num} className="flex items-start gap-3">
              <span className="text-xs font-mono text-accent-gold mt-0.5 shrink-0 w-4 text-center">
                {num}
              </span>
              <div>
                <span className="text-sm text-text-primary font-medium">
                  {title}
                </span>
                <span className="text-xs text-text-muted block">{desc}</span>
              </div>
            </div>
          ))}
        </div>

        <p className="text-sm text-text-secondary mb-3">
          What are you thinking about building?
        </p>

        <div className="flex flex-wrap gap-2 justify-center">
          {SUGGESTIONS.map((s) => (
            <button
              key={s}
              onClick={() => onSuggestionClick?.(s)}
              className="px-3 py-1.5 bg-surface border border-border-default rounded-full text-xs text-text-secondary hover:text-accent-gold hover:border-accent-gold/40 transition-colors cursor-pointer flex items-center gap-1.5"
            >
              {s}
              <ArrowRight className="w-3 h-3" />
            </button>
          ))}
        </div>

        {templates.length > 0 && (
          <div className="mt-5 w-full max-w-lg">
            <p className="text-xs text-text-muted mb-2">Or start from a template</p>
            <div className="flex flex-wrap gap-2 justify-center">
              {templates.map((template) => (
                <button
                  key={template.id}
                  onClick={() => onTemplateSelect?.(template.id)}
                  className="px-3 py-1.5 bg-surface border border-border-default rounded-full text-xs text-text-secondary hover:text-accent-gold hover:border-accent-gold/40 transition-colors cursor-pointer"
                  title={template.description}
                >
                  {template.name}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  }

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
      {templates.length > 0 && (
        <div className="mt-4">
          <p className="text-xs text-text-muted mb-2">Start from a template</p>
          <div className="flex flex-wrap gap-2 justify-center">
            {templates.map((template) => (
              <button
                key={template.id}
                onClick={() => onTemplateSelect?.(template.id)}
                className="px-3 py-1.5 bg-surface border border-border-default rounded-full text-xs text-text-secondary hover:text-accent-gold hover:border-accent-gold/40 transition-colors cursor-pointer"
                title={template.description}
              >
                {template.name}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
