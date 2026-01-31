import { useMemo } from "react";

interface Ember {
  id: number;
  x: number;
  size: number;
  duration: number;
  delay: number;
}

export function EmberParticles() {
  const embers = useMemo<Ember[]>(
    () =>
      Array.from({ length: 14 }, (_, i) => ({
        id: i,
        x: Math.random() * 100,
        size: 2 + Math.random() * 3,
        duration: 8 + Math.random() * 6,
        delay: Math.random() * 8,
      })),
    [],
  );

  return (
    <div
      className="ember-container fixed inset-0 pointer-events-none overflow-hidden"
      style={{ zIndex: 0 }}
      aria-hidden="true"
    >
      {embers.map((ember) => (
        <div
          key={ember.id}
          className="absolute bottom-[-20px] rounded-full opacity-0"
          style={{
            left: `${ember.x}%`,
            width: `${ember.size}px`,
            height: `${ember.size}px`,
            background:
              "radial-gradient(circle at center, #E8A045 0%, #FF6B35 40%, transparent 70%)",
            animation: `ember-rise ${ember.duration}s ease-in-out infinite`,
            animationDelay: `${ember.delay}s`,
          }}
        />
      ))}
    </div>
  );
}
