import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { GeneratedDocument, StreamChunk } from "../types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: vi.fn(),
}));

import { useChatStore } from "./chatStore";

type EventPayload = {
  payload: StreamChunk;
};

type EventHandler = (event: EventPayload) => void;

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const listeners = new Map<string, EventHandler>();

function mockListenHandlers() {
  listenMock.mockImplementation(async (event, handler) => {
    listeners.set(String(event), handler as EventHandler);
    return () => {
      listeners.delete(String(event));
    };
  });
}

function resetStoreState() {
  const state = useChatStore.getState();
  state.cleanupEventListeners();
  useChatStore.setState({
    currentSessionId: null,
    messages: [],
    documents: [],
    isStreaming: false,
    streamingContent: "",
    streamError: null,
    searchQuery: null,
    searchResults: null,
    isGenerating: false,
    generateProgress: null,
    generationMetadata: null,
    generationConfidence: null,
    planningCoverage: null,
    latestImportSummary: null,
    _generatingSessionId: null,
  });
}

function makeDoc(filename: string, content: string): GeneratedDocument {
  return {
    id: `${filename}-id`,
    session_id: "session-a",
    filename,
    content,
    created_at: "2026-01-01 00:00:00",
  };
}

describe("chatStore async race safety", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    listeners.clear();
    mockListenHandlers();
    resetStoreState();
  });

  afterEach(() => {
    resetStoreState();
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  it("does not apply loaded documents after session switches", async () => {
    let resolveDocs!: (docs: GeneratedDocument[]) => void;
    const docsPromise = new Promise<GeneratedDocument[]>((resolve) => {
      resolveDocs = resolve;
    });

    invokeMock.mockImplementation(async (command: string) => {
      switch (command) {
        case "get_documents":
          return docsPromise;
        case "get_generation_metadata":
          return null;
        case "get_generation_confidence":
          return null;
        default:
          throw new Error(`Unexpected invoke command in test: ${command}`);
      }
    });

    useChatStore.setState({
      currentSessionId: "session-a",
      documents: [],
    });

    const loadPromise = useChatStore.getState().loadDocuments();
    useChatStore.setState({ currentSessionId: "session-b" });

    resolveDocs([makeDoc("README.md", "doc content")]);
    await loadPromise;

    const state = useChatStore.getState();
    expect(state.currentSessionId).toBe("session-b");
    expect(state.documents).toEqual([]);

    const calledCommands = invokeMock.mock.calls.map(([command]) => command);
    expect(calledCommands).toEqual([
      "get_documents",
      "get_generation_metadata",
      "get_generation_confidence",
    ]);
  });

  it("does not clear a new session stream when old cancel timeout fires", async () => {
    invokeMock.mockResolvedValue(undefined);

    useChatStore.setState({
      currentSessionId: "session-a",
      isStreaming: true,
      streamingContent: "working",
    });

    await useChatStore.getState().cancelResponse();

    useChatStore.setState({
      currentSessionId: "session-b",
      isStreaming: true,
      streamingContent: "new-session-stream",
    });

    vi.advanceTimersByTime(2100);

    const state = useChatStore.getState();
    expect(state.currentSessionId).toBe("session-b");
    expect(state.isStreaming).toBe(true);
    expect(state.streamingContent).toBe("new-session-stream");
    expect(invokeMock).toHaveBeenCalledWith("cancel_response", {
      session_id: "session-a",
    });
  });

  it("clears cancel safety timeout on stream done", async () => {
    invokeMock.mockResolvedValue(undefined);

    useChatStore.setState({
      currentSessionId: "session-a",
      isStreaming: true,
    });

    await useChatStore.getState().initEventListeners();
    await useChatStore.getState().cancelResponse();

    const doneHandler = listeners.get("stream:done");
    expect(doneHandler).toBeDefined();
    doneHandler?.({
      payload: {
        type: "done",
        session_id: "session-a",
      },
    });

    // Simulate a new stream after completion.
    useChatStore.setState({
      currentSessionId: "session-a",
      isStreaming: true,
      streamingContent: "fresh-stream",
    });

    vi.advanceTimersByTime(2100);

    const state = useChatStore.getState();
    expect(state.isStreaming).toBe(true);
    expect(state.streamingContent).toBe("fresh-stream");
  });
});
