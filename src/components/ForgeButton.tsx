import { Hammer } from "lucide-react";

interface ForgeButtonProps {
  onClick: () => void;
  disabled: boolean;
  generating: boolean;
}

export function ForgeButton({ onClick, disabled, generating }: ForgeButtonProps) {
  return (
    <div className="flex justify-center py-4">
      <button
        onClick={onClick}
        disabled={disabled || generating}
        aria-label={generating ? "Forging documents" : "Forge the Plan"}
        className="group relative overflow-hidden px-8 py-4 bg-transparent border-2 border-accent-gold rounded-lg font-heading text-lg font-medium text-accent-gold tracking-wider cursor-pointer transition-all duration-300 hover:shadow-glow-intense hover:-translate-y-0.5 active:translate-y-0 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none disabled:shadow-none"
      >
        {/* Molten border glow on hover */}
        <span
          className="absolute pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-300"
          style={{
            inset: "-2px",
            background:
              "linear-gradient(90deg, #D4483B 0%, #FF6B35 25%, #E8A045 50%, #FF6B35 75%, #D4483B 100%)",
            backgroundSize: "200% 100%",
            borderRadius: "calc(0.5rem + 2px)",
            zIndex: -1,
            animation: "molten-flow 3s linear infinite",
          }}
        />

        {/* Inner glow on hover */}
        <span className="absolute inset-0 bg-gradient-to-t from-accent-gold/10 via-accent-gold/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />

        {generating ? (
          <span className="relative flex items-center gap-2">
            <span className="w-5 h-5 border-2 border-accent-gold/30 border-t-accent-gold rounded-full animate-spin" />
            <span>Forging...</span>
          </span>
        ) : (
          <span className="relative flex items-center gap-2">
            <Hammer className="w-5 h-5" />
            <span>Forge the Plan</span>
          </span>
        )}
      </button>
    </div>
  );
}
