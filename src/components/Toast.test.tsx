import "../test-utils";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Toast } from "./Toast";

describe("Toast", () => {
  it("renders message text", () => {
    render(
      <Toast message="Saved successfully" type="success" onDismiss={vi.fn()} />,
    );
    expect(screen.getByText("Saved successfully")).toBeInTheDocument();
  });

  it("fires onDismiss when dismiss button is clicked", () => {
    const onDismiss = vi.fn();
    render(
      <Toast message="Error occurred" type="error" onDismiss={onDismiss} />,
    );
    fireEvent.click(screen.getByLabelText("Dismiss notification"));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it("renders action button when provided", () => {
    const onClick = vi.fn();
    render(
      <Toast
        message="Exported"
        type="success"
        action={{ label: "Open Folder", onClick }}
        onDismiss={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByText("Open Folder"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});
