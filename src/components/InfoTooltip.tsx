import { Info } from "lucide-react";

interface InfoTooltipProps {
  text: string;
}

export function InfoTooltip({ text }: InfoTooltipProps) {
  return (
    <span className="relative inline-flex items-center group">
      <Info className="w-3.5 h-3.5 text-text-muted cursor-help" />
      <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1.5 px-2.5 py-1.5 bg-surface border border-border-default rounded-lg text-xs text-text-secondary whitespace-nowrap opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity duration-200 shadow-md z-50 max-w-[240px] whitespace-normal text-center">
        {text}
      </span>
    </span>
  );
}
