export function ThermalBackground() {
  return (
    <div
      className="thermal-background fixed inset-0 pointer-events-none bg-void"
      style={{ zIndex: 0 }}
      aria-hidden="true"
    >
      <div
        className="absolute opacity-50"
        style={{
          width: "40%",
          height: "40%",
          top: "20%",
          left: "10%",
          background:
            "radial-gradient(ellipse at center, rgba(26, 10, 10, 0.6) 0%, transparent 70%)",
          animation: "thermal-drift 20s ease-in-out infinite",
          animationDelay: "0s",
        }}
      />
      <div
        className="absolute opacity-50"
        style={{
          width: "40%",
          height: "40%",
          top: "50%",
          right: "10%",
          background:
            "radial-gradient(ellipse at center, rgba(26, 10, 10, 0.6) 0%, transparent 70%)",
          animation: "thermal-drift 20s ease-in-out infinite",
          animationDelay: "-7s",
        }}
      />
      <div
        className="absolute opacity-50"
        style={{
          width: "40%",
          height: "40%",
          bottom: "10%",
          left: "30%",
          background:
            "radial-gradient(ellipse at center, rgba(26, 10, 10, 0.6) 0%, transparent 70%)",
          animation: "thermal-drift 20s ease-in-out infinite",
          animationDelay: "-14s",
        }}
      />
    </div>
  );
}
