import { ReactNode } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { vi } from 'vitest';

// Core Tauri mocks
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(), emit: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
  openPath: vi.fn(),
}));

// Mock Tauri API for E2E testing
export const mockTauriAPI = {
  commands: {
    sendMessage: vi.fn(),
    generateDocuments: vi.fn(),
    saveToFolder: vi.fn(),
    getConfig: vi.fn(),
    updateConfig: vi.fn(),
    createSession: vi.fn(),
    renameSession: vi.fn(),
    deleteSession: vi.fn(),
    loadSessions: vi.fn(),
    cancelResponse: vi.fn(),
    retryLastMessage: vi.fn(),
    loadMessages: vi.fn(),
    analyzePlanReadiness: vi.fn(),
  },
  events: {
    listen: vi.fn(),
    emit: vi.fn(),
  },
};

// Custom render function with mocks
export function renderWithMocks(
  ui: ReactNode,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, options);
}

// Helper: Setup mock event listener with immediate callback
export function setupEventListenerMock(
  eventName: string,
  payload: unknown,
  delayMs: number = 0
) {
  const listenMock = vi.fn((name: string, callback: Function) => {
    if (name === eventName) {
      if (delayMs > 0) {
        setTimeout(() => callback({ payload }), delayMs);
      } else {
        callback({ payload });
      }
      return Promise.resolve(() => {});
    }
    return Promise.resolve(() => {});
  });

  return listenMock;
}

// Helper: Setup streaming message chunks
export function setupStreamingMessageMock(tokens: string[], delayPerToken: number = 10) {
  const chunks = tokens.map((token, idx) => ({
    token,
    done: idx === tokens.length - 1,
    tokens_total: tokens.length,
  }));

  mockTauriAPI.commands.sendMessage.mockImplementation(async () => {
    for (const chunk of chunks) {
      await new Promise(resolve => setTimeout(resolve, delayPerToken));
      // Emit via listener (simulating Tauri event)
      const listeners = mockTauriAPI.events.listen.mock.calls
        .filter((call: any) => call[0] === 'message_chunk')
        .map((call: any) => call[1]);
      listeners.forEach((listener: Function) => listener({ payload: chunk }));
    }
  });
}

// Helper: Wait for async actions
export async function waitForAsync() {
  return new Promise(resolve => setTimeout(resolve, 0));
}

// Helper: Reset all mocks
export function resetMocks() {
  Object.values(mockTauriAPI.commands).forEach(mock => mock.mockClear());
  Object.values(mockTauriAPI.events).forEach(mock => mock.mockClear());
}
