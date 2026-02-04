import type { PlanningReadiness } from "../types";

interface PlanningReadinessCardProps {
  readiness: PlanningReadiness | null;
}

function statusDot(status: "covered" | "partial" | "missing") {
  if (status === "covered") return "bg-status-success";
  if (status === "partial") return "bg-status-warning";
  return "bg-status-error";
}

export function PlanningReadinessCard({ readiness }: PlanningReadinessCardProps) {
  if (!readiness) return null;

  const coveredMustHaves = readiness.must_haves.filter((i) => i.status === "covered").length;

  return (
    <div className="w-full border border-border-subtle bg-elevated rounded-xl px-4 py-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-xs uppercase tracking-wider text-text-muted">Planning Readiness</p>
          <p className="text-sm text-text-primary">
            Score {readiness.score}/100 · Must-haves covered {coveredMustHaves}/
            {readiness.must_haves.length}
          </p>
        </div>
        <div className="text-[11px] text-text-secondary">{readiness.unresolved_tbd} TBDs</div>
      </div>
      <div className="mt-2 flex flex-wrap gap-2">
        {readiness.must_haves.map((item) => (
          <span
            key={item.key}
            className="inline-flex items-center gap-1.5 text-[11px] px-2 py-1 rounded-full bg-surface text-text-secondary"
          >
            <span className={`w-1.5 h-1.5 rounded-full ${statusDot(item.status)}`} />
            {item.label}
          </span>
        ))}
      </div>
      <p className="mt-2 text-xs text-text-muted">{readiness.recommendation}</p>
    </div>
  );
}
