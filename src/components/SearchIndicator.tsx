import { Search } from "lucide-react";

interface SearchIndicatorProps {
  query: string;
}

export function SearchIndicator({ query }: SearchIndicatorProps) {
  return (
    <div
      className="self-start animate-[message-in-left_0.3s_ease]"
      role="status"
      aria-label={`Searching the web for: ${query}`}
    >
      <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-surface rounded-full border border-border-subtle">
        <Search
          className="w-3.5 h-3.5 text-accent-glow animate-pulse"
          style={{
            filter: "drop-shadow(0 0 4px rgba(232, 160, 69, 0.3))",
          }}
          aria-hidden="true"
        />
        <span className="text-xs text-text-secondary">
          Searching: {query}...
        </span>
      </div>
    </div>
  );
}
