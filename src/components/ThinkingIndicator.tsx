export function ThinkingIndicator() {
  return (
    <div
      className="self-start flex gap-2 px-4 py-3 animate-[message-in-left_0.3s_ease]"
      role="status"
      aria-label="AI is thinking"
    >
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          className="w-2 h-3 rounded-[2px_2px_4px_4px]"
          aria-hidden="true"
          style={{
            background:
              "linear-gradient(to top, #D4483B 0%, #FF6B35 50%, #E8A045 100%)",
            animation: `flame-flicker 0.8s ease-in-out infinite`,
            animationDelay: `${i * 0.2}s`,
          }}
        />
      ))}
    </div>
  );
}
