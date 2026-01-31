interface ForgingProgressProps {
  current: number;
  total: number;
  filename: string;
}

export function ForgingProgress({ current, total, filename }: ForgingProgressProps) {
  const progress = current / total;
  const circumference = 2 * Math.PI * 36;
  const dashoffset = circumference * (1 - progress);

  return (
    <div
      className="flex flex-col items-center py-8 animate-[message-in-left_0.3s_ease]"
      role="status"
      aria-label={`Generating document ${current} of ${total}: ${filename}`}
    >
      {/* Progress Ring */}
      <div className="relative w-20 h-20 mb-4">
        <svg className="w-full h-full -rotate-90" viewBox="0 0 80 80">
          <circle
            cx="40"
            cy="40"
            r="36"
            fill="none"
            strokeWidth="4"
            className="stroke-border-subtle"
          />
          <circle
            cx="40"
            cy="40"
            r="36"
            fill="none"
            strokeWidth="4"
            strokeLinecap="round"
            className="stroke-accent-gold transition-all duration-500 ease-out"
            style={{
              strokeDasharray: circumference,
              strokeDashoffset: dashoffset,
              filter: "drop-shadow(0 0 6px rgba(212, 175, 55, 0.6))",
            }}
          />
        </svg>
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-sm font-heading text-accent-glow font-medium">
            {current}/{total}
          </span>
        </div>
      </div>

      {/* Status Text */}
      <p
        className="font-heading text-base text-accent-glow mb-1"
        style={{ animation: "status-glow 2s ease-in-out infinite" }}
      >
        Forging Documents
      </p>
      <p className="text-sm text-text-secondary">
        Generating {filename}...
      </p>
    </div>
  );
}
