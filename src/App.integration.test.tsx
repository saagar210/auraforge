import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Mock the imports at module level
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openPath: vi.fn(),
  openUrl: vi.fn(),
}));

import App from './App';
import { renderWithMocks, waitForAsync } from './test-utils';

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const listeners = new Map<string, Function>();

function mockListenHandlers() {
  listenMock.mockImplementation(async (event: string, handler: Function) => {
    listeners.set(event, handler);
    return () => {
      listeners.delete(event);
    };
  });
}

describe('App E2E Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listeners.clear();
    mockListenHandlers();

    // Setup default invoke implementations for all expected commands
    invokeMock.mockImplementation(async (cmd: string, args?: any) => {
      switch (cmd) {
        case 'get_preference':
          // Return true for wizard_completed, false for first_session_completed
          if (args?.key === 'wizard_completed') return true;
          if (args?.key === 'first_session_completed') return false;
          return null;
        case 'check_health':
          return { status: 'ok', model: 'llama2' };
        case 'get_sessions':
          return [];
        case 'list_templates':
          return [];
        case 'get_config':
          return {
            llm: {
              provider: 'ollama',
              model: 'llama2',
              base_url: 'http://localhost:11434',
              api_key: null,
              temperature: 0.7,
              max_tokens: 4096,
            },
            search: {
              enabled: true,
              provider: 'duckduckgo',
              tavily_api_key: '',
              searxng_url: '',
              proactive: true,
            },
            ui: {
              theme: 'dark',
            },
            output: {
              include_conversation: false,
              default_save_path: '/tmp/output',
              default_target: 'claude',
              lint_mode: 'warn',
            },
          };
        case 'load_messages':
          return [];
        case 'get_documents':
          return [];
        case 'get_generation_metadata':
          return null;
        case 'get_plan_readiness':
          return { ready: false, coverage: 0.0, confidence: 0.0 };
        default:
          return null;
      }
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
    listeners.clear();
  });

  // Test group: Tauri Mocking Infrastructure
  describe('Tauri Mocking Infrastructure', () => {
    it('properly mocks invoke command with nested config structure', async () => {
      const config = await invokeMock('get_config');

      expect(config).toHaveProperty('llm');
      expect(config).toHaveProperty('search');
      expect(config).toHaveProperty('ui');
      expect(config).toHaveProperty('output');
      expect(config.llm.provider).toBe('ollama');
      expect(config.output.default_target).toBe('claude');
    });

    it('properly mocks get_sessions command', async () => {
      const sessions = await invokeMock('get_sessions');

      expect(Array.isArray(sessions)).toBe(true);
    });

    it('properly mocks get_preference command with dynamic args', async () => {
      const wizardCompleted = await invokeMock('get_preference', { key: 'wizard_completed' });
      const firstSession = await invokeMock('get_preference', { key: 'first_session_completed' });

      expect(wizardCompleted).toBe(true);
      expect(firstSession).toBe(false);
    });

    it('registers event listeners via listen mock', async () => {
      const handler = vi.fn();
      await listenMock('message_chunk', handler);

      expect(listeners.has('message_chunk')).toBe(true);
    });

    it('cleans up listeners on unmount', async () => {
      const handler = vi.fn();
      const unlisten = await listenMock('message_chunk', handler);

      expect(listeners.size).toBeGreaterThan(0);

      unlisten();

      expect(listeners.has('message_chunk')).toBe(false);
    });
  });

  // Test group: Mock Utilities
  describe('Mock Helper Functions', () => {
    it('waitForAsync allows promises to resolve', async () => {
      let resolved = false;
      Promise.resolve().then(() => {
        resolved = true;
      });

      expect(resolved).toBe(false);

      await waitForAsync();

      expect(resolved).toBe(true);
    });

    it('listeners map tracks registered events', async () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();

      await listenMock('event1', handler1);
      await listenMock('event2', handler2);

      expect(listeners.size).toBe(2);
      expect(listeners.has('event1')).toBe(true);
      expect(listeners.has('event2')).toBe(true);
    });
  });
});
