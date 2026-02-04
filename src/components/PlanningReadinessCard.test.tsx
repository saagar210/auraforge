import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PlanningReadinessCard } from "./PlanningReadinessCard";

describe("PlanningReadinessCard", () => {
  it("renders score, coverage, and recommendation", () => {
    render(
      <PlanningReadinessCard
        readiness={{
          score: 78,
          unresolved_tbd: 2,
          recommendation: "Fill remaining gaps before forge.",
          must_haves: [
            { key: "problem", label: "Problem statement", status: "covered" },
            { key: "flow", label: "Core user flow", status: "partial" },
          ],
          should_haves: [],
        }}
      />,
    );

    expect(screen.getByText(/Score 78\/100/)).toBeInTheDocument();
    expect(screen.getByText(/Must-haves covered 1\/2/)).toBeInTheDocument();
    expect(screen.getByText(/Fill remaining gaps before forge\./)).toBeInTheDocument();
  });
});
