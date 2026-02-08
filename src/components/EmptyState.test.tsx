import "../test-utils";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EmptyState } from "./EmptyState";

describe("EmptyState", () => {
  it("renders 'Your forge awaits' when no session", () => {
    render(<EmptyState hasSession={false} onNewProject={vi.fn()} />);
    expect(screen.getByText("Your forge awaits")).toBeInTheDocument();
  });

  it("renders Begin button that fires onNewProject", () => {
    const onNewProject = vi.fn();
    render(<EmptyState hasSession={false} onNewProject={onNewProject} />);
    fireEvent.click(screen.getByText("Begin"));
    expect(onNewProject).toHaveBeenCalledTimes(1);
  });

  it("renders first-session instructions when hasSession and isFirstSession", () => {
    render(
      <EmptyState
        hasSession={true}
        isFirstSession={true}
        onNewProject={vi.fn()}
      />,
    );
    expect(screen.getByText("Here's how it works")).toBeInTheDocument();
  });

  it("renders minimal prompt when hasSession but not first", () => {
    render(<EmptyState hasSession={true} onNewProject={vi.fn()} />);
    expect(screen.getByText("Describe your vision...")).toBeInTheDocument();
  });
});
