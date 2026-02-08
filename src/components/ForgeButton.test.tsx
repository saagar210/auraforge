import "../test-utils";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ForgeButton } from "./ForgeButton";

describe("ForgeButton", () => {
  const defaults = {
    onClick: vi.fn(),
    disabled: false,
    generating: false,
    target: "generic" as const,
    onTargetChange: vi.fn(),
  };

  it("renders with 'Forge the Plan' label", () => {
    render(<ForgeButton {...defaults} />);
    expect(screen.getByText("Forge the Plan")).toBeInTheDocument();
  });

  it("fires onClick when clicked", () => {
    const onClick = vi.fn();
    render(<ForgeButton {...defaults} onClick={onClick} />);
    fireEvent.click(screen.getByLabelText("Forge the Plan"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("disables when disabled prop is true", () => {
    render(<ForgeButton {...defaults} disabled={true} />);
    expect(screen.getByLabelText("Forge the Plan")).toBeDisabled();
  });

  it("shows Forging... when generating", () => {
    render(<ForgeButton {...defaults} generating={true} />);
    expect(screen.getByText("Forging...")).toBeInTheDocument();
  });
});
